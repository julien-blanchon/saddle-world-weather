use bevy::{
    color::LinearRgba,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
};

use crate::{
    WeatherConfig, WeatherRuntime, WeatherSurface, WeatherSurfaceState, WeatherZone,
    resolve_runtime, resolve_zone_profile,
};

#[derive(Component, Debug, Clone)]
pub(crate) struct WeatherSurfaceMaterialBinding {
    pub material: Handle<StandardMaterial>,
    pub base_color: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

pub(crate) fn sync_surface_materials(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<WeatherConfig>,
    runtime: Res<WeatherRuntime>,
    internal: Res<crate::systems::WeatherInternalState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    zones: Query<(&WeatherZone, &GlobalTransform)>,
    mut surfaces: Query<
        (
            Entity,
            &WeatherSurface,
            &GlobalTransform,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut WeatherSurfaceState>,
            Option<&WeatherSurfaceMaterialBinding>,
        ),
    >,
) {
    let dt = time.delta_secs();

    for (entity, surface, transform, mut material_slot, state, binding) in &mut surfaces {
        if !surface.enabled {
            continue;
        }

        let Some(binding) = ensure_material_binding(
            &mut commands,
            &mut materials,
            entity,
            &mut material_slot,
            binding,
        ) else {
            continue;
        };

        let contributions = crate::systems::collect_zone_contributions(
            transform.translation(),
            &zones,
        );
        let zone_result = resolve_zone_profile(&runtime.active_profile, &contributions);
        let (_, precipitation, _, _, _, factors) = resolve_runtime(
            &zone_result.profile,
            config.seed,
            internal.elapsed_time_secs,
        );

        let mut next_state = state.as_deref().cloned().unwrap_or_default();
        next_state.base_profile_label = runtime.active_profile.label.clone();
        next_state.resolved_profile_label = zone_result.profile.label.clone();
        next_state.zone_label = zone_result.dominant_label;
        next_state.precipitation_kind = precipitation.kind;
        next_state.rain_factor = factors.rain_factor;
        next_state.snow_factor = factors.snow_factor;
        next_state.wetness_factor = factors.wetness_factor;

        let wetness_target = (factors.wetness_factor * surface.wetness_response).clamp(0.0, 1.0);
        let puddle_target = puddle_target(surface, factors.rain_factor);
        let snow_target =
            (factors.snow_factor * surface.snow_response * surface.max_snow_coverage).clamp(0.0, 1.0);

        next_state.wetness = approach(
            next_state.wetness,
            wetness_target,
            rate_for_target(
                next_state.wetness,
                wetness_target,
                surface.wetting_speed,
                surface.drying_speed,
            ) * dt,
        );
        next_state.puddle_coverage = approach(
            next_state.puddle_coverage,
            puddle_target,
            rate_for_target(
                next_state.puddle_coverage,
                puddle_target,
                surface.puddle_fill_speed,
                surface.puddle_drain_speed,
            ) * dt,
        );
        next_state.snow_coverage = approach(
            next_state.snow_coverage,
            snow_target,
            rate_for_target(
                next_state.snow_coverage,
                snow_target,
                surface.snow_accumulation_speed,
                surface.snow_melt_speed,
            ) * dt,
        );

        let Some(material) = materials.get_mut(&binding.material) else {
            continue;
        };

        let wet_mix = next_state.wetness.clamp(0.0, 1.0);
        let puddle_mix = next_state.puddle_coverage.clamp(0.0, 1.0);
        let snow_mix = next_state.snow_coverage.clamp(0.0, 1.0);

        let darkening =
            1.0 - surface.wet_darkening * wet_mix - surface.puddle_darkening * puddle_mix;
        let damp_color = scale_color(binding.base_color, darkening.clamp(0.0, 1.0));
        material.base_color = mix_color(damp_color, surface.snow_tint, snow_mix);
        let roughness = lerp(
            lerp(
                binding.perceptual_roughness,
                surface.wet_roughness,
                wet_mix,
            ),
            surface.puddle_roughness,
            puddle_mix,
        );
        material.perceptual_roughness = lerp(roughness, surface.snow_roughness, snow_mix)
            .clamp(0.0, 1.0);
        let reflectance = lerp(
            lerp(binding.reflectance, surface.wet_reflectance, wet_mix),
            surface.puddle_reflectance,
            puddle_mix,
        );
        material.reflectance = lerp(reflectance, surface.snow_reflectance, snow_mix)
            .clamp(0.0, 1.0);
        material.metallic = binding.metallic;

        if let Some(mut state) = state {
            *state = next_state;
        } else {
            commands.entity(entity).insert(next_state);
        }
    }
}

pub(crate) fn reset_surface_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut surfaces: Query<
        (
            Entity,
            &WeatherSurfaceMaterialBinding,
            &MeshMaterial3d<StandardMaterial>,
            Option<&WeatherSurfaceState>,
        ),
    >,
) {
    for (entity, binding, material_slot, state) in &mut surfaces {
        if let Some(material) = materials.get_mut(&material_slot.0) {
            material.base_color = binding.base_color;
            material.perceptual_roughness = binding.perceptual_roughness;
            material.metallic = binding.metallic;
            material.reflectance = binding.reflectance;
        }

        commands.entity(entity).remove::<WeatherSurfaceMaterialBinding>();
        if state.is_some() {
            commands.entity(entity).remove::<WeatherSurfaceState>();
        }
    }
}

