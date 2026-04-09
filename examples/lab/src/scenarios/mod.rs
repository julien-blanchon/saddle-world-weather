mod support;

use crate::support as example_support;
use bevy::camera::Viewport;
use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_world_weather::{WeatherConfig, WeatherProfile, WeatherQuality, WeatherVisualsConfig};

#[derive(Component)]
struct ScreenFxRightCamera;

fn log_runtime_snapshot(label: &'static str) -> Action {
    Action::Custom(Box::new(move |world| {
        let runtime = support::runtime(world);
        let diagnostics = support::diagnostics(world);
        let visuals = support::visual_diagnostics(world);
        info!(
            "[weather_e2e:{label}] profile={:?} precip={:?} transition_active={} visibility={:.1} rain={:.2} storm={:.2} snow={:.2} particles={} flashes={}",
            runtime.active_profile.label.as_deref(),
            diagnostics.current_precipitation_kind,
            runtime.transition.active,
            runtime.visibility.visibility_distance,
            runtime.factors.rain_factor,
            runtime.factors.storm_factor,
            runtime.factors.snow_factor,
            visuals.precipitation_particles_estimate,
            diagnostics.lightning_flash_count,
        );
    }))
}

fn camera_state_for<T: Component>(
    world: &mut World,
) -> Option<saddle_world_weather::WeatherCameraState> {
    let mut query = world.query_filtered::<&saddle_world_weather::WeatherCameraState, With<T>>();
    query.iter(world).next().cloned()
}

fn camera_visual_state_for<T: Component>(
    world: &mut World,
) -> Option<saddle_world_weather::WeatherCameraVisualState> {
    let mut query =
        world.query_filtered::<&saddle_world_weather::WeatherCameraVisualState, With<T>>();
    query.iter(world).next().cloned()
}

fn windy_snow_profile() -> WeatherProfile {
    let mut profile = WeatherProfile::snow();
    profile.label = Some("Windy Snow".into());
    profile.precipitation.wind_influence = 1.45;
    profile.precipitation.fall_speed = 5.5;
    profile.precipitation.near_radius = 14.0;
    profile.precipitation.density = 0.82;
    profile.fog.visibility_distance = 72.0;
    profile.fog.density = 0.24;
    profile.wind.direction = Vec3::new(-1.0, 0.0, 0.35);
    profile.wind.base_speed = 12.0;
    profile.wind.gust_amplitude = 0.82;
    profile.wind.gust_frequency_hz = 0.55;
    profile.wind.sway = 0.95;
    profile
}

fn reset_message_log() -> Action {
    Action::Custom(Box::new(|world| {
        *world.resource_mut::<crate::WeatherMessageLog>() = crate::WeatherMessageLog::default();
    }))
}

fn queue_immediate_profile(profile: WeatherProfile) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<WeatherConfig>().queue_immediate(profile);
    }))
}

fn queue_transition_profile(profile: WeatherProfile, seconds: f32) -> Action {
    Action::Custom(Box::new(move |world| {
        world.resource_mut::<WeatherConfig>()
            .queue_transition(profile, seconds);
    }))
}

fn wait_for_profile(label: &'static str, expected: &'static str) -> Action {
    Action::WaitUntil {
        label: label.into(),
        condition: Box::new(move |world| {
            let runtime = support::runtime(world);
            runtime.active_profile.label.as_deref() == Some(expected) && !runtime.transition.active
        }),
        max_frames: 180,
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "weather_smoke",
        "weather_transition_gallery",
        "weather_windy_snow",
        "weather_localized_zones",
        "weather_camera_screen_fx",
        "weather_shelter_occlusion",
        "weather_storm_flash",
        "weather_quality_compare",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "weather_smoke" => Some(weather_smoke()),
        "weather_transition_gallery" => Some(weather_transition_gallery()),
        "weather_windy_snow" => Some(weather_windy_snow()),
        "weather_localized_zones" => Some(weather_localized_zones()),
        "weather_camera_screen_fx" => Some(weather_camera_screen_fx()),
        "weather_shelter_occlusion" => Some(weather_shelter_occlusion()),
        "weather_storm_flash" => Some(weather_storm_flash()),
        "weather_quality_compare" => Some(weather_quality_compare()),
        _ => None,
    }
}

