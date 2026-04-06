use bevy::{
    color::LinearRgba,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
};

use crate::{WeatherSurfaceStandardMaterial, WeatherSurfaceState};

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut surfaces: Query<(
        Entity,
        &WeatherSurfaceStandardMaterial,
        &mut MeshMaterial3d<StandardMaterial>,
        Option<&WeatherSurfaceState>,
        Option<&WeatherSurfaceMaterialBinding>,
    )>,
) {
    for (entity, settings, mut material_slot, state, binding) in &mut surfaces {
        let Some(binding) = ensure_material_binding(
            &mut commands,
            &mut materials,
            entity,
            &mut material_slot,
            binding,
        ) else {
            continue;
        };

        let Some(material) = materials.get_mut(&binding.material) else {
            continue;
        };

        if !settings.enabled {
            restore_material(material, &binding);
            continue;
        }

        let Some(state) = state else {
            restore_material(material, &binding);
            continue;
        };

        let wet_mix = state.wetness.clamp(0.0, 1.0);
        let puddle_mix = state.puddle_coverage.clamp(0.0, 1.0);
        let snow_mix = state.snow_coverage.clamp(0.0, 1.0);

        let darkening =
            1.0 - settings.wet_darkening * wet_mix - settings.puddle_darkening * puddle_mix;
        let damp_color = scale_color(binding.base_color, darkening.clamp(0.0, 1.0));
        material.base_color = mix_color(damp_color, settings.snow_tint, snow_mix);
        let roughness = lerp(
            lerp(
                binding.perceptual_roughness,
                settings.wet_roughness,
                wet_mix,
            ),
            settings.puddle_roughness,
            puddle_mix,
        );
        material.perceptual_roughness =
            lerp(roughness, settings.snow_roughness, snow_mix).clamp(0.0, 1.0);
        let reflectance = lerp(
            lerp(binding.reflectance, settings.wet_reflectance, wet_mix),
            settings.puddle_reflectance,
            puddle_mix,
        );
        material.reflectance =
            lerp(reflectance, settings.snow_reflectance, snow_mix).clamp(0.0, 1.0);
        material.metallic = binding.metallic;
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
        ),
        With<WeatherSurfaceStandardMaterial>,
    >,
) {
    for (entity, binding, material_slot) in &mut surfaces {
        if let Some(material) = materials.get_mut(&material_slot.0) {
            restore_material(material, binding);
        }

        commands
            .entity(entity)
            .remove::<WeatherSurfaceMaterialBinding>();
    }
}

fn restore_material(material: &mut StandardMaterial, binding: &WeatherSurfaceMaterialBinding) {
    material.base_color = binding.base_color;
    material.perceptual_roughness = binding.perceptual_roughness;
    material.metallic = binding.metallic;
    material.reflectance = binding.reflectance;
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
#[path = "surface_materials_tests.rs"]
mod tests;
