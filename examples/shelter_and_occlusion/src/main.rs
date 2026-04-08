use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{
    WeatherConfig, WeatherOcclusionVolume, WeatherPlugin, WeatherProfile, WeatherQuality,
    WeatherSurfaceMaterialsPlugin, WeatherVisualsConfig, WeatherVisualsPlugin, WeatherVolumeShape,
};

fn main() {
    let config = WeatherConfig {
        initial_profile: WeatherProfile::storm(),
        seed: 5,
        ..default()
    };
    let visuals = WeatherVisualsConfig {
        quality: WeatherQuality::High,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.54, 0.60, 0.70)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.76, 0.80, 0.86),
        brightness: 480.0,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather shelter_and_occlusion".into(),
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
            support::move_camera_on_rail,
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
        "Shelter Camera",
        Transform::from_xyz(-14.0, 2.8, 0.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera::default(),
    );
    commands.entity(camera).insert((
        support::PrimaryShowcaseCamera,
        support::LinearCameraRail {
            start: Vec3::new(-14.0, 2.8, 0.0),
            end: Vec3::new(14.0, 2.8, 0.0),
            focus: Vec3::new(0.0, 2.0, 0.0),
            speed: 0.28,
            phase_offset: -1.1,
        },
    ));
    support::spawn_overlay(
        &mut commands,
        "Camera moves through a roofed precipitation shelter",
        500.0,
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