fn weather_smoke() -> Scenario {
    Scenario::builder("weather_smoke")
        .description("Boot the lab, verify core resources and named weather entities exist, then capture the initial clear-state frame.")
        .then(Action::WaitFrames(60))
        .then(assertions::custom(
            "core weather resources exist",
            Box::new(|world: &World| {
                world.contains_resource::<saddle_world_weather::WeatherRuntime>()
                    && world.contains_resource::<saddle_world_weather::WeatherDiagnostics>()
            }),
        ))
        .then(Action::Custom(Box::new(|world| {
            assert!(support::entity_by_name::<saddle_world_weather::WeatherZone>(
                world,
                "Fog Pocket",
            )
            .is_some());
            assert!(support::entity_by_name::<saddle_world_weather::WeatherZone>(
                world,
                "Storm Cell",
            )
            .is_some());
            assert!(support::entity_by_name::<saddle_world_weather::WeatherOcclusionVolume>(
                world,
                "Shelter Occlusion",
            )
            .is_some());
            let overlay = support::overlay_text(world).expect("overlay text should exist");
            assert!(overlay.contains("Weather Lab"));
            assert!(overlay.contains("Diagnostics"));
            let camera_state = support::camera_state(world);
            assert!(camera_state.visibility_distance > 150.0);
        })))
        .then(assertions::custom(
            "smoke boots into the authored clear high-quality state",
            Box::new(|world: &World| {
                let runtime = support::runtime(world);
                let diagnostics = support::diagnostics(world);
                let visual_diagnostics = support::visual_diagnostics(world);
                runtime.active_profile.label.as_deref() == Some("Clear")
                    && visual_diagnostics.quality == WeatherQuality::High
                    && diagnostics.current_precipitation_kind
                        == saddle_world_weather::PrecipitationKind::None
                    && diagnostics.transition_started_count == 0
            }),
        ))
        .then(log_runtime_snapshot("smoke"))
        .then(assertions::log_summary("weather_smoke summary"))
        .then(Action::Screenshot("weather_smoke".into()))
        .build()
}

fn weather_transition_gallery() -> Scenario {
    Scenario::builder("weather_transition_gallery")
        .description("Transition through foggy, rain, storm, and snow profiles with assertions and screenshots at each stable resolved state.")
        .then(reset_message_log())
        .then(Action::WaitFrames(5))
        .then(queue_immediate_profile(WeatherProfile::clear()))
        .then(queue_transition_profile(WeatherProfile::foggy(), 1.0))
        .then(wait_for_profile("foggy", "Foggy"))
        .then(log_runtime_snapshot("foggy"))
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            assert!(runtime.visibility.visibility_distance < 60.0);
            assert!(runtime.factors.fog_factor > 0.35);
        })))
        .then(Action::Screenshot("foggy".into()))
        .then(queue_transition_profile(WeatherProfile::rain(), 1.0))
        .then(wait_for_profile("rain", "Rain"))
        .then(log_runtime_snapshot("rain"))
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            let visual_diagnostics = support::visual_diagnostics(world);
            assert!(runtime.factors.rain_factor > 0.5);
            assert!(visual_diagnostics.precipitation_particles_estimate > 0);
        })))
        .then(Action::Screenshot("rain".into()))
        .then(queue_transition_profile(WeatherProfile::storm(), 1.0))
        .then(wait_for_profile("storm", "Storm"))
        .then(log_runtime_snapshot("storm"))
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            assert!(runtime.factors.storm_factor > 0.8);
            assert!(runtime.wind.base_speed >= 12.0);
            assert!(runtime.wind.vector.length() > 1.0);
        })))
        .then(Action::Screenshot("storm".into()))
        .then(queue_transition_profile(WeatherProfile::snow(), 1.0))
        .then(wait_for_profile("snow", "Snow"))
        .then(log_runtime_snapshot("snow"))
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            let diagnostics = support::diagnostics(world);
            assert!(runtime.factors.snow_factor > 0.45);
            assert!(runtime.precipitation.fall_speed < 8.0);
            assert!(diagnostics.transition_started_count >= 4);
            assert!(diagnostics.transition_finished_count >= 4);
            assert!(diagnostics.profile_changed_count >= 4);
        })))
        .then(assertions::custom(
            "transition gallery completed through snow with multiple profile changes",
            Box::new(|world: &World| {
                let runtime = support::runtime(world);
                let diagnostics = support::diagnostics(world);
                runtime.active_profile.label.as_deref() == Some("Snow")
                    && !runtime.transition.active
                    && diagnostics.transition_started_count >= 4
                    && diagnostics.profile_changed_count >= 4
            }),
        ))
        .then(assertions::log_summary("weather_transition_gallery summary"))
        .then(Action::Screenshot("snow".into()))
        .build()
}

