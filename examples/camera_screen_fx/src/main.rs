use saddle_world_weather_example_support as support;

use bevy::{camera::Viewport, prelude::*};
use saddle_world_weather::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherQuality};

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

#[derive(Component)]
struct SplitOverlay;

fn main() {
    let config = WeatherConfig {
        quality: WeatherQuality::High,
        initial_profile: WeatherProfile::storm(),
        seed: 99,
        ..default()
    };
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.56, 0.63, 0.72)));
    app.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.78, 0.82, 0.88),
        brightness: 500.0,
        ..default()
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "weather camera_screen_fx".into(),
            resolution: (1440, 810).into(),
            ..default()
        }),
        ..default()
    }));
    support::install_demo_pane(&mut app, &config);
    app.add_plugins(WeatherPlugin::default().with_config(config));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (support::animate_props, update_split_overlay));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    support::spawn_showcase_environment(&mut commands, meshes.as_mut(), materials.as_mut());

    commands.spawn((
        Name::new("Gameplay Camera"),
        Camera3d::default(),
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(720, 810),
                ..default()
            }),
            ..default()
        },
        saddle_world_weather::WeatherCamera {
            receive_screen_fx: false,
            ..default()
        },
        LeftCamera,
        Transform::from_xyz(-13.0, 7.2, -15.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Cinematic Camera"),
        Camera3d::default(),
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(720, 0),
                physical_size: UVec2::new(720, 810),
                ..default()
            }),
            ..default()
        },
        saddle_world_weather::WeatherCamera {
            receive_screen_fx: true,
            ..default()
        },
        RightCamera,
        Transform::from_xyz(-13.0, 7.2, -15.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Split Overlay"),
        SplitOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(720.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.72)),
        Text::default(),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn update_split_overlay(
    left: Query<&saddle_world_weather::WeatherCameraState, With<LeftCamera>>,
    right: Query<&saddle_world_weather::WeatherCameraState, With<RightCamera>>,
    mut overlay: Query<&mut Text, With<SplitOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    let left_line = left
        .single()
        .map(|state| {
            format!(
                "Left viewport: gameplay clarity, screen_fx={:>4.2}, particles={}",
                state.screen_fx_factor, state.active_particles
            )
        })
        .unwrap_or_else(|_| "Left viewport unavailable".into());
    let right_line = right
        .single()
        .map(|state| {
            format!(
                "Right viewport: cinematic, screen_fx={:>4.2}, particles={}",
                state.screen_fx_factor, state.active_particles
            )
        })
        .unwrap_or_else(|_| "Right viewport unavailable".into());

    text.0 = format!("Weather split-screen camera comparison\n{left_line}\n{right_line}");
}
