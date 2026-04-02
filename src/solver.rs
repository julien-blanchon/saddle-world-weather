use bevy::prelude::*;

use crate::{
    FogProfile, PrecipitationKind, ScreenFxProfile, StormProfile, VisibilityClass, WeatherFactors,
    WeatherProfile, WeatherScreenState, WeatherVisibility, WindProfile, WindState,
    profiles::lerp_scalar,
    resources::{PrecipitationState, StormState},
};

#[derive(Debug, Clone, PartialEq)]
pub struct LightningSample {
    pub active: bool,
    pub flash_id: Option<u64>,
    pub intensity: f32,
}

impl Default for LightningSample {
    fn default() -> Self {
        Self {
            active: false,
            flash_id: None,
            intensity: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneContribution {
    pub label: Option<String>,
    pub priority: i32,
    pub weight: f32,
    pub influence: f32,
    pub profile: WeatherProfile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBlendResult {
    pub profile: WeatherProfile,
    pub dominant_label: Option<String>,
    pub active_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OcclusionContribution {
    pub precipitation_multiplier: f32,
    pub screen_fx_multiplier: f32,
    pub influence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OcclusionResult {
    pub precipitation_multiplier: f32,
    pub screen_fx_multiplier: f32,
}

pub fn resolve_runtime(
    profile: &WeatherProfile,
    seed: u64,
    time_secs: f32,
) -> (
    WindState,
    PrecipitationState,
    WeatherVisibility,
    WeatherScreenState,
    StormState,
    WeatherFactors,
) {
    let profile = profile.clone().clamped();
    let wind = resolve_wind(&profile.wind, seed, time_secs);
    let lightning = sample_lightning(seed, time_secs, &profile.storm);
    let precipitation = resolve_precipitation(&profile.precipitation);
    let visibility = resolve_visibility(&profile.fog, &profile.precipitation, &profile.storm);
    let screen = resolve_screen_fx(&profile.screen_fx, &profile.precipitation, &profile.fog);
    let storm = StormState {
        intensity: profile.storm.intensity,
        lightning_active: lightning.active,
        lightning_flash_intensity: lightning.intensity,
        lightning_flash_id: lightning.flash_id,
    };
    let factors = resolve_factors(
        &profile.precipitation,
        &profile.fog,
        &profile.screen_fx,
        &profile.storm,
        &wind,
        lightning.intensity,
    );

    (wind, precipitation, visibility, screen, storm, factors)
}

pub fn sample_gust(seed: u64, time_secs: f32, profile: &WindProfile) -> f32 {
    if profile.gust_amplitude <= 0.0 || profile.gust_frequency_hz <= 0.0 {
        return 0.0;
    }

    let base = hash01(seed, 17, 0);
    let phase_a = std::f32::consts::TAU * hash01(seed, 19, 1);
    let phase_b = std::f32::consts::TAU * hash01(seed, 23, 2);
    let frequency = profile.gust_frequency_hz.max(0.01);
    let a = (time_secs * frequency * std::f32::consts::TAU + phase_a).sin();
    let b = (time_secs * frequency * 0.6 * std::f32::consts::TAU + phase_b).sin();
    let c = (time_secs * frequency * 1.8 * std::f32::consts::TAU + phase_a * 0.5).cos();
    ((a * 0.55 + b * 0.30 + c * 0.15) * 0.5 + 0.5).clamp(0.0, 1.0) * (0.8 + base * 0.2)
}

pub fn sample_lightning(seed: u64, time_secs: f32, profile: &StormProfile) -> LightningSample {
    if profile.intensity <= 0.0
        || profile.lightning_frequency_hz <= 0.0
        || profile.lightning_brightness <= 0.0
    {
        return LightningSample::default();
    }

    let bucket_length = 0.25_f32;
    let bucket = (time_secs / bucket_length).floor() as i64;
    let flash_probability = (profile.lightning_frequency_hz * bucket_length).clamp(0.0, 0.95);
    let chance = hash01(seed, bucket as u64, 7);
    if chance > flash_probability {
        return LightningSample::default();
    }

    let max_offset = (bucket_length - profile.lightning_duration_secs).max(0.0);
    let offset = hash01(seed, bucket as u64, 11) * max_offset;
    let local_time = time_secs - bucket as f32 * bucket_length;
    if local_time < offset || local_time > offset + profile.lightning_duration_secs {
        return LightningSample {
            active: false,
            flash_id: Some(bucket as u64),
            intensity: 0.0,
        };
    }

    let progress = ((local_time - offset) / profile.lightning_duration_secs).clamp(0.0, 1.0);
    let pulse = if progress <= 0.4 {
        progress / 0.4
    } else {
        1.0 - (progress - 0.4) / 0.6
    }
    .clamp(0.0, 1.0);

    LightningSample {
        active: true,
        flash_id: Some(bucket as u64),
        intensity: profile.lightning_brightness * pulse,
    }
}

pub fn resolve_zone_profile(
    base_profile: &WeatherProfile,
    contributions: &[ZoneContribution],
) -> ZoneBlendResult {
    let active: Vec<&ZoneContribution> = contributions
        .iter()
        .filter(|entry| entry.influence > 0.0 && entry.weight > 0.0)
        .collect();
    if active.is_empty() {
        return ZoneBlendResult {
            profile: base_profile.clone(),
            dominant_label: None,
            active_count: 0,
        };
    }

    let highest_priority = active.iter().map(|entry| entry.priority).max().unwrap_or(0);
    let selected: Vec<&ZoneContribution> = active
        .into_iter()
        .filter(|entry| entry.priority == highest_priority)
        .collect();

    let mut total_zone_weight = 0.0;
    let mut dominant = None;
    let mut dominant_strength = -1.0_f32;
    for entry in &selected {
        let strength = entry.influence * entry.weight;
        total_zone_weight += strength;
        if strength > dominant_strength {
            dominant_strength = strength;
            dominant = entry.label.clone();
        }
    }

    let zone_mix = total_zone_weight.clamp(0.0, 1.0);
    let mut blended_zone = selected[0].profile.clone();
    let mut accumulated_weight = selected[0].influence * selected[0].weight;
    for entry in selected.iter().skip(1) {
        let weight = entry.influence * entry.weight;
        let blend = if accumulated_weight + weight <= f32::EPSILON {
            0.0
        } else {
            weight / (accumulated_weight + weight)
        };
        blended_zone = blended_zone.blend(&entry.profile, blend);
        accumulated_weight += weight;
    }

    ZoneBlendResult {
        profile: base_profile.blend(&blended_zone, zone_mix),
        dominant_label: dominant,
        active_count: selected.len(),
    }
}

pub fn resolve_occlusion(contributions: &[OcclusionContribution]) -> OcclusionResult {
    if contributions.is_empty() {
        return OcclusionResult {
            precipitation_multiplier: 1.0,
            screen_fx_multiplier: 1.0,
        };
    }

    let mut precipitation = 1.0_f32;
    let mut screen = 1.0_f32;
    for contribution in contributions {
        let influence = contribution.influence.clamp(0.0, 1.0);
        precipitation = precipitation.min(lerp_scalar(
            1.0,
            contribution.precipitation_multiplier.clamp(0.0, 1.0),
            influence,
        ));
        screen = screen.min(lerp_scalar(
            1.0,
            contribution.screen_fx_multiplier.clamp(0.0, 1.0),
            influence,
        ));
    }

    OcclusionResult {
        precipitation_multiplier: precipitation,
        screen_fx_multiplier: screen,
    }
}

fn resolve_wind(profile: &WindProfile, seed: u64, time_secs: f32) -> WindState {
    let direction = profile.direction.normalize_or_zero();
    let gust_factor = sample_gust(seed, time_secs, profile);
    let gust_scale = 1.0 + profile.gust_amplitude * ((gust_factor - 0.5) * 2.0);
    let speed = (profile.base_speed * gust_scale).max(0.0);
    WindState {
        direction,
        base_speed: profile.base_speed,
        speed,
        gust_factor,
        vector: direction * speed,
    }
}

fn resolve_precipitation(profile: &crate::PrecipitationProfile) -> PrecipitationState {
    PrecipitationState {
        kind: profile.kind.clone(),
        intensity: profile.intensity,
        density: profile.density,
        fall_speed: profile.fall_speed,
        particle_size: profile.particle_size,
        wind_influence: profile.wind_influence,
        near_radius: profile.near_radius,
        near_height: profile.near_height,
        far_density: profile.far_density,
        tint: profile.tint,
    }
}

fn resolve_visibility(
    fog: &FogProfile,
    precipitation: &crate::PrecipitationProfile,
    storm: &StormProfile,
) -> WeatherVisibility {
    let fog_density =
        (fog.density + precipitation.intensity * 0.12 + storm.intensity * 0.10).clamp(0.0, 1.0);
    let visibility_distance = (fog.visibility_distance
        * (1.0 - precipitation.intensity * 0.38)
        * (1.0 - storm.intensity * 0.22))
        .max(5.0);

    let classification = if visibility_distance > 140.0 {
        VisibilityClass::Clear
    } else if visibility_distance > 70.0 {
        VisibilityClass::Hazy
    } else if visibility_distance > 28.0 {
        VisibilityClass::Low
    } else {
        VisibilityClass::Severe
    };

    WeatherVisibility {
        fog_density,
        visibility_distance,
        volumetric_intensity: (fog.volumetric_intensity + precipitation.far_density * 0.3)
            .clamp(0.0, 1.0),
        fog_color: fog.color,
        classification,
    }
}

fn resolve_screen_fx(
    screen_fx: &ScreenFxProfile,
    precipitation: &crate::PrecipitationProfile,
    fog: &FogProfile,
) -> WeatherScreenState {
    let moisture = precipitation.intensity.max(fog.density * 0.45);
    WeatherScreenState {
        overlay_intensity: (screen_fx.intensity * (0.35 + moisture * 0.65)).clamp(0.0, 1.0),
        droplet_intensity: (screen_fx.droplet_intensity * precipitation.intensity).clamp(0.0, 1.0),
        frost_intensity: (screen_fx.frost_intensity * (0.25 + moisture * 0.75)).clamp(0.0, 1.0),
        streak_intensity: (screen_fx.streak_intensity * precipitation.intensity).clamp(0.0, 1.0),
        tint: screen_fx.tint,
    }
}

fn resolve_factors(
    precipitation: &crate::PrecipitationProfile,
    fog: &FogProfile,
    screen_fx: &ScreenFxProfile,
    storm: &StormProfile,
    wind: &WindState,
    lightning_intensity: f32,
) -> WeatherFactors {
    let rain_factor = if matches!(precipitation.kind, PrecipitationKind::Rain) {
        precipitation.intensity
    } else {
        0.0
    };
    let snow_factor = if matches!(precipitation.kind, PrecipitationKind::Snow) {
        precipitation.intensity
    } else {
        0.0
    };
    let fog_factor =
        (fog.density + (1.0 / fog.visibility_distance.max(1.0)) * 40.0).clamp(0.0, 1.0);
    let wind_factor = (wind.speed / 18.0).clamp(0.0, 1.0);
    let storm_factor = storm
        .intensity
        .max(lightning_intensity.clamp(0.0, 1.0))
        .max((precipitation.intensity * 0.8).clamp(0.0, 1.0));
    let wetness_factor =
        (rain_factor * 0.8 + fog_factor * 0.25 + storm.wetness_bonus).clamp(0.0, 1.0);
    let screen_fx_factor = (screen_fx.intensity * 0.4
        + screen_fx.droplet_intensity * 0.3
        + screen_fx.frost_intensity * 0.3)
        .clamp(0.0, 1.0);

    WeatherFactors {
        rain_factor,
        snow_factor,
        fog_factor,
        storm_factor,
        wind_factor,
        wetness_factor,
        screen_fx_factor,
    }
}

pub(crate) fn hash01(seed: u64, index: u64, salt: u64) -> f32 {
    let mut value = seed
        .wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    ((value >> 40) as u32 as f32) / ((1_u32 << 24) as f32)
}

#[cfg(test)]
#[path = "solver_tests.rs"]
mod tests;
