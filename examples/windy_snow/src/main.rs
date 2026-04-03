use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherQuality};

fn main() {
    let config = WeatherConfig {
        quality: WeatherQuality::High,
        initial_profile: windy_snow_profile(),
        seed: 42,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.60, 0.68, 0.78)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.84, 0.88, 0.94),
        brightness: 460.0,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather windy_snow".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &config);
    app.add_plugins(WeatherPlugin::default().with_config(config));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::animate_props,
            support::orbit_camera,
            support::update_weather_overlay,
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    support::spawn_showcase_environment(&mut commands, meshes.as_mut(), materials.as_mut());
    let camera = support::spawn_weather_camera(
        &mut commands,
        "Windy Snow Camera",
        Transform::from_xyz(-12.0, 8.0, -18.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera::default(),
    );
    commands.entity(camera).insert((
        support::PrimaryShowcaseCamera,
        support::AutoOrbitCamera {
            focus: Vec3::new(0.0, 1.5, 0.0),
            radius: 24.0,
            height: 10.5,
            angular_speed: 0.18,
            phase_offset: -0.6,
        },
    ));
    support::spawn_overlay(&mut commands, "Wind-heavy snowfall with gusts", 460.0);
}

fn windy_snow_profile() -> WeatherProfile {
    let mut profile = WeatherProfile::snow();
    profile.label = Some("Windy Snow".into());
    profile.precipitation.wind_influence = 1.45;
    profile.precipitation.fall_speed = 5.5;
    profile.precipitation.near_radius = 14.0;
    profile.precipitation.density = 0.82;
    profile.fog.density = 0.24;
    profile.fog.visibility_distance = 72.0;
    profile.wind.direction = Vec3::new(-1.0, 0.0, 0.35);
    profile.wind.base_speed = 12.0;
    profile.wind.gust_amplitude = 0.82;
    profile.wind.gust_frequency_hz = 0.55;
    profile.wind.sway = 0.95;
    profile.screen_fx.intensity = 0.32;
    profile.screen_fx.frost_intensity = 0.55;
    profile
}
