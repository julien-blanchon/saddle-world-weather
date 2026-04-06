use std::time::Duration;

use bevy::{
    asset::AssetPlugin,
    ecs::{message::Messages, schedule::ScheduleLabel},
    prelude::*,
    time::TimeUpdateStrategy,
    transform::TransformPlugin,
};

use crate::{
    LightningFlashEmitted, WeatherCamera, WeatherCameraState, WeatherCameraVisualState,
    WeatherConfig, WeatherDiagnostics, WeatherOcclusionVolume, WeatherPlugin, WeatherProfile,
    WeatherProfileChanged, WeatherRuntime, WeatherSystems, WeatherTransitionFinished,
    WeatherTransitionStarted, WeatherVisualDiagnostics, WeatherVisualsPlugin, WeatherVolumeShape,
    WeatherZone,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

#[derive(Resource, Default)]
struct MessageCounts {
    started: u32,
    finished: u32,
    changed: u32,
    lightning: u32,
}

fn count_messages(
    mut counts: ResMut<MessageCounts>,
    mut started: MessageReader<WeatherTransitionStarted>,
    mut finished: MessageReader<WeatherTransitionFinished>,
    mut changed: MessageReader<WeatherProfileChanged>,
    mut lightning: MessageReader<LightningFlashEmitted>,
) {
    counts.started += started.read().count() as u32;
    counts.finished += finished.read().count() as u32;
    counts.changed += changed.read().count() as u32;
    counts.lightning += lightning.read().count() as u32;
}

fn init_scheduled_app() -> App {
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
    app.add_plugins((
        WeatherPlugin::new(Activate, Deactivate, Tick),
        WeatherVisualsPlugin::new(Activate, Deactivate, Tick),
    ));
    app.insert_resource(MessageCounts::default());
    app.add_systems(Tick, count_messages.after(WeatherSystems::EmitMessages));
    app
}

fn init_always_on_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.add_plugins((WeatherPlugin::default(), WeatherVisualsPlugin::default()));
    app.insert_resource(MessageCounts::default());
    app.add_systems(Update, count_messages.after(WeatherSystems::EmitMessages));
    app
}

fn spawn_camera(
    app: &mut App,
    name: &str,
    translation: Vec3,
    weather_camera: WeatherCamera,
) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_string()),
            Camera3d::default(),
            weather_camera,
            Transform::from_translation(translation),
            GlobalTransform::from_translation(translation),
        ))
        .id()
}

fn move_entity(app: &mut App, entity: Entity, translation: Vec3) {
    if let Some(mut transform) = app.world_mut().get_mut::<Transform>(entity) {
        transform.translation = translation;
    }
    if let Some(mut global) = app.world_mut().get_mut::<GlobalTransform>(entity) {
        *global = GlobalTransform::from_translation(translation);
    }
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
fn plugin_builds_and_initializes_resources() {
    let mut app = init_always_on_app();
    app.update();

    assert!(app.world().contains_resource::<WeatherConfig>());
    assert!(app.world().contains_resource::<WeatherRuntime>());
    assert!(app.world().contains_resource::<WeatherDiagnostics>());
    assert!(app.world().contains_resource::<WeatherVisualDiagnostics>());
    assert!(
        app.world()
            .contains_resource::<Messages<WeatherTransitionStarted>>()
    );
}

#[test]
fn transition_messages_fire_once_per_transition() {
    let mut app = init_scheduled_app();
    spawn_camera(
        &mut app,
        "Test Camera",
        Vec3::new(0.0, 2.0, 6.0),
        WeatherCamera::default(),
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_transition(WeatherProfile::rain(), 0.10);
    advance_tick(&mut app, 16);

    let counts = app.world().resource::<MessageCounts>();
    assert_eq!(counts.started, 1);
    assert_eq!(counts.finished, 1);
    assert_eq!(counts.changed, 1);

    let diagnostics = app.world().resource::<WeatherDiagnostics>();
    assert_eq!(diagnostics.transition_started_count, 1);
    assert_eq!(diagnostics.transition_finished_count, 1);
    assert_eq!(diagnostics.profile_changed_count, 1);
}

#[test]
fn lightning_messages_emit_when_storm_flash_becomes_active() {
    let mut app = init_scheduled_app();
    spawn_camera(
        &mut app,
        "Lightning Camera",
        Vec3::new(0.0, 2.0, 6.0),
        WeatherCamera::default(),
    );

    let mut storm = WeatherProfile::storm();
    storm.storm.lightning_frequency_hz = 4.0;
    storm.storm.lightning_duration_secs = 0.20;
    storm.storm.lightning_brightness = 1.0;

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(storm);
    advance_tick(&mut app, 240);

    let counts = app.world().resource::<MessageCounts>();
    assert!(counts.lightning >= 1);

    let diagnostics = app.world().resource::<WeatherDiagnostics>();
    assert!(diagnostics.lightning_flash_count >= 1);
    assert!(diagnostics.last_lightning_flash_id.is_some());
}

#[test]
fn emitter_lifecycle_spawns_and_cleans_up() {
    let mut app = init_scheduled_app();
    let camera = spawn_camera(
        &mut app,
        "Emitter Camera",
        Vec3::new(0.0, 2.0, 6.0),
        WeatherCamera::default(),
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::rain());
    advance_tick(&mut app, 2);

    let emitter_count = {
        let mut query = app
            .world_mut()
            .query::<&crate::visuals::WeatherEmitterRoot>();
        query.iter(app.world()).count()
    };
    assert!(emitter_count >= 1);
    assert!(
        app.world()
            .get::<WeatherCameraVisualState>(camera)
            .is_some_and(|state| state.active_particles > 0)
    );

    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::clear());
    advance_tick(&mut app, 2);

    let emitter_count = {
        let mut query = app
            .world_mut()
            .query::<&crate::visuals::WeatherEmitterRoot>();
        query.iter(app.world()).count()
    };
    assert_eq!(emitter_count, 0);
    assert_eq!(
        app.world()
            .get::<WeatherCameraVisualState>(camera)
            .expect("camera visual state should exist")
            .active_particles,
        0
    );
}

#[test]
fn deactivate_schedule_cleans_runtime_entities_and_camera_state() {
    let mut app = init_scheduled_app();
    let camera = spawn_camera(
        &mut app,
        "Cleanup Camera",
        Vec3::new(0.0, 2.0, 6.0),
        WeatherCamera::default(),
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::rain());
    advance_tick(&mut app, 2);

    app.world_mut().run_schedule(Deactivate);

    let emitter_count = {
        let mut query = app
            .world_mut()
            .query::<&crate::visuals::WeatherEmitterRoot>();
        query.iter(app.world()).count()
    };
    assert_eq!(emitter_count, 0);
    assert!(app.world().get::<WeatherCameraState>(camera).is_none());
    assert!(
        app.world()
            .get::<WeatherCameraVisualState>(camera)
            .is_none()
    );
}

#[test]
fn per_camera_screen_fx_opt_in_is_respected() {
    let mut app = init_scheduled_app();
    spawn_camera(
        &mut app,
        "Gameplay Camera",
        Vec3::new(-2.0, 2.0, 6.0),
        WeatherCamera {
            receive_screen_fx: false,
            ..default()
        },
    );
    let cinematic = spawn_camera(
        &mut app,
        "Cinematic Camera",
        Vec3::new(2.0, 2.0, 6.0),
        WeatherCamera::default(),
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::storm());
    advance_tick(&mut app, 2);

    let overlays: Vec<Entity> = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &crate::visuals::WeatherScreenOverlay)>();
        query
            .iter(app.world())
            .filter_map(|(entity, overlay)| (overlay.camera == cinematic).then_some(entity))
            .collect()
    };

    assert_eq!(overlays.len(), 1);
}

