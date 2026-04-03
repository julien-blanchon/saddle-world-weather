use std::time::Duration;

use bevy::{
    asset::AssetPlugin, ecs::schedule::ScheduleLabel, pbr::MeshMaterial3d, prelude::*,
    time::TimeUpdateStrategy, transform::TransformPlugin,
};

use crate::{
    WeatherCamera, WeatherConfig, WeatherPlugin, WeatherProfile, WeatherScreenFxMode,
    WeatherSurface, WeatherSurfaceState,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn init_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins(WeatherPlugin::new(Activate, Deactivate, Tick));
    app
}

fn advance_tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(16));
        app.world_mut().run_schedule(Tick);
    }
}

#[test]
fn weather_surface_accumulates_wetness_and_modulates_material() {
    let mut app = init_app();
    let base_material = StandardMaterial {
        base_color: Color::srgb(0.60, 0.54, 0.46),
        perceptual_roughness: 0.86,
        reflectance: 0.06,
        ..default()
    };
    let base_color = base_material.base_color;
    let material_handle = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(base_material);

    let surface = app
        .world_mut()
        .spawn((
            Name::new("Wet Test Surface"),
            WeatherSurface::default(),
            MeshMaterial3d(material_handle),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::storm());
    advance_tick(&mut app, 90);

    let state = app
        .world()
        .get::<WeatherSurfaceState>(surface)
        .expect("weather surface state should be inserted");
    assert!(state.wetness > 0.4);
    assert!(state.puddle_coverage > 0.05);

    let material_handle = app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(surface)
        .expect("surface should have a material handle")
        .0
        .clone();
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let material = materials
        .get(&material_handle)
        .expect("unique surface material should exist");
    assert!(material.reflectance > 0.06);
    assert!(material.perceptual_roughness < 0.86);
    assert!(
        material.base_color.to_linear().red < base_color.to_linear().red,
        "wet surfaces should darken the original material"
    );
}

#[test]
fn state_only_screen_fx_mode_keeps_camera_state_without_overlay_entities() {
    let mut app = init_app();
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .screen_fx_mode = WeatherScreenFxMode::StateOnly;

    let camera = app
        .world_mut()
        .spawn((
            Name::new("Weather Camera"),
            Camera3d::default(),
            WeatherCamera::default(),
            Transform::from_xyz(0.0, 2.0, 5.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 2.0, 5.0)),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::storm());
    advance_tick(&mut app, 4);

    let state = app
        .world()
        .get::<crate::WeatherCameraState>(camera)
        .expect("camera state should still be published");
    assert!(state.screen_fx_factor > 0.05);

    let overlay_count = {
        let mut query = app
            .world_mut()
            .query::<&crate::visuals::WeatherScreenOverlay>();
        query.iter(app.world()).count()
    };
    assert_eq!(overlay_count, 0);
}

#[test]
fn deactivating_weather_restores_surface_material_baseline() {
    let mut app = init_app();
    let material_handle = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.52, 0.48, 0.40),
            perceptual_roughness: 0.90,
            reflectance: 0.08,
            ..default()
        });

    let surface = app
        .world_mut()
        .spawn((
            Name::new("Reset Surface"),
            WeatherSurface::default(),
            MeshMaterial3d(material_handle),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::storm());
    advance_tick(&mut app, 60);
    app.world_mut().run_schedule(Deactivate);

    assert!(app.world().get::<WeatherSurfaceState>(surface).is_none());
    assert!(
        app.world()
            .get::<crate::surfaces::WeatherSurfaceMaterialBinding>(surface)
            .is_none()
    );

    let material_handle = app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(surface)
        .expect("surface should still keep its unique material")
        .0
        .clone();
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let material = materials
        .get(&material_handle)
        .expect("reset material should exist");
    assert!((material.perceptual_roughness - 0.90).abs() < 0.001);
    assert!((material.reflectance - 0.08).abs() < 0.001);
}
