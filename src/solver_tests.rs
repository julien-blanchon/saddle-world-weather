use super::*;
use crate::{
    FogProfile, PrecipitationKind, PrecipitationProfile, StormProfile, WeatherProfile, WindProfile,
};

#[test]
fn clear_to_rain_transition_reaches_exact_target() {
    let source = WeatherProfile::clear();
    let target = WeatherProfile::rain();

    let halfway = source.blend(&target, 0.5);
    assert!(halfway.precipitation.intensity > 0.2);
    assert!(halfway.fog.visibility_distance < source.fog.visibility_distance);

    let finished = source.blend(&target, 1.0);
    assert_eq!(finished, target.clamped());
}

#[test]
fn same_seed_produces_same_gust_samples() {
    let wind = WindProfile {
        base_speed: 8.0,
        gust_amplitude: 0.6,
        gust_frequency_hz: 0.35,
        ..default()
    };

    let left: Vec<f32> = [0.0, 0.5, 1.0, 3.25, 8.0]
        .into_iter()
        .map(|time| sample_gust(42, time, &wind))
        .collect();
    let right: Vec<f32> = [0.0, 0.5, 1.0, 3.25, 8.0]
        .into_iter()
        .map(|time| sample_gust(42, time, &wind))
        .collect();

    assert_eq!(left, right);
}

#[test]
fn storm_without_precipitation_normalizes_predictably() {
    let profile = WeatherProfile {
        label: Some("Dry Storm".into()),
        precipitation: PrecipitationProfile::none(),
        fog: FogProfile::default(),
        wind: WindProfile::default(),
        storm: StormProfile {
            intensity: 1.0,
            lightning_frequency_hz: 0.3,
            lightning_duration_secs: 0.1,
            lightning_brightness: 1.0,
            wetness_bonus: 0.2,
        },
    }
    .clamped();

    assert_eq!(profile.precipitation.kind, PrecipitationKind::Rain);
    assert!(profile.precipitation.intensity >= 0.2);
}

#[test]
fn zone_priority_beats_lower_priority_when_overlap_exists() {
    let result = resolve_zone_profile(
        &WeatherProfile::clear(),
        &[
            ZoneContribution {
                label: Some("Low".into()),
                priority: 0,
                weight: 1.0,
                influence: 1.0,
                profile: WeatherProfile::rain(),
            },
            ZoneContribution {
                label: Some("High".into()),
                priority: 5,
                weight: 1.0,
                influence: 1.0,
                profile: WeatherProfile::snow(),
            },
        ],
    );

    assert_eq!(result.dominant_label.as_deref(), Some("High"));
    assert_eq!(result.profile.label.as_deref(), Some("Snow"));
}

#[test]
fn visibility_and_wetness_increase_with_rain() {
    let (_, _, clear_visibility, _, clear_factors) =
        resolve_runtime(&WeatherProfile::clear(), 7, 2.0);
    let (_, _, rain_visibility, _, rain_factors) = resolve_runtime(&WeatherProfile::rain(), 7, 2.0);

    assert!(rain_visibility.visibility_distance < clear_visibility.visibility_distance);
    assert!(rain_factors.wetness_factor > clear_factors.wetness_factor);
    assert!(rain_factors.rain_factor > 0.0);
}

#[test]
fn occlusion_uses_the_strongest_suppression() {
    let result = resolve_occlusion(&[
        OcclusionContribution {
            precipitation_multiplier: 0.5,
            screen_fx_multiplier: 0.6,
            influence: 0.5,
        },
        OcclusionContribution {
            precipitation_multiplier: 0.0,
            screen_fx_multiplier: 0.25,
            influence: 1.0,
        },
    ]);

    assert_eq!(result.precipitation_multiplier, 0.0);
    assert!(result.screen_fx_multiplier <= 0.25);
}
