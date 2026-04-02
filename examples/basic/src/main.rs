use saddle_world_weather_example_support as support;

use bevy::prelude::*;
use saddle_world_weather::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherQuality};

#[derive(Resource)]
struct BasicCycle {
    timer: Timer,
    raining: bool,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.58, 0.66, 0.78)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.82, 0.88),
        brightness: 480.0,
        ..default()
    });
    app.insert_resource(BasicCycle {
        timer: Timer::from_seconds(3.5, TimerMode::Repeating),
        raining: false,
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather basic".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(WeatherPlugin::default().with_config(WeatherConfig {
        quality: WeatherQuality::Medium,
        initial_profile: WeatherProfile::clear(),
        ..default()
    }));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            support::animate_props,
            drive_cycle,
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
        "Basic Camera",
        Transform::from_xyz(-14.0, 7.2, -15.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
        saddle_world_weather::WeatherCamera {
            receive_screen_fx: false,
            ..default()
        },
    );
    commands
        .entity(camera)
        .insert(support::PrimaryShowcaseCamera);
    support::spawn_overlay(&mut commands, "Basic clear <-> rain transition", 460.0);
}

fn drive_cycle(time: Res<Time>, mut cycle: ResMut<BasicCycle>, mut config: ResMut<WeatherConfig>) {
    if cycle.timer.tick(time.delta()).just_finished() {
        if cycle.raining {
            config.queue_transition(WeatherProfile::clear(), 1.2);
        } else {
            config.queue_transition(WeatherProfile::rain(), 1.2);
        }
        cycle.raining = !cycle.raining;
    }
}
