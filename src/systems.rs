use bevy::prelude::*;

use crate::{
    LightningFlashEmitted, WeatherCamera, WeatherCameraState, WeatherConfig, WeatherDiagnostics,
    WeatherOcclusionVolume, WeatherProfile, WeatherProfileChanged, WeatherRuntime,
    WeatherTransitionFinished, WeatherTransitionMode, WeatherTransitionStarted, WeatherZone,
    resolve_occlusion, resolve_runtime, resolve_zone_profile,
    solver::{OcclusionContribution, ZoneContribution},
};

#[derive(Resource, Default)]
pub(crate) struct PendingWeatherMessages {
    pub transition_started: Option<WeatherTransitionStarted>,
    pub transition_finished: Option<WeatherTransitionFinished>,
    pub profile_changed: Option<WeatherProfileChanged>,
    pub lightning_flash: Option<LightningFlashEmitted>,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub(crate) struct WeatherInternalState {
    pub active: bool,
    pub elapsed_time_secs: f32,
    pub source_profile: WeatherProfile,
}

impl Default for WeatherInternalState {
    fn default() -> Self {
        Self {
            active: false,
            elapsed_time_secs: 0.0,
            source_profile: WeatherProfile::clear(),
        }
    }
}

pub(crate) fn runtime_is_active(state: Option<Res<WeatherInternalState>>) -> bool {
    state.is_some_and(|state| state.active)
}

pub(crate) fn activate_runtime(
    config: Res<WeatherConfig>,
    mut runtime: ResMut<WeatherRuntime>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    mut internal: ResMut<WeatherInternalState>,
    mut pending: ResMut<PendingWeatherMessages>,
) {
    let initial = config.initial_profile.clone().clamped();
    internal.active = true;
    internal.elapsed_time_secs = 0.0;
    internal.source_profile = initial.clone();
    pending.transition_started = None;
    pending.transition_finished = None;
    pending.profile_changed = None;
    pending.lightning_flash = None;

    runtime.active_profile = initial.clone();
    runtime.target_profile = initial.clone();
    runtime.transition = default_transition_state(&initial);
    apply_resolved_runtime(&mut runtime, &config, 0.0);

    diagnostics.active_profile_label = runtime.active_profile.label.clone();
    diagnostics.target_profile_label = runtime.target_profile.label.clone();
    diagnostics.transition_progress = runtime.transition.progress;
    diagnostics.transition_active = runtime.transition.active;
    diagnostics.active_zone_count = 0;
    diagnostics.current_wind = runtime.wind.vector;
    diagnostics.current_fog_density = runtime.visibility.fog_density;
    diagnostics.current_visibility_distance = runtime.visibility.visibility_distance;
    diagnostics.current_precipitation_kind = runtime.precipitation.kind.clone();
    diagnostics.primary_camera_name = None;
    diagnostics.primary_zone_label = None;
    diagnostics.last_transition_started_at = None;
    diagnostics.last_transition_finished_at = None;
    diagnostics.last_lightning_flash_id = runtime.storm.lightning_flash_id;
    diagnostics.transition_started_count = 0;
    diagnostics.transition_finished_count = 0;
    diagnostics.profile_changed_count = 0;
    diagnostics.lightning_flash_count = 0;
}

pub(crate) fn deactivate_runtime(mut internal: ResMut<WeatherInternalState>) {
    internal.active = false;
}

pub(crate) fn cleanup_runtime(
    mut commands: Commands,
    camera_states: Query<Entity, With<WeatherCameraState>>,
) {
    for entity in &camera_states {
        commands.entity(entity).remove::<WeatherCameraState>();
    }
}

pub(crate) fn apply_weather_requests(
    mut config: ResMut<WeatherConfig>,
    mut runtime: ResMut<WeatherRuntime>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    mut internal: ResMut<WeatherInternalState>,
    mut pending: ResMut<PendingWeatherMessages>,
) {
    let Some(request) = config.pending_request.take() else {
        return;
    };

    let target = request.profile.clamped();
    match request.mode {
        WeatherTransitionMode::Immediate if request.duration_secs <= 0.0 => {
            internal.source_profile = target.clone();
            runtime.active_profile = target.clone();
            runtime.target_profile = target.clone();
            runtime.transition = default_transition_state(&target);
            pending.profile_changed = Some(WeatherProfileChanged {
                active_label: target.label.clone(),
            });
            diagnostics.last_transition_finished_at = Some(internal.elapsed_time_secs);
        }
        WeatherTransitionMode::Immediate => {
            internal.source_profile = target.clone();
            runtime.active_profile = target.clone();
            runtime.target_profile = target.clone();
            runtime.transition = default_transition_state(&target);
            pending.profile_changed = Some(WeatherProfileChanged {
                active_label: target.label.clone(),
            });
            diagnostics.last_transition_finished_at = Some(internal.elapsed_time_secs);
        }
        WeatherTransitionMode::Smooth => {
            let duration_secs = if request.duration_secs <= 0.0 {
                config.default_transition_duration_secs.max(0.01)
            } else {
                request.duration_secs.max(0.01)
            };
            internal.source_profile = runtime.active_profile.clone();
            runtime.target_profile = target.clone();
            runtime.transition.active = true;
            runtime.transition.elapsed_secs = 0.0;
            runtime.transition.duration_secs = duration_secs;
            runtime.transition.progress = 0.0;
            runtime.transition.source_label = internal.source_profile.label.clone();
            runtime.transition.target_label = target.label.clone();

            pending.transition_started = Some(WeatherTransitionStarted {
                from_label: internal.source_profile.label.clone(),
                to_label: target.label.clone(),
                duration_secs,
            });
            diagnostics.last_transition_started_at = Some(internal.elapsed_time_secs);
        }
    }
}

pub(crate) fn advance_transition(
    time: Res<Time>,
    mut runtime: ResMut<WeatherRuntime>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    mut pending: ResMut<PendingWeatherMessages>,
    mut internal: ResMut<WeatherInternalState>,
) {
    let dt = time.delta_secs();
    internal.elapsed_time_secs += dt;

    if runtime.transition.active {
        runtime.transition.elapsed_secs += dt;
        runtime.transition.progress = if runtime.transition.duration_secs <= 0.0 {
            1.0
        } else {
            (runtime.transition.elapsed_secs / runtime.transition.duration_secs).clamp(0.0, 1.0)
        };
        runtime.active_profile = internal
            .source_profile
            .blend(&runtime.target_profile, runtime.transition.progress);

        if runtime.transition.progress >= 1.0 {
            runtime.transition.active = false;
            runtime.transition.progress = 1.0;
            runtime.active_profile = runtime.target_profile.clone();
            internal.source_profile = runtime.active_profile.clone();
            pending.transition_finished = Some(WeatherTransitionFinished {
                active_label: runtime.active_profile.label.clone(),
            });
            pending.profile_changed = Some(WeatherProfileChanged {
                active_label: runtime.active_profile.label.clone(),
            });
            diagnostics.last_transition_finished_at = Some(internal.elapsed_time_secs);
        }
    } else {
        runtime.active_profile = runtime.target_profile.clone();
        runtime.transition.progress = 1.0;
        internal.source_profile = runtime.active_profile.clone();
    }
}

pub(crate) fn resolve_base_runtime(
    config: Res<WeatherConfig>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    mut pending: ResMut<PendingWeatherMessages>,
    mut runtime: ResMut<WeatherRuntime>,
    internal: Res<WeatherInternalState>,
) {
    let previous_flash = runtime.storm.lightning_flash_id;
    let previous_active = runtime.storm.lightning_active;
    apply_resolved_runtime(&mut runtime, &config, internal.elapsed_time_secs);

    if runtime.storm.lightning_active
        && (!previous_active || runtime.storm.lightning_flash_id != previous_flash)
    {
        if let Some(flash_id) = runtime.storm.lightning_flash_id {
            pending.lightning_flash = Some(LightningFlashEmitted {
                flash_id,
                intensity: runtime.storm.lightning_flash_intensity,
            });
            diagnostics.last_lightning_flash_id = Some(flash_id);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_camera_states(
    mut commands: Commands,
    config: Res<WeatherConfig>,
    runtime: Res<WeatherRuntime>,
    internal: Res<WeatherInternalState>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    cameras: Query<
        (
            Entity,
            &WeatherCamera,
            Option<&Name>,
            &GlobalTransform,
            Option<&WeatherCameraState>,
        ),
        With<Camera>,
    >,
    zones: Query<(&WeatherZone, &GlobalTransform)>,
    occlusion_volumes: Query<(&WeatherOcclusionVolume, &GlobalTransform)>,
) {
    let mut best_priority = i32::MIN;
    let mut primary_camera_name = None;
    let mut primary_zone_label = None;
    let mut primary_zone_count = 0usize;

    for (entity, weather_camera, name, global_transform, existing_state) in &cameras {
        if !weather_camera.enabled {
            if existing_state.is_some() {
                commands.entity(entity).remove::<WeatherCameraState>();
            }
            continue;
        }

        let camera_position = global_transform.translation();
        let zone_contributions = collect_zone_contributions(camera_position, &zones);
        let zone_result = resolve_zone_profile(&runtime.active_profile, &zone_contributions);
        let (wind, precipitation, visibility, storm, factors) = resolve_runtime(
            &zone_result.profile,
            config.seed,
            internal.elapsed_time_secs,
        );
        let occlusion_result = resolve_occlusion(&collect_occlusion_contributions(
            camera_position,
            weather_camera,
            &occlusion_volumes,
        ));

        let precipitation_factor = if weather_camera.receive_precipitation {
            precipitation.intensity
                * occlusion_result.precipitation_multiplier
                * (1.0 - weather_camera.precipitation_blocked_factor).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let state = WeatherCameraState {
            base_profile_label: runtime.active_profile.label.clone(),
            resolved_profile_label: zone_result.profile.label.clone(),
            zone_label: zone_result.dominant_label.clone(),
            transition_progress: runtime.transition.progress,
            precipitation_kind: precipitation.kind.clone(),
            precipitation_factor,
            precipitation_density: precipitation.density,
            fall_speed: precipitation.fall_speed,
            particle_size: precipitation.particle_size,
            wind_influence: precipitation.wind_influence,
            near_radius: precipitation.near_radius,
            near_height: precipitation.near_height,
            far_density: precipitation.far_density,
            occlusion_factor: occlusion_result.precipitation_multiplier,
            screen_occlusion_factor: occlusion_result.screen_fx_multiplier,
            wetness_factor: factors.wetness_factor,
            fog_density: visibility.fog_density,
            fog_color: visibility.fog_color,
            visibility_distance: visibility.visibility_distance,
            wind_vector: wind.vector,
            lightning_flash_intensity: storm.lightning_flash_intensity,
        };

        commands.entity(entity).insert(state);

        if weather_camera.priority >= best_priority {
            best_priority = weather_camera.priority;
            primary_camera_name = name.map(|value| value.as_str().to_owned());
            primary_zone_label = zone_result.dominant_label.clone();
            primary_zone_count = zone_result.active_count;
        }
    }

    diagnostics.primary_camera_name = primary_camera_name;
    diagnostics.primary_zone_label = primary_zone_label;
    diagnostics.active_zone_count = primary_zone_count;
}

pub(crate) fn emit_pending_messages(
    mut pending: ResMut<PendingWeatherMessages>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
    mut started: MessageWriter<WeatherTransitionStarted>,
    mut finished: MessageWriter<WeatherTransitionFinished>,
    mut changed: MessageWriter<WeatherProfileChanged>,
    mut lightning: MessageWriter<LightningFlashEmitted>,
) {
    if let Some(message) = pending.transition_started.take() {
        started.write(message);
        diagnostics.transition_started_count += 1;
    }
    if let Some(message) = pending.transition_finished.take() {
        finished.write(message);
        diagnostics.transition_finished_count += 1;
    }
    if let Some(message) = pending.profile_changed.take() {
        changed.write(message);
        diagnostics.profile_changed_count += 1;
    }
    if let Some(message) = pending.lightning_flash.take() {
        lightning.write(message);
        diagnostics.lightning_flash_count += 1;
    }
}

pub(crate) fn publish_diagnostics(
    runtime: Res<WeatherRuntime>,
    mut diagnostics: ResMut<WeatherDiagnostics>,
) {
    diagnostics.active_profile_label = runtime.active_profile.label.clone();
    diagnostics.target_profile_label = runtime.target_profile.label.clone();
    diagnostics.transition_progress = runtime.transition.progress;
    diagnostics.transition_active = runtime.transition.active;
    diagnostics.current_wind = runtime.wind.vector;
    diagnostics.current_fog_density = runtime.visibility.fog_density;
    diagnostics.current_visibility_distance = runtime.visibility.visibility_distance;
    diagnostics.current_precipitation_kind = runtime.precipitation.kind.clone();
}

fn apply_resolved_runtime(runtime: &mut WeatherRuntime, config: &WeatherConfig, time_secs: f32) {
    let (wind, precipitation, visibility, storm, factors) =
        resolve_runtime(&runtime.active_profile, config.seed, time_secs);
    runtime.wind = wind;
    runtime.precipitation = precipitation;
    runtime.visibility = visibility;
    runtime.storm = storm;
    runtime.factors = factors;
}

fn default_transition_state(profile: &WeatherProfile) -> crate::WeatherTransitionState {
    crate::WeatherTransitionState {
        active: false,
        elapsed_secs: 0.0,
        duration_secs: 0.0,
        progress: 1.0,
        source_label: profile.label.clone(),
        target_label: profile.label.clone(),
    }
}

pub(crate) fn collect_zone_contributions(
    camera_position: Vec3,
    zones: &Query<(&WeatherZone, &GlobalTransform)>,
) -> Vec<ZoneContribution> {
    let mut result = Vec::new();
    for (zone, transform) in zones {
        if !zone.enabled {
            continue;
        }
        let local = transform
            .to_matrix()
            .inverse()
            .transform_point3(camera_position);
        let influence = zone.shape.influence(local, zone.blend_distance);
        if influence <= 0.0 {
            continue;
        }
        result.push(ZoneContribution {
            label: zone.label.clone(),
            priority: zone.priority,
            weight: zone.weight.clamp(0.0, 4.0),
            influence,
            profile: zone.profile.clone().clamped(),
        });
    }
    result
}

fn collect_occlusion_contributions(
    camera_position: Vec3,
    weather_camera: &WeatherCamera,
    occlusion_volumes: &Query<(&WeatherOcclusionVolume, &GlobalTransform)>,
) -> Vec<OcclusionContribution> {
    let mut result = Vec::new();
    if weather_camera.precipitation_blocked_factor > 0.0 {
        result.push(OcclusionContribution {
            precipitation_multiplier: (1.0 - weather_camera.precipitation_blocked_factor)
                .clamp(0.0, 1.0),
            screen_fx_multiplier: (1.0 - weather_camera.precipitation_blocked_factor * 0.8)
                .clamp(0.0, 1.0),
            influence: 1.0,
        });
    }

    for (volume, transform) in occlusion_volumes {
        if !volume.enabled {
            continue;
        }
        let local = transform
            .to_matrix()
            .inverse()
            .transform_point3(camera_position);
        let influence = volume.shape.influence(local, volume.blend_distance);
        if influence <= 0.0 {
            continue;
        }
        result.push(OcclusionContribution {
            precipitation_multiplier: volume.precipitation_multiplier.clamp(0.0, 1.0),
            screen_fx_multiplier: volume.screen_fx_multiplier.clamp(0.0, 1.0),
            influence,
        });
    }
    result
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