fn weather_windy_snow() -> Scenario {
    Scenario::builder("weather_windy_snow")
        .description("Apply a gust-heavy snow profile, verify the wind and snowfall factors increase, and capture the windy_snow showcase state.")
        .then(queue_immediate_profile(windy_snow_profile()))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<WeatherVisualsConfig>().screen_fx.snow_intensity = 0.32;
            world.resource_mut::<WeatherVisualsConfig>().screen_fx.frost_intensity = 0.55;
        })))
        .then(wait_for_profile("windy snow resolved", "Windy Snow"))
        .then(log_runtime_snapshot("windy_snow"))
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            let visuals = support::visual_diagnostics(world);
            let camera = support::camera_state(world);
            assert_eq!(runtime.active_profile.label.as_deref(), Some("Windy Snow"));
            assert!(runtime.factors.snow_factor > 0.65);
            assert!(runtime.wind.base_speed >= 12.0);
            assert!(runtime.wind.vector.length() > 1.0);
            assert!(runtime.precipitation.fall_speed < 6.0);
            assert!(visuals.precipitation_particles_estimate >= 120);
            assert!(camera.precipitation_factor > 0.25);
        })))
        .then(assertions::custom(
            "windy snow increases snow and wind intensity",
            Box::new(|world: &World| {
                let runtime = support::runtime(world);
                let visuals = support::visual_diagnostics(world);
                runtime.active_profile.label.as_deref() == Some("Windy Snow")
                    && runtime.factors.snow_factor > 0.65
                    && runtime.wind.base_speed >= 12.0
                    && visuals.precipitation_particles_estimate >= 120
            }),
        ))
        .then(assertions::log_summary("weather_windy_snow summary"))
        .then(Action::Screenshot("windy_snow".into()))
        .build()
}

fn weather_localized_zones() -> Scenario {
    Scenario::builder("weather_localized_zones")
        .description("Move the weather camera through the fog and storm pockets and verify the resolved zone, profile, and visibility change with position.")
        .then(Action::WaitFrames(45))
        .then(Action::Custom(Box::new(|world| {
            support::set_primary_camera(
                world,
                Vec3::new(-18.0, 2.8, -16.0),
                Vec3::new(-18.0, 2.0, 0.0),
            );
        })))
        .then(Action::WaitFrames(20))
        .then(log_runtime_snapshot("fog_pocket"))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            assert_eq!(camera.zone_label.as_deref(), Some("Fog Pocket"));
            assert_eq!(camera.resolved_profile_label.as_deref(), Some("Foggy"));
            assert!(camera.visibility_distance < 80.0);
        })))
        .then(Action::Screenshot("fog_pocket".into()))
        .then(Action::Custom(Box::new(|world| {
            support::set_primary_camera(
                world,
                Vec3::new(18.0, 2.8, -16.0),
                Vec3::new(18.0, 2.0, 0.0),
            );
        })))
        .then(Action::WaitFrames(20))
        .then(log_runtime_snapshot("storm_cell"))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            let visuals = support::camera_visual_state(world)
                .expect("camera visual state should exist in the storm cell");
            assert_eq!(camera.zone_label.as_deref(), Some("Storm Cell"));
            assert_eq!(camera.resolved_profile_label.as_deref(), Some("Storm"));
            assert!(camera.precipitation_factor > 0.4);
            assert!(visuals.screen.overlay_intensity > 0.08);
        })))
        .then(Action::Screenshot("storm_cell".into()))
        .then(assertions::log_summary("weather_localized_zones summary"))
        .build()
}

