use bevy::prelude::*;

use crate::{
    WeatherConfig, WeatherRuntime, WeatherSurface, WeatherSurfaceState, WeatherZone,
    resolve_runtime, resolve_zone_profile,
};

pub(crate) fn sync_surface_states(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<WeatherConfig>,
    runtime: Res<WeatherRuntime>,
    internal: Res<crate::systems::WeatherInternalState>,
    zones: Query<(&WeatherZone, &GlobalTransform)>,
    mut surfaces: Query<(
        Entity,
        &WeatherSurface,
        &GlobalTransform,
        Option<&mut WeatherSurfaceState>,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, surface, transform, state) in &mut surfaces {
        if !surface.enabled {
            if state.is_some() {
                commands.entity(entity).remove::<WeatherSurfaceState>();
            }
            continue;
        }

        let contributions =
            crate::systems::collect_zone_contributions(transform.translation(), &zones);
        let zone_result = resolve_zone_profile(&runtime.active_profile, &contributions);
        let (_, precipitation, _, _, factors) = resolve_runtime(
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
        let snow_target = (factors.snow_factor * surface.snow_response * surface.max_snow_coverage)
            .clamp(0.0, 1.0);

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

        if let Some(mut state) = state {
            *state = next_state;
        } else {
            commands.entity(entity).insert(next_state);
        }
    }
}

pub(crate) fn reset_surface_states(
    mut commands: Commands,
    surfaces: Query<(Entity, &WeatherSurfaceState)>,
) {
    for (entity, _) in &surfaces {
        commands.entity(entity).remove::<WeatherSurfaceState>();
    }
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

#[cfg(test)]
#[path = "surfaces_tests.rs"]
mod tests;
