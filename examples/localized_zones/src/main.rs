use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{
    WeatherConfig, WeatherPlugin, WeatherProfile, WeatherQuality, WeatherSurfaceMaterialsPlugin,
    WeatherVisualsConfig, WeatherVisualsPlugin, WeatherVolumeShape, WeatherZone,
};

fn main() {
    let config = WeatherConfig {
        initial_profile: WeatherProfile::clear(),
        seed: 13,
        ..default()
    };
    let visuals = WeatherVisualsConfig {
        quality: WeatherQuality::High,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.58, 0.66, 0.76)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.82, 0.88),
        brightness: 500.0,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather localized_zones".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &config, &visuals);
    app.add_plugins((
        WeatherPlugin::default().with_config(config),
        WeatherVisualsPlugin::default().with_config(visuals),
        WeatherSurfaceMaterialsPlugin::default(),
    ));
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
        "Zone Camera",
        Transform::from_xyz(-22.0, 7.5, 0.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera::default(),
    );
    commands.entity(camera).insert((
        support::PrimaryShowcaseCamera,
        support::AutoOrbitCamera {
            focus: Vec3::new(0.0, 1.8, 0.0),
            radius: 20.0,
            height: 7.8,
            angular_speed: 0.20,
            phase_offset: 0.0,
        },
    ));
    support::spawn_overlay(
        &mut commands,
        "Orbital pass through fog and storm pockets",
        480.0,
    );

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
}