fn weather_camera_screen_fx() -> Scenario {
    Scenario::builder("weather_camera_screen_fx")
        .description("Split the showcase into gameplay and cinematic viewports, disable screen-space effects on the left camera, and verify only the right camera receives the full storm overlay.")
        .then(queue_immediate_profile(WeatherProfile::storm()))
        .then(Action::Custom(Box::new(|world| {
            let Ok((mut camera, mut weather_camera, mut transform)) = world
                .query_filtered::<
                    (&mut Camera, &mut saddle_world_weather::WeatherCamera, &mut Transform),
                    With<example_support::PrimaryShowcaseCamera>,
                >()
                .single_mut(world)
            else {
                panic!("primary weather camera should exist");
            };
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(720, 810),
                ..default()
            });
            weather_camera.receive_screen_fx = false;
            *transform = Transform::from_xyz(-13.0, 7.2, -15.0)
                .looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y);
        })))
        .then(Action::Custom(Box::new(|world| {
            let mut commands = world.commands();
            let right = example_support::spawn_weather_camera(
                &mut commands,
                "Screen Fx Right Camera",
                Transform::from_xyz(-13.0, 7.2, -15.0).looking_at(Vec3::new(0.0, 1.8, 0.0), Vec3::Y),
                saddle_world_weather::WeatherCamera {
                    receive_screen_fx: true,
                    ..default()
                },
            );
            world.entity_mut(right).insert((
                Camera {
                    viewport: Some(Viewport {
                        physical_position: UVec2::new(720, 0),
                        physical_size: UVec2::new(720, 810),
                        ..default()
                    }),
                    ..default()
                },
                ScreenFxRightCamera,
            ));
        })))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let left_state = camera_state_for::<example_support::PrimaryShowcaseCamera>(world)
                .expect("left comparison camera should have a state");
            let left_visual = camera_visual_state_for::<example_support::PrimaryShowcaseCamera>(world)
                .expect("left comparison camera should have visuals");
            let right_state = camera_state_for::<ScreenFxRightCamera>(world)
                .expect("right comparison camera should have a state");
            let right_visual = camera_visual_state_for::<ScreenFxRightCamera>(world)
                .expect("right comparison camera should have visuals");

            assert!(left_state.precipitation_factor > 0.2);
            assert!(right_state.precipitation_factor > 0.2);
            assert!(left_visual.screen.overlay_intensity < 0.02);
            assert!(right_visual.screen.overlay_intensity > 0.08);
            assert!(right_visual.screen.overlay_intensity > left_visual.screen.overlay_intensity);
        })))
        .then(log_runtime_snapshot("camera_screen_fx"))
        .then(assertions::log_summary("weather_camera_screen_fx summary"))
        .then(Action::Screenshot("camera_screen_fx".into()))
        .build()
}

fn weather_shelter_occlusion() -> Scenario {
    Scenario::builder("weather_shelter_occlusion")
        .description("Move the camera from open rain into the shelter and verify precipitation plus screen-space cues are strongly suppressed.")
        .then(queue_immediate_profile(WeatherProfile::storm()))
        .then(Action::Custom(Box::new(|world| {
            support::set_primary_camera(world, Vec3::new(0.0, 2.8, -16.0), Vec3::new(0.0, 2.0, 0.0));
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            let visual_state = support::camera_visual_state(world)
                .expect("camera visual state should exist outside the shelter");
            assert!(camera.precipitation_factor > 0.2);
            assert!(visual_state.screen.overlay_intensity > 0.08);
        })))
        .then(log_runtime_snapshot("shelter_open"))
        .then(Action::Screenshot("shelter_open".into()))
        .then(Action::Custom(Box::new(|world| {
            support::set_primary_camera(world, Vec3::new(0.0, 2.8, 0.0), Vec3::new(0.0, 2.0, 0.0));
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            let visual_state = support::camera_visual_state(world)
                .expect("camera visual state should exist under the shelter");
            assert!(camera.occlusion_factor < 0.2);
            assert!(camera.precipitation_factor < 0.12);
            assert!(visual_state.screen.overlay_intensity < 0.12);
        })))
        .then(log_runtime_snapshot("shelter_under_roof"))
        .then(assertions::log_summary("weather_shelter_occlusion summary"))
        .then(Action::Screenshot("shelter_under_roof".into()))
        .build()
}

