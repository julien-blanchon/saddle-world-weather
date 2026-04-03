use bevy::prelude::*;

use crate::{PrecipitationKind, WeatherProfile};

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherCamera {
    pub enabled: bool,
    pub priority: i32,
    pub receive_precipitation: bool,
    pub receive_screen_fx: bool,
    pub apply_distance_fog: bool,
    pub insert_missing_components: bool,
    pub quality_bias: f32,
    pub precipitation_blocked_factor: f32,
}

impl Default for WeatherCamera {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 0,
            receive_precipitation: true,
            receive_screen_fx: true,
            apply_distance_fog: true,
            insert_missing_components: true,
            quality_bias: 1.0,
            precipitation_blocked_factor: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherCameraState {
    pub base_profile_label: Option<String>,
    pub resolved_profile_label: Option<String>,
    pub zone_label: Option<String>,
    pub transition_progress: f32,
    pub precipitation_kind: PrecipitationKind,
    pub precipitation_factor: f32,
    pub precipitation_density: f32,
    pub fall_speed: f32,
    pub particle_size: Vec2,
    pub wind_influence: f32,
    pub near_radius: f32,
    pub near_height: f32,
    pub far_density: f32,
    pub occlusion_factor: f32,
    pub screen_fx_factor: f32,
    pub screen_tint: Color,
    pub wetness_factor: f32,
    pub fog_density: f32,
    pub fog_color: Color,
    pub visibility_distance: f32,
    pub wind_vector: Vec3,
    pub lightning_flash_intensity: f32,
    pub active_particles: usize,
}

impl Default for WeatherCameraState {
    fn default() -> Self {
        Self {
            base_profile_label: None,
            resolved_profile_label: None,
            zone_label: None,
            transition_progress: 1.0,
            precipitation_kind: PrecipitationKind::None,
            precipitation_factor: 0.0,
            precipitation_density: 0.0,
            fall_speed: 0.0,
            particle_size: Vec2::new(0.04, 0.7),
            wind_influence: 0.0,
            near_radius: 12.0,
            near_height: 10.0,
            far_density: 0.0,
            occlusion_factor: 1.0,
            screen_fx_factor: 0.0,
            screen_tint: Color::WHITE,
            wetness_factor: 0.0,
            fog_density: 0.0,
            fog_color: Color::srgb(0.70, 0.74, 0.80),
            visibility_distance: 500.0,
            wind_vector: Vec3::ZERO,
            lightning_flash_intensity: 0.0,
            active_particles: 0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherSurface {
    pub enabled: bool,
    pub wetness_response: f32,
    pub puddle_response: f32,
    pub snow_response: f32,
    pub wetting_speed: f32,
    pub drying_speed: f32,
    pub puddle_fill_speed: f32,
    pub puddle_drain_speed: f32,
    pub snow_accumulation_speed: f32,
    pub snow_melt_speed: f32,
    pub puddle_threshold: f32,
    pub max_puddle_coverage: f32,
    pub max_snow_coverage: f32,
    pub wet_roughness: f32,
    pub puddle_roughness: f32,
    pub snow_roughness: f32,
    pub wet_reflectance: f32,
    pub puddle_reflectance: f32,
    pub snow_reflectance: f32,
    pub wet_darkening: f32,
    pub puddle_darkening: f32,
    pub snow_tint: Color,
}

impl Default for WeatherSurface {
    fn default() -> Self {
        Self {
            enabled: true,
            wetness_response: 1.0,
            puddle_response: 0.85,
            snow_response: 1.0,
            wetting_speed: 0.55,
            drying_speed: 0.08,
            puddle_fill_speed: 0.30,
            puddle_drain_speed: 0.06,
            snow_accumulation_speed: 0.22,
            snow_melt_speed: 0.10,
            puddle_threshold: 0.35,
            max_puddle_coverage: 0.7,
            max_snow_coverage: 1.0,
            wet_roughness: 0.18,
            puddle_roughness: 0.04,
            snow_roughness: 0.92,
            wet_reflectance: 0.34,
            puddle_reflectance: 0.52,
            snow_reflectance: 0.16,
            wet_darkening: 0.18,
            puddle_darkening: 0.28,
            snow_tint: Color::srgb(0.92, 0.95, 1.0),
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherSurfaceState {
    pub base_profile_label: Option<String>,
    pub resolved_profile_label: Option<String>,
    pub zone_label: Option<String>,
    pub precipitation_kind: PrecipitationKind,
    pub rain_factor: f32,
    pub snow_factor: f32,
    pub wetness_factor: f32,
    pub wetness: f32,
    pub puddle_coverage: f32,
    pub snow_coverage: f32,
}

impl Default for WeatherSurfaceState {
    fn default() -> Self {
        Self {
            base_profile_label: None,
            resolved_profile_label: None,
            zone_label: None,
            precipitation_kind: PrecipitationKind::None,
            rain_factor: 0.0,
            snow_factor: 0.0,
            wetness_factor: 0.0,
            wetness: 0.0,
            puddle_coverage: 0.0,
            snow_coverage: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum WeatherVolumeShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
}

impl Default for WeatherVolumeShape {
    fn default() -> Self {
        Self::Sphere { radius: 8.0 }
    }
}

impl WeatherVolumeShape {
    pub fn influence(&self, local_point: Vec3, blend_distance: f32) -> f32 {
        let blend_distance = blend_distance.max(0.0);
        match self {
            Self::Sphere { radius } => {
                let radius = radius.max(0.01);
                let distance = local_point.length();
                if distance <= radius {
                    1.0
                } else if blend_distance > 0.0 && distance <= radius + blend_distance {
                    1.0 - ((distance - radius) / blend_distance)
                } else {
                    0.0
                }
            }
            Self::Box { half_extents } => {
                let half_extents = half_extents.max(Vec3::splat(0.01));
                let delta = local_point.abs() - half_extents;
                let outside = delta.max(Vec3::ZERO).length();
                if outside <= 0.0 {
                    1.0
                } else if blend_distance > 0.0 && outside <= blend_distance {
                    1.0 - outside / blend_distance
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherZone {
    pub label: Option<String>,
    pub enabled: bool,
    pub profile: WeatherProfile,
    pub shape: WeatherVolumeShape,
    pub blend_distance: f32,
    pub priority: i32,
    pub weight: f32,
}

impl Default for WeatherZone {
    fn default() -> Self {
        Self {
            label: Some("Weather Zone".into()),
            enabled: true,
            profile: WeatherProfile::foggy(),
            shape: WeatherVolumeShape::default(),
            blend_distance: 6.0,
            priority: 0,
            weight: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct WeatherOcclusionVolume {
    pub label: Option<String>,
    pub enabled: bool,
    pub shape: WeatherVolumeShape,
    pub blend_distance: f32,
    pub precipitation_multiplier: f32,
    pub screen_fx_multiplier: f32,
}

impl Default for WeatherOcclusionVolume {
    fn default() -> Self {
        Self {
            label: Some("Weather Occlusion".into()),
            enabled: true,
            shape: WeatherVolumeShape::default(),
            blend_distance: 3.0,
            precipitation_multiplier: 0.0,
            screen_fx_multiplier: 0.0,
        }
    }
}
