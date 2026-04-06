use std::time::Duration;

use bevy::{
    asset::AssetPlugin, ecs::schedule::ScheduleLabel, prelude::*, time::TimeUpdateStrategy,
    transform::TransformPlugin,
};

use crate::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherSurface, WeatherSurfaceState};

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
fn weather_surface_accumulates_wetness_state() {
    let mut app = init_app();
    let surface = app
        .world_mut()
        .spawn((
            Name::new("Wet Test Surface"),
            WeatherSurface::default(),
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
}

#[test]
fn disabled_surface_removes_runtime_surface_state() {
    let mut app = init_app();
    let surface = app
        .world_mut()
        .spawn((
            Name::new("Disabled Surface"),
            WeatherSurface::default(),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::storm());
    advance_tick(&mut app, 30);

    app.world_mut().entity_mut(surface).insert(WeatherSurface {
        enabled: false,
        ..default()
    });
    advance_tick(&mut app, 1);

    assert!(app.world().get::<WeatherSurfaceState>(surface).is_none());
}

#[test]
fn deactivating_weather_removes_surface_state() {
    let mut app = init_app();
    let surface = app
        .world_mut()
        .spawn((
            Name::new("Reset Surface"),
            WeatherSurface::default(),
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
}