fn ensure_material_binding(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    entity: Entity,
    material_slot: &mut MeshMaterial3d<StandardMaterial>,
    existing: Option<&WeatherSurfaceMaterialBinding>,
) -> Option<WeatherSurfaceMaterialBinding> {
    if let Some(existing) = existing {
        if material_slot.0 == existing.material && materials.get(&existing.material).is_some() {
            return Some(existing.clone());
        }
    }

    let source = material_slot.0.clone();
    let material = materials.get(&source)?.clone();
    let unique = materials.add(material.clone());
    *material_slot = MeshMaterial3d(unique.clone());

    let binding = WeatherSurfaceMaterialBinding {
        material: unique,
        base_color: material.base_color,
        perceptual_roughness: material.perceptual_roughness,
        metallic: material.metallic,
        reflectance: material.reflectance,
    };
    commands.entity(entity).insert(binding.clone());
    Some(binding)
}

fn puddle_target(surface: &WeatherSurface, rain_factor: f32) -> f32 {
    if rain_factor <= surface.puddle_threshold {
        0.0
    } else {
        (((rain_factor - surface.puddle_threshold) / (1.0 - surface.puddle_threshold).max(0.001))
            * surface.puddle_response
            * surface.max_puddle_coverage)
            .clamp(0.0, surface.max_puddle_coverage)
    }
}

fn rate_for_target(current: f32, target: f32, rise: f32, fall: f32) -> f32 {
    if target >= current {
        rise.max(0.0)
    } else {
        fall.max(0.0)
    }
}

fn approach(current: f32, target: f32, delta: f32) -> f32 {
    if delta <= 0.0 {
        return current;
    }
    let difference = target - current;
    if difference.abs() <= delta {
        target
    } else {
        current + difference.signum() * delta
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn scale_color(color: Color, factor: f32) -> Color {
    let linear = color.to_linear();
    Color::linear_rgba(
        linear.red * factor,
        linear.green * factor,
        linear.blue * factor,
        linear.alpha,
    )
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let a = a.to_linear();
    let b = b.to_linear();
    let mixed = LinearRgba {
        red: lerp(a.red, b.red, t),
        green: lerp(a.green, b.green, t),
        blue: lerp(a.blue, b.blue, t),
        alpha: lerp(a.alpha, b.alpha, t),
    };
    Color::linear_rgba(mixed.red, mixed.green, mixed.blue, mixed.alpha)
}

#[cfg(test)]
#[path = "surfaces_tests.rs"]
mod tests;
