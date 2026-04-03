use bevy::{color::LinearRgba, prelude::*};
use saddle_pane::prelude::*;
use saddle_world_weather::{
    WeatherCamera, WeatherCameraState, WeatherConfig, WeatherDiagnostics, WeatherProfile,
    WeatherQuality, WeatherRuntime, WeatherScreenFxMode,
};

#[derive(Component)]
pub struct ShowcaseSpinner {
    pub axis: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct ShowcaseOverlay;

#[derive(Component)]
pub struct PrimaryShowcaseCamera;

#[derive(Component)]
pub struct AutoOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub height: f32,
    pub angular_speed: f32,
    pub phase_offset: f32,
}

#[derive(Component)]
pub struct LinearCameraRail {
    pub start: Vec3,
    pub end: Vec3,
    pub focus: Vec3,
    pub speed: f32,
    pub phase_offset: f32,
}

#[derive(Resource, Clone, Default, Pane)]
#[pane(title = "Weather Controls", position = "top-right")]
pub struct WeatherDemoPane {
    #[pane(select(options = ["Clear", "Foggy", "Rain", "Storm", "Snow"]))]
    pub profile_index: usize,
    #[pane]
    pub instant_apply: bool,
    #[pane(slider, min = 0.1, max = 8.0, step = 0.1)]
    pub transition_duration_secs: f32,
    #[pane(select(options = ["Low", "Medium", "High"]))]
    pub quality_index: usize,
    #[pane]
    pub built_in_overlays: bool,
    #[pane(monitor)]
    pub rain_factor: f32,
    #[pane(monitor)]
    pub wetness_factor: f32,
    #[pane(monitor)]
    pub visibility_distance: f32,
}

#[derive(Resource, Clone, Copy)]
struct WeatherDemoPaneState {
    last_profile_index: usize,
}

impl WeatherDemoPane {
    pub fn from_config(config: &WeatherConfig) -> Self {
        Self {
            profile_index: profile_to_index(&config.initial_profile),
            instant_apply: false,
            transition_duration_secs: config.default_transition_duration_secs,
            quality_index: quality_to_index(config.quality),
            built_in_overlays: matches!(config.screen_fx_mode, WeatherScreenFxMode::BuiltInOverlay),
            rain_factor: 0.0,
            wetness_factor: 0.0,
            visibility_distance: 0.0,
        }
    }
}

pub fn install_demo_pane(app: &mut App, config: &WeatherConfig) {
    let pane = WeatherDemoPane::from_config(config);
    app.insert_resource(WeatherDemoPaneState {
        last_profile_index: pane.profile_index,
    });
    app.insert_resource(pane);
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<WeatherDemoPane>();
    app.add_systems(Update, (sync_demo_pane, sync_demo_monitors));
}

pub fn spawn_weather_camera(
    commands: &mut Commands,
    name: impl Into<String>,
    transform: Transform,
    weather_camera: WeatherCamera,
) -> Entity {
    commands
        .spawn((
            Name::new(name.into()),
            Camera3d::default(),
            weather_camera,
            transform,
        ))
        .id()
}

