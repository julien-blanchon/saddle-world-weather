use bevy::prelude::*;

use crate::support::{PrimaryShowcaseCamera, ShowcaseOverlay};

pub(super) fn entity_by_name<T: Component>(world: &mut World, target_name: &str) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, &Name), With<T>>();
    query
        .iter(world)
        .find_map(|(entity, name)| (name.as_str() == target_name).then_some(entity))
}

pub(super) fn overlay_text(world: &mut World) -> Option<String> {
    let mut query = world.query_filtered::<&Text, With<ShowcaseOverlay>>();
    query.iter(world).next().map(|text| text.0.clone())
}

pub(super) fn runtime(world: &World) -> saddle_world_weather::WeatherRuntime {
    world
        .get_resource::<saddle_world_weather::WeatherRuntime>()
        .cloned()
        .expect("WeatherRuntime resource should exist")
}

pub(super) fn diagnostics(world: &World) -> saddle_world_weather::WeatherDiagnostics {
    world
        .get_resource::<saddle_world_weather::WeatherDiagnostics>()
        .cloned()
        .expect("WeatherDiagnostics resource should exist")
}

pub(super) fn visual_diagnostics(world: &World) -> saddle_world_weather::WeatherVisualDiagnostics {
    world
        .get_resource::<saddle_world_weather::WeatherVisualDiagnostics>()
        .cloned()
        .expect("WeatherVisualDiagnostics resource should exist")
}

pub(super) fn camera_state(world: &mut World) -> saddle_world_weather::WeatherCameraState {
    let mut query = world
        .query_filtered::<&saddle_world_weather::WeatherCameraState, With<PrimaryShowcaseCamera>>();
    query
        .iter(world)
        .next()
        .cloned()
        .expect("primary camera state should exist")
}

pub(super) fn camera_visual_state(
    world: &mut World,
) -> Option<saddle_world_weather::WeatherCameraVisualState> {
    let mut query = world.query_filtered::<
        &saddle_world_weather::WeatherCameraVisualState,
        With<PrimaryShowcaseCamera>,
    >();
    query.iter(world).next().cloned()
}

pub(super) fn message_log(world: &World) -> crate::WeatherMessageLog {
    world
        .get_resource::<crate::WeatherMessageLog>()
        .cloned()
        .expect("WeatherMessageLog resource should exist")
}

pub(super) fn set_primary_camera(world: &mut World, translation: Vec3, focus: Vec3) {
    let mut query = world.query_filtered::<&mut Transform, With<PrimaryShowcaseCamera>>();
    let mut transform = query
        .iter_mut(world)
        .next()
        .expect("primary camera transform should exist");
    *transform = Transform::from_translation(translation).looking_at(focus, Vec3::Y);
}
