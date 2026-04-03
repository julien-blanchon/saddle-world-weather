use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherQuality};

#[derive(Resource)]
struct TransitionCycle {
    timer: Timer,
    index: usize,
}

fn main() {
    let config = WeatherConfig {
        quality: WeatherQuality::High,
        initial_profile: WeatherProfile::clear(),
        seed: 7,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.56, 0.63, 0.72)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.82, 0.88),
        brightness: 520.0,
        ..default()
    });
    app.insert_resource(TransitionCycle {
        timer: Timer::from_seconds(4.0, TimerMode::Repeating),
        index: 0,
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather transitions".into(),
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
            cycle_profiles,
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
        "Transition Camera",
        Transform::from_xyz(-16.0, 8.5, -16.0).looking_at(Vec3::new(0.0, 1.7, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera::default(),
    );
    commands
        .entity(camera)
        .insert(support::PrimaryShowcaseCamera);
    support::spawn_overlay(
        &mut commands,
        "Clear -> foggy -> rain -> storm -> snow",
        460.0,
    );
}

fn cycle_profiles(
    time: Res<Time>,
    mut cycle: ResMut<TransitionCycle>,
    mut config: ResMut<WeatherConfig>,
) {
    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    let profiles = [
        WeatherProfile::foggy(),
        WeatherProfile::rain(),
        WeatherProfile::storm(),
        WeatherProfile::snow(),
        WeatherProfile::clear(),
    ];
    let profile = profiles[cycle.index % profiles.len()].clone();
    cycle.index += 1;
    config.queue_transition(profile, 1.8);
}
