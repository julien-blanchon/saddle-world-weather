mod support;

use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, scenario::Scenario};
use saddle_world_weather::{WeatherConfig, WeatherProfile, WeatherQuality};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "weather_smoke",
        "weather_transition_gallery",
        "weather_shelter_occlusion",
        "weather_storm_flash",
        "weather_quality_compare",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "weather_smoke" => Some(weather_smoke()),
        "weather_transition_gallery" => Some(weather_transition_gallery()),
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
        .then(Action::Custom(Box::new(|world| {
            assert!(world.contains_resource::<saddle_world_weather::WeatherRuntime>());
            assert!(world.contains_resource::<saddle_world_weather::WeatherDiagnostics>());
            assert!(support::entity_by_name::<saddle_world_weather::WeatherZone>(world, "Fog Pocket").is_some());
            assert!(support::entity_by_name::<saddle_world_weather::WeatherZone>(world, "Storm Cell").is_some());
            assert!(
                support::entity_by_name::<saddle_world_weather::WeatherOcclusionVolume>(world, "Shelter Occlusion")
                    .is_some()
            );
            let runtime = support::runtime(world);
            let diagnostics = support::diagnostics(world);
            assert_eq!(runtime.active_profile.label.as_deref(), Some("Clear"));
            assert_eq!(diagnostics.quality, WeatherQuality::High);
            assert_eq!(diagnostics.current_precipitation_kind, saddle_world_weather::PrecipitationKind::None);
            assert_eq!(diagnostics.transition_started_count, 0);
            let overlay = support::overlay_text(world).expect("overlay text should exist");
            assert!(overlay.contains("Weather Lab"));
            assert!(overlay.contains("Diagnostics"));
            let camera_state = support::camera_state(world);
            assert!(camera_state.visibility_distance > 150.0);
        })))
        .then(Action::Screenshot("weather_smoke".into()))
        .build()
}

fn weather_transition_gallery() -> Scenario {
    Scenario::builder("weather_transition_gallery")
        .description("Transition through foggy, rain, storm, and snow profiles with assertions and screenshots at each stable resolved state.")
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<crate::WeatherMessageLog>() = crate::WeatherMessageLog::default();
            world
                .resource_mut::<WeatherConfig>()
                .queue_immediate(WeatherProfile::clear());
        })))
        .then(Action::WaitFrames(5))
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_transition(WeatherProfile::foggy(), 1.0);
        })))
        .then(Action::WaitUntil {
            label: "foggy".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.active_profile.label.as_deref() == Some("Foggy") && !runtime.transition.active
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            assert!(runtime.visibility.visibility_distance < 60.0);
            assert!(runtime.factors.fog_factor > 0.35);
        })))
        .then(Action::Screenshot("foggy".into()))
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_transition(WeatherProfile::rain(), 1.0);
        })))
        .then(Action::WaitUntil {
            label: "rain".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.active_profile.label.as_deref() == Some("Rain") && !runtime.transition.active
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            let diagnostics = support::diagnostics(world);
            assert!(runtime.factors.rain_factor > 0.5);
            assert!(diagnostics.precipitation_particles_estimate > 0);
        })))
        .then(Action::Screenshot("rain".into()))
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_transition(WeatherProfile::storm(), 1.0);
        })))
        .then(Action::WaitUntil {
            label: "storm".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.active_profile.label.as_deref() == Some("Storm") && !runtime.transition.active
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            assert!(runtime.factors.storm_factor > 0.8);
            assert!(runtime.wind.base_speed >= 12.0);
            assert!(runtime.wind.vector.length() > 1.0);
        })))
        .then(Action::Screenshot("storm".into()))
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_transition(WeatherProfile::snow(), 1.0);
        })))
        .then(Action::WaitUntil {
            label: "snow".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.active_profile.label.as_deref() == Some("Snow") && !runtime.transition.active
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world| {
            let runtime = support::runtime(world);
            let diagnostics = support::diagnostics(world);
            assert!(runtime.factors.snow_factor > 0.45);
            assert!(runtime.precipitation.fall_speed < 8.0);
            assert!(diagnostics.transition_started_count >= 4);
            assert!(diagnostics.transition_finished_count >= 4);
            assert!(diagnostics.profile_changed_count >= 4);
        })))
        .then(Action::Screenshot("snow".into()))
        .build()
}

