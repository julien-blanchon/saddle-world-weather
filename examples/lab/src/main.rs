#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;
use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{
    LightningFlashEmitted, WeatherCameraState, WeatherConfig, WeatherDiagnostics,
    WeatherOcclusionVolume, WeatherPlugin, WeatherProfile, WeatherProfileChanged, WeatherQuality,
    WeatherRuntime, WeatherSystems, WeatherTransitionFinished, WeatherTransitionStarted,
    WeatherVolumeShape, WeatherZone,
};

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct WeatherMessageLog {
    pub transition_started: u32,
    pub transition_finished: u32,
    pub profile_changed: u32,
    pub lightning_flashes: u32,
    pub last_flash_id: Option<u64>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualitySnapshot {
    pub low_particles: usize,
    pub high_particles: usize,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.56, 0.63, 0.72)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.82, 0.88),
        brightness: 520.0,
        ..default()
    });
    app.insert_resource(WeatherMessageLog::default());
    app.insert_resource(QualitySnapshot::default());
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Weather Lab".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::E2EPlugin);
    app.add_plugins(WeatherPlugin::default().with_config(lab_config()));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::animate_props,
            update_lab_overlay.after(WeatherSystems::Diagnostics),
            count_messages.after(WeatherSystems::EmitMessages),
        ),
    );
    app.run();
}