pub fn spawn_showcase_environment(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Weather Sun"),
        DirectionalLight {
            illuminance: 42_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.88, 0.42, 0.0)),
    ));

    commands.spawn((
        Name::new("Weather Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(120.0, 120.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.18, 0.20),
            perceptual_roughness: 0.98,
            ..default()
        })),
    ));

    let props = [
        (
            "Weather Tower North",
            Vec3::new(-18.0, 2.8, -12.0),
            Vec3::new(2.4, 5.6, 2.4),
            Color::srgb(0.27, 0.42, 0.72),
        ),
        (
            "Weather Tower West",
            Vec3::new(-7.5, 2.0, 5.0),
            Vec3::new(1.8, 4.0, 1.8),
            Color::srgb(0.72, 0.38, 0.22),
        ),
        (
            "Weather Block Center",
            Vec3::new(0.0, 1.1, -7.0),
            Vec3::new(4.6, 2.2, 4.6),
            Color::srgb(0.26, 0.62, 0.50),
        ),
        (
            "Weather Tower East",
            Vec3::new(12.0, 3.2, 7.0),
            Vec3::new(2.2, 6.4, 2.2),
            Color::srgb(0.80, 0.68, 0.24),
        ),
        (
            "Weather Slab South",
            Vec3::new(5.5, 0.9, 18.0),
            Vec3::new(6.4, 1.8, 3.2),
            Color::srgb(0.38, 0.28, 0.22),
        ),
    ];

    for (index, (name, translation, scale, color)) in props.into_iter().enumerate() {
        commands.spawn((
            Name::new(name),
            Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.04,
                perceptual_roughness: 0.40,
                ..default()
            })),
            Transform::from_translation(translation),
            ShowcaseSpinner {
                axis: Vec3::new(0.18 + index as f32 * 0.08, 1.0, 0.14).normalize(),
                speed: 0.08 + index as f32 * 0.025,
            },
        ));
    }

    spawn_shelter(commands, meshes, materials);
}