fn weather_shelter_occlusion() -> Scenario {
    Scenario::builder("weather_shelter_occlusion")
        .description("Move the camera from open rain into the shelter and verify precipitation plus screen-space cues are strongly suppressed.")
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_immediate(WeatherProfile::storm());
            support::set_primary_camera(world, Vec3::new(0.0, 2.8, -16.0), Vec3::new(0.0, 2.0, 0.0));
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            assert!(camera.precipitation_factor > 0.2);
            assert!(camera.screen_fx_factor > 0.08);
        })))
        .then(Action::Screenshot("shelter_open".into()))
        .then(Action::Custom(Box::new(|world| {
            support::set_primary_camera(world, Vec3::new(0.0, 2.8, 0.0), Vec3::new(0.0, 2.0, 0.0));
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Custom(Box::new(|world| {
            let camera = support::camera_state(world);
            assert!(camera.occlusion_factor < 0.2);
            assert!(camera.precipitation_factor < 0.12);
            assert!(camera.screen_fx_factor < 0.12);
        })))
        .then(Action::Screenshot("shelter_under_roof".into()))
        .build()
}

fn weather_storm_flash() -> Scenario {
    Scenario::builder("weather_storm_flash")
        .description("Run a deterministic storm until a lightning flash is active, then assert both the flash state and emitted message count before capturing the frame.")
        .then(Action::Custom(Box::new(|world| {
            *world.resource_mut::<crate::WeatherMessageLog>() = crate::WeatherMessageLog::default();
            world
                .resource_mut::<WeatherConfig>()
                .queue_immediate(WeatherProfile::storm());
        })))
        .then(Action::WaitUntil {
            label: "storm flash".into(),
            condition: Box::new(|world| {
                let runtime = support::runtime(world);
                runtime.storm.lightning_active && runtime.storm.lightning_flash_intensity > 0.05
            }),
            max_frames: 720,
        })
        .then(Action::WaitFrames(2))
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
        .then(Action::Screenshot("storm_flash".into()))
        .build()
}

fn weather_quality_compare() -> Scenario {
    Scenario::builder("weather_quality_compare")
        .description("Pause on a storm profile, capture low quality particle density, then switch to high quality and assert the particle budget increases.")
        .then(Action::Custom(Box::new(|world| {
            world
                .resource_mut::<WeatherConfig>()
                .queue_immediate(WeatherProfile::storm());
            world.resource_mut::<WeatherConfig>().quality = WeatherQuality::Low;
            *world.resource_mut::<crate::QualitySnapshot>() = crate::QualitySnapshot::default();
        })))
        .then(Action::WaitFrames(12))
        .then(Action::Custom(Box::new(|world| {
            let diagnostics = support::diagnostics(world);
            world.resource_mut::<crate::QualitySnapshot>().low_particles =
                diagnostics.precipitation_particles_estimate;
            assert_eq!(diagnostics.quality, WeatherQuality::Low);
            assert!(diagnostics.precipitation_particles_estimate <= 64);
            assert_eq!(diagnostics.managed_screen_overlays, 0);
        })))
        .then(Action::Screenshot("quality_low".into()))
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<WeatherConfig>().quality = WeatherQuality::High;
        })))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let diagnostics = support::diagnostics(world);
            world.resource_mut::<crate::QualitySnapshot>().high_particles =
                diagnostics.precipitation_particles_estimate;
            let snapshot = *world.resource::<crate::QualitySnapshot>();
            assert_eq!(diagnostics.quality, WeatherQuality::High);
            assert!(snapshot.high_particles > snapshot.low_particles);
            assert!(diagnostics.precipitation_particles_estimate >= 120);
            assert!(diagnostics.managed_screen_overlays >= 1);
        })))
        .then(Action::Screenshot("quality_high".into()))
        .build()
}