fn lab_config() -> WeatherConfig {
    WeatherConfig {
        initial_profile: WeatherProfile::clear(),
        quality: WeatherQuality::High,
        seed: 21,
        default_transition_duration_secs: 1.5,
        diagnostics_enabled: true,
        pending_request: None,
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    support::spawn_showcase_environment(&mut commands, meshes.as_mut(), materials.as_mut());
    let camera = support::spawn_weather_camera(
        &mut commands,
        "Lab Camera",
        Transform::from_xyz(-15.0, 7.6, -14.5).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera::default(),
    );
    commands
        .entity(camera)
        .insert(support::PrimaryShowcaseCamera);
    support::spawn_overlay(&mut commands, "Weather Lab", 520.0);

    commands.spawn((
        Name::new("Fog Pocket"),
        WeatherZone {
            label: Some("Fog Pocket".into()),
            profile: WeatherProfile::foggy(),
            shape: WeatherVolumeShape::Sphere { radius: 9.5 },
            blend_distance: 6.0,
            priority: 1,
            weight: 1.0,
            ..default()
        },
        Transform::from_xyz(-18.0, 2.0, 0.0),
    ));
    support::spawn_zone_marker(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        "Fog Pocket Marker",
        Vec3::new(-18.0, 2.0, 0.0),
        Vec3::new(19.0, 4.0, 19.0),
        Color::srgb(0.78, 0.88, 0.95),
    );

    commands.spawn((
        Name::new("Storm Cell"),
        WeatherZone {
            label: Some("Storm Cell".into()),
            profile: WeatherProfile::storm(),
            shape: WeatherVolumeShape::Sphere { radius: 11.0 },
            blend_distance: 7.0,
            priority: 2,
            weight: 1.0,
            ..default()
        },
        Transform::from_xyz(18.0, 2.5, 0.0),
    ));
    support::spawn_zone_marker(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        "Storm Cell Marker",
        Vec3::new(18.0, 2.5, 0.0),
        Vec3::new(22.0, 5.0, 22.0),
        Color::srgb(0.58, 0.68, 0.98),
    );

    let mut snow_ridge = WeatherProfile::snow();
    snow_ridge.label = Some("Snow Ridge".into());
    snow_ridge.fog.visibility_distance = 65.0;
    snow_ridge.wind.base_speed = 8.0;
    snow_ridge.wind.gust_amplitude = 0.55;
    commands.spawn((
        Name::new("Snow Ridge"),
        WeatherZone {
            label: Some("Snow Ridge".into()),
            profile: snow_ridge,
            shape: WeatherVolumeShape::Box {
                half_extents: Vec3::new(8.0, 5.0, 8.0),
            },
            blend_distance: 5.0,
            priority: 1,
            weight: 0.8,
            ..default()
        },
        Transform::from_xyz(0.0, 2.5, 18.0),
    ));
    support::spawn_zone_marker(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        "Snow Ridge Marker",
        Vec3::new(0.0, 2.5, 18.0),
        Vec3::new(16.0, 5.0, 16.0),
        Color::srgb(0.92, 0.96, 1.0),
    );

    commands.spawn((
        Name::new("Shelter Occlusion"),
        WeatherOcclusionVolume {
            label: Some("Shelter".into()),
            shape: WeatherVolumeShape::Box {
                half_extents: Vec3::new(5.8, 3.1, 3.8),
            },
            blend_distance: 2.0,
            precipitation_multiplier: 0.05,
            screen_fx_multiplier: 0.12,
            ..default()
        },
        Transform::from_xyz(0.0, 2.2, 0.0),
    ));
    support::spawn_zone_marker(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        "Shelter Marker",
        Vec3::new(0.0, 2.2, 0.0),
        Vec3::new(11.6, 6.2, 7.6),
        Color::srgb(0.84, 0.72, 0.42),
    );
}

fn count_messages(
    mut log: ResMut<WeatherMessageLog>,
    mut started: MessageReader<WeatherTransitionStarted>,
    mut finished: MessageReader<WeatherTransitionFinished>,
    mut changed: MessageReader<WeatherProfileChanged>,
    mut lightning: MessageReader<LightningFlashEmitted>,
) {
    log.transition_started += started.read().count() as u32;
    log.transition_finished += finished.read().count() as u32;
    log.profile_changed += changed.read().count() as u32;
    for message in lightning.read() {
        log.lightning_flashes += 1;
        log.last_flash_id = Some(message.flash_id);
    }
}

fn update_lab_overlay(
    runtime: Res<WeatherRuntime>,
    diagnostics: Res<WeatherDiagnostics>,
    log: Res<WeatherMessageLog>,
    primary_camera: Query<&WeatherCameraState, With<support::PrimaryShowcaseCamera>>,
    mut overlay: Query<&mut Text, With<support::ShowcaseOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    let camera_line = primary_camera
        .single()
        .map(|camera| {
            format!(
                "Camera base {}  resolved {}  zone {}\nPrecip {:?} {:>4.2}  screen {:>4.2}\nFog {:>4.2}  visibility {:>6.1}  far {:>4.2}\nWind [{:>5.2}, {:>5.2}, {:>5.2}] x {:>4.2}  particles {}\nWetness {:>4.2}  flash {:>4.2}",
                camera.base_profile_label.as_deref().unwrap_or("Unnamed"),
                camera.resolved_profile_label.as_deref().unwrap_or("Unnamed"),
                camera.zone_label.as_deref().unwrap_or("Global"),
                camera.precipitation_kind,
                camera.precipitation_factor,
                camera.screen_fx_factor,
                camera.fog_density,
                camera.visibility_distance,
                camera.far_density,
                camera.wind_vector.x,
                camera.wind_vector.y,
                camera.wind_vector.z,
                camera.wind_influence,
                camera.active_particles,
                camera.wetness_factor,
                camera.lightning_flash_intensity,
            )
        })
        .unwrap_or_else(|_| "Camera state unavailable".into());

    text.0 = format!(
        "Weather Lab\nActive {}  Target {}\nTransition {:>4.2} active={}  storm {:>4.2}\nRain {:>4.2}  Snow {:>4.2}  Fog {:>4.2}  Wet {:>4.2}\nQuality {:?}  Global fog {:>4.2}  Global precip {:?}\nEmitters {}  particles {}  overlays {}\nDiagnostics start={} finish={} changed={} flash={}\nScenario log start={} finish={} changed={} flash={}\nPrimary camera {}  primary zone {}\n{}",
        runtime.active_profile.label.as_deref().unwrap_or("Unnamed"),
        runtime.target_profile.label.as_deref().unwrap_or("Unnamed"),
        runtime.transition.progress,
        runtime.transition.active,
        runtime.factors.storm_factor,
        runtime.factors.rain_factor,
        runtime.factors.snow_factor,
        runtime.factors.fog_factor,
        runtime.factors.wetness_factor,
        diagnostics.quality,
        diagnostics.current_fog_density,
        diagnostics.current_precipitation_kind,
        diagnostics.active_emitters,
        diagnostics.precipitation_particles_estimate,
        diagnostics.managed_screen_overlays,
        diagnostics.transition_started_count,
        diagnostics.transition_finished_count,
        diagnostics.profile_changed_count,
        diagnostics.lightning_flash_count,
        log.transition_started,
        log.transition_finished,
        log.profile_changed,
        log.lightning_flashes,
        diagnostics
            .primary_camera_name
            .as_deref()
            .unwrap_or("Unknown"),
        diagnostics
            .primary_zone_label
            .as_deref()
            .unwrap_or("Global"),
        camera_line,
    );
}