fn weather_storm_flash() -> Scenario {
    Scenario::builder("weather_storm_flash")
        .description("Run a deterministic storm until a lightning flash is active, then assert both the flash state and emitted message count before capturing the frame.")
        .then(reset_message_log())
        .then(queue_immediate_profile(WeatherProfile::storm()))
        .then(Action::WaitUntil {
            label: "storm flash".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.storm.lightning_active && runtime.storm.lightning_flash_intensity > 0.05
            }),
            max_frames: 720,
        })
        .then(Action::WaitFrames(2))
        .then(log_runtime_snapshot("storm_flash"))
        .then(Action::Custom(Box::new(|world| {
            let log = support::message_log(world);
            let diagnostics = support::diagnostics(world);
            let camera = support::camera_state(world);
            assert!(log.lightning_flashes >= 1);
            assert!(log.last_flash_id.is_some());
            assert!(diagnostics.last_lightning_flash_id.is_some());
            assert!(diagnostics.lightning_flash_count >= 1);
            assert!(camera.lightning_flash_intensity > 0.05);
        })))
        .then(assertions::custom(
            "storm flash was observed and counted",
            Box::new(|world: &World| {
                let log = support::message_log(world);
                let diagnostics = support::diagnostics(world);
                log.lightning_flashes >= 1
                    && diagnostics.lightning_flash_count >= 1
            }),
        ))
        .then(assertions::log_summary("weather_storm_flash summary"))
        .then(Action::Screenshot("storm_flash".into()))
        .build()
}

fn weather_quality_compare() -> Scenario {
    Scenario::builder("weather_quality_compare")
        .description("Pause on a storm profile, capture low quality particle density, then switch to high quality and assert the particle budget increases.")
        .then(queue_immediate_profile(WeatherProfile::storm()))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<WeatherVisualsConfig>().quality = WeatherQuality::Low;
            *world.resource_mut::<crate::QualitySnapshot>() = crate::QualitySnapshot::default();
        })))
        .then(Action::WaitFrames(12))
        .then(Action::Custom(Box::new(|world| {
            let diagnostics = support::visual_diagnostics(world);
            world.resource_mut::<crate::QualitySnapshot>().low_particles =
                diagnostics.precipitation_particles_estimate;
            assert_eq!(diagnostics.quality, WeatherQuality::Low);
            assert!(diagnostics.precipitation_particles_estimate <= 64);
            assert_eq!(diagnostics.managed_screen_overlays, 0);
        })))
        .then(log_runtime_snapshot("quality_low"))
        .then(Action::Screenshot("quality_low".into()))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<WeatherVisualsConfig>().quality = WeatherQuality::High;
        })))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let diagnostics = support::visual_diagnostics(world);
            world.resource_mut::<crate::QualitySnapshot>().high_particles =
                diagnostics.precipitation_particles_estimate;
            let snapshot = *world.resource::<crate::QualitySnapshot>();
            assert_eq!(diagnostics.quality, WeatherQuality::High);
            assert!(snapshot.high_particles > snapshot.low_particles);
            assert!(diagnostics.precipitation_particles_estimate >= 120);
            assert!(diagnostics.managed_screen_overlays >= 1);
        })))
        .then(log_runtime_snapshot("quality_high"))
        .then(assertions::custom(
            "high quality increases precipitation particle budget",
            Box::new(|world: &World| {
                let snapshot = *world.resource::<crate::QualitySnapshot>();
                let diagnostics = support::visual_diagnostics(world);
                diagnostics.quality == WeatherQuality::High
                    && snapshot.high_particles > snapshot.low_particles
                    && diagnostics.precipitation_particles_estimate >= 120
                    && diagnostics.managed_screen_overlays >= 1
            }),
        ))
        .then(assertions::log_summary("weather_quality_compare summary"))
        .then(Action::Screenshot("quality_high".into()))
        .build()
}