pub fn spawn_zone_marker(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    translation: Vec3,
    scale: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.into()),
        Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color.with_alpha(0.18),
            emissive: LinearRgba::from(color) * 0.06,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Transform::from_translation(translation),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &str, width: f32) {
    commands.spawn((
        Name::new(format!("{title} Overlay")),
        ShowcaseOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(width),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.78)),
        Text::new(title),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn animate_props(time: Res<Time>, mut query: Query<(&ShowcaseSpinner, &mut Transform)>) {
    for (spinner, mut transform) in &mut query {
        transform.rotate(Quat::from_axis_angle(
            spinner.axis,
            spinner.speed * time.delta_secs(),
        ));
    }
}

pub fn orbit_camera(time: Res<Time>, mut query: Query<(&AutoOrbitCamera, &mut Transform)>) {
    for (orbit, mut transform) in &mut query {
        let angle = orbit.phase_offset + time.elapsed_secs() * orbit.angular_speed;
        transform.translation = orbit.focus
            + Vec3::new(
                angle.cos() * orbit.radius,
                orbit.height,
                angle.sin() * orbit.radius,
            );
        transform.look_at(orbit.focus + Vec3::Y * 1.8, Vec3::Y);
    }
}

pub fn move_camera_on_rail(
    time: Res<Time>,
    mut query: Query<(&LinearCameraRail, &mut Transform), Without<AutoOrbitCamera>>,
) {
    for (rail, mut transform) in &mut query {
        let t = ((time.elapsed_secs() * rail.speed + rail.phase_offset).sin() * 0.5 + 0.5)
            .clamp(0.0, 1.0);
        transform.translation = rail.start.lerp(rail.end, t);
        transform.look_at(rail.focus, Vec3::Y);
    }
}

pub fn update_weather_overlay(
    runtime: Res<WeatherRuntime>,
    diagnostics: Res<WeatherDiagnostics>,
    primary_camera: Query<&WeatherCameraState, With<PrimaryShowcaseCamera>>,
    mut overlay: Query<&mut Text, With<ShowcaseOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    let camera_lines = if let Ok(camera) = primary_camera.single() {
        format!(
            "Camera base {}  resolved {}  zone {}\nPrecip {:?} {:>4.2}  Screen {:>4.2}\nFog {:>4.2}  Visibility {:>6.1}  Far {:>4.2}\nWind [{:>5.2}, {:>5.2}, {:>5.2}] x {:>4.2}  Particles {}\nWetness {:>4.2}  Lightning {:>4.2}",
            camera.base_profile_label.as_deref().unwrap_or("Unnamed"),
            camera
                .resolved_profile_label
                .as_deref()
                .unwrap_or("Unnamed"),
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
    } else {
        "Camera state unavailable".to_string()
    };

    text.0 = format!(
        "Weather Showcase\nActive {}  Target {}\nTransition {:>4.2}  Active? {}\nWind factor {:>4.2}  Wetness {:>4.2}  Storm {:>4.2}\nQuality {:?}  Fog {:>4.2}  Global precip {:?}\nEmitters {}  Estimated particles {}  Overlays {}\nMessages start={} finish={} changed={} flash={}\nPrimary camera {}  Primary zone {}\n{}",
        runtime.active_profile.label.as_deref().unwrap_or("Unnamed"),
        runtime.target_profile.label.as_deref().unwrap_or("Unnamed"),
        runtime.transition.progress,
        runtime.transition.active,
        runtime.factors.wind_factor,
        runtime.factors.wetness_factor,
        runtime.factors.storm_factor,
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
        diagnostics
            .primary_camera_name
            .as_deref()
            .unwrap_or("Unknown"),
        diagnostics
            .primary_zone_label
            .as_deref()
            .unwrap_or("Global"),
        camera_lines,
    );
}

fn sync_demo_pane(
    pane: Res<WeatherDemoPane>,
    mut pane_state: ResMut<WeatherDemoPaneState>,
    mut config: ResMut<WeatherConfig>,
) {
    let desired_transition = pane.transition_duration_secs.max(0.05);
    let desired_quality = index_to_quality(pane.quality_index);
    let desired_screen_fx_mode = if pane.built_in_overlays {
        WeatherScreenFxMode::BuiltInOverlay
    } else {
        WeatherScreenFxMode::StateOnly
    };
    if (config.default_transition_duration_secs - desired_transition).abs() > f32::EPSILON {
        config.default_transition_duration_secs = desired_transition;
    }
    if config.quality != desired_quality {
        config.quality = desired_quality;
    }
    if config.screen_fx_mode != desired_screen_fx_mode {
        config.screen_fx_mode = desired_screen_fx_mode;
    }

    if pane.profile_index != pane_state.last_profile_index {
        let profile = profile_from_index(pane.profile_index);
        if pane.instant_apply {
            config.queue_immediate(profile);
        } else {
            config.queue_transition(profile, pane.transition_duration_secs.max(0.05));
        }
        pane_state.last_profile_index = pane.profile_index;
    }
}

fn sync_demo_monitors(runtime: Res<WeatherRuntime>, mut pane: ResMut<WeatherDemoPane>) {
    pane.rain_factor = runtime.factors.rain_factor;
    pane.wetness_factor = runtime.factors.wetness_factor;
    pane.visibility_distance = runtime.visibility.visibility_distance;
}

fn spawn_shelter(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Shelter Roof"),
        Mesh3d(meshes.add(Cuboid::new(12.0, 0.6, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.20, 0.24),
            perceptual_roughness: 0.92,
            ..default()
        })),
        Transform::from_xyz(0.0, 4.2, 0.0),
    ));

    for (index, translation) in [
        Vec3::new(-5.2, 2.0, -3.0),
        Vec3::new(5.2, 2.0, -3.0),
        Vec3::new(-5.2, 2.0, 3.0),
        Vec3::new(5.2, 2.0, 3.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Shelter Column {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(0.55, 4.0, 0.55))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.48, 0.45, 0.38),
                perceptual_roughness: 0.88,
                ..default()
            })),
            Transform::from_translation(translation),
        ));
    }
}

fn profile_to_index(profile: &WeatherProfile) -> usize {
    match profile.label.as_deref() {
        Some("Foggy") => 1,
        Some("Rain") => 2,
        Some("Storm") => 3,
        Some("Snow") => 4,
        _ => 0,
    }
}

fn profile_from_index(index: usize) -> WeatherProfile {
    match index {
        1 => WeatherProfile::foggy(),
        2 => WeatherProfile::rain(),
        3 => WeatherProfile::storm(),
        4 => WeatherProfile::snow(),
        _ => WeatherProfile::clear(),
    }
}

fn quality_to_index(quality: WeatherQuality) -> usize {
    match quality {
        WeatherQuality::Low => 0,
        WeatherQuality::Medium => 1,
        WeatherQuality::High => 2,
    }
}

fn index_to_quality(index: usize) -> WeatherQuality {
    match index {
        0 => WeatherQuality::Low,
        1 => WeatherQuality::Medium,
        _ => WeatherQuality::High,
    }
}