#[test]
fn moving_into_zone_updates_local_weather_state() {
    let mut app = init_scheduled_app();
    let camera = spawn_camera(
        &mut app,
        "Zone Camera",
        Vec3::new(20.0, 2.0, 0.0),
        WeatherCamera::default(),
    );
    app.world_mut().spawn((
        Name::new("Fog Pocket"),
        WeatherZone {
            label: Some("Fog Pocket".into()),
            profile: WeatherProfile::foggy(),
            shape: WeatherVolumeShape::Sphere { radius: 4.0 },
            blend_distance: 2.0,
            priority: 2,
            weight: 1.0,
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    app.world_mut().run_schedule(Activate);
    advance_tick(&mut app, 2);
    let outside_visibility = app
        .world()
        .get::<WeatherCameraState>(camera)
        .expect("camera state should exist")
        .visibility_distance;

    move_entity(&mut app, camera, Vec3::ZERO);
    advance_tick(&mut app, 2);
    let state = app
        .world()
        .get::<WeatherCameraState>(camera)
        .expect("camera state should exist");

    assert_eq!(state.base_profile_label.as_deref(), Some("Clear"));
    assert_eq!(state.resolved_profile_label.as_deref(), Some("Foggy"));
    assert_eq!(state.zone_label.as_deref(), Some("Fog Pocket"));
    assert!(state.visibility_distance < outside_visibility);
}

#[test]
fn occlusion_volume_suppresses_precipitation() {
    let mut app = init_scheduled_app();
    let camera = spawn_camera(
        &mut app,
        "Shelter Camera",
        Vec3::new(12.0, 2.0, 0.0),
        WeatherCamera::default(),
    );
    app.world_mut().spawn((
        Name::new("Shelter"),
        WeatherOcclusionVolume {
            label: Some("Shelter".into()),
            shape: WeatherVolumeShape::Box {
                half_extents: Vec3::new(3.0, 2.0, 3.0),
            },
            precipitation_multiplier: 0.0,
            screen_fx_multiplier: 0.0,
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    app.world_mut().run_schedule(Activate);
    app.world_mut()
        .resource_mut::<WeatherConfig>()
        .queue_immediate(WeatherProfile::rain());
    advance_tick(&mut app, 2);
    let outside_factor = app
        .world()
        .get::<WeatherCameraState>(camera)
        .expect("camera state should exist")
        .precipitation_factor;

    move_entity(&mut app, camera, Vec3::ZERO);
    advance_tick(&mut app, 2);
    let inside_factor = app
        .world()
        .get::<WeatherCameraState>(camera)
        .expect("camera state should exist")
        .precipitation_factor;

    assert!(inside_factor < outside_factor * 0.2);
}
