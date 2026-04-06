use bevy::prelude::*;

use crate::{PrecipitationKind, WeatherProfile, WeatherQuality};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum WeatherTransitionMode {
    Immediate,
    #[default]
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum WeatherScreenFxMode {
    #[default]
    BuiltInOverlay,
    StateOnly,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherTransitionRequest {
    pub profile: WeatherProfile,
    pub duration_secs: f32,
    pub mode: WeatherTransitionMode,
}

impl WeatherTransitionRequest {
    pub fn immediate(profile: WeatherProfile) -> Self {
        Self {
            profile,
            duration_secs: 0.0,
            mode: WeatherTransitionMode::Immediate,
        }
    }

    pub fn smooth(profile: WeatherProfile, duration_secs: f32) -> Self {
        Self {
            profile,
            duration_secs,
            mode: WeatherTransitionMode::Smooth,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WeatherConfig {
    pub initial_profile: WeatherProfile,
    pub seed: u64,
    pub diagnostics_enabled: bool,
    pub default_transition_duration_secs: f32,
    pub pending_request: Option<WeatherTransitionRequest>,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            initial_profile: WeatherProfile::clear(),
            seed: 0xC0FFEE_u64,
            diagnostics_enabled: true,
            default_transition_duration_secs: 4.0,
            pending_request: None,
        }
    }
}

impl WeatherConfig {
    pub fn queue_transition(&mut self, profile: WeatherProfile, duration_secs: f32) {
        self.pending_request = Some(WeatherTransitionRequest::smooth(profile, duration_secs));
    }

    pub fn queue_immediate(&mut self, profile: WeatherProfile) {
        self.pending_request = Some(WeatherTransitionRequest::immediate(profile));
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherScreenFxSettings {
    pub base_intensity: f32,
    pub rain_intensity: f32,
    pub snow_intensity: f32,
    pub fog_intensity: f32,
    pub storm_intensity: f32,
    pub droplet_intensity: f32,
    pub frost_intensity: f32,
    pub streak_intensity: f32,
    pub rain_tint: Color,
    pub snow_tint: Color,
    pub fog_tint: Color,
    pub storm_tint: Color,
}

impl Default for WeatherScreenFxSettings {
    fn default() -> Self {
        Self {
            base_intensity: 0.0,
            rain_intensity: 0.34,
            snow_intensity: 0.26,
            fog_intensity: 0.10,
            storm_intensity: 0.20,
            droplet_intensity: 0.40,
            frost_intensity: 0.42,
            streak_intensity: 0.28,
            rain_tint: Color::srgb(0.90, 0.96, 1.0),
            snow_tint: Color::srgb(0.90, 0.94, 1.0),
            fog_tint: Color::WHITE,
            storm_tint: Color::srgb(0.92, 0.95, 1.0),
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WeatherVisualsConfig {
    pub quality: WeatherQuality,
    pub screen_fx_mode: WeatherScreenFxMode,
    pub screen_fx: WeatherScreenFxSettings,
}

impl Default for WeatherVisualsConfig {
    fn default() -> Self {
        Self {
            quality: WeatherQuality::High,
            screen_fx_mode: WeatherScreenFxMode::BuiltInOverlay,
            screen_fx: WeatherScreenFxSettings::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherTransitionState {
    pub active: bool,
    pub elapsed_secs: f32,
    pub duration_secs: f32,
    pub progress: f32,
    pub source_label: Option<String>,
    pub target_label: Option<String>,
}

impl Default for WeatherTransitionState {
    fn default() -> Self {
        Self {
            active: false,
            elapsed_secs: 0.0,
            duration_secs: 0.0,
            progress: 1.0,
            source_label: None,
            target_label: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WindState {
    pub direction: Vec3,
    pub base_speed: f32,
    pub speed: f32,
    pub gust_factor: f32,
    pub vector: Vec3,
}

impl Default for WindState {
    fn default() -> Self {
        Self {
            direction: Vec3::X,
            base_speed: 0.0,
            speed: 0.0,
            gust_factor: 0.0,
            vector: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct PrecipitationState {
    pub kind: PrecipitationKind,
    pub intensity: f32,
    pub density: f32,
    pub fall_speed: f32,
    pub particle_size: Vec2,
    pub wind_influence: f32,
    pub near_radius: f32,
    pub near_height: f32,
    pub far_density: f32,
    pub tint: Color,
}

impl Default for PrecipitationState {
    fn default() -> Self {
        Self {
            kind: PrecipitationKind::None,
            intensity: 0.0,
            density: 0.0,
            fall_speed: 0.0,
            particle_size: Vec2::new(0.04, 0.7),
            wind_influence: 0.0,
            near_radius: 12.0,
            near_height: 10.0,
            far_density: 0.0,
            tint: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum VisibilityClass {
    #[default]
    Clear,
    Hazy,
    Low,
    Severe,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherVisibility {
    pub fog_density: f32,
    pub visibility_distance: f32,
    pub volumetric_intensity: f32,
    pub fog_color: Color,
    pub classification: VisibilityClass,
}

impl Default for WeatherVisibility {
    fn default() -> Self {
        Self {
            fog_density: 0.0,
            visibility_distance: 500.0,
            volumetric_intensity: 0.0,
            fog_color: Color::srgb(0.70, 0.74, 0.80),
            classification: VisibilityClass::Clear,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherScreenState {
    pub overlay_intensity: f32,
    pub droplet_intensity: f32,
    pub frost_intensity: f32,
    pub streak_intensity: f32,
    pub tint: Color,
}

impl Default for WeatherScreenState {
    fn default() -> Self {
        Self {
            overlay_intensity: 0.0,
            droplet_intensity: 0.0,
            frost_intensity: 0.0,
            streak_intensity: 0.0,
            tint: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct StormState {
    pub intensity: f32,
    pub lightning_active: bool,
    pub lightning_flash_intensity: f32,
    pub lightning_flash_id: Option<u64>,
}

impl Default for StormState {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            lightning_active: false,
            lightning_flash_intensity: 0.0,
            lightning_flash_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherFactors {
    pub rain_factor: f32,
    pub snow_factor: f32,
    pub fog_factor: f32,
    pub storm_factor: f32,
    pub wind_factor: f32,
    pub wetness_factor: f32,
}

impl Default for WeatherFactors {
    fn default() -> Self {
        Self {
            rain_factor: 0.0,
            snow_factor: 0.0,
            fog_factor: 0.0,
            storm_factor: 0.0,
            wind_factor: 0.0,
            wetness_factor: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WeatherRuntime {
    pub active_profile: WeatherProfile,
    pub target_profile: WeatherProfile,
    pub transition: WeatherTransitionState,
    pub wind: WindState,
    pub precipitation: PrecipitationState,
    pub visibility: WeatherVisibility,
    pub storm: StormState,
    pub factors: WeatherFactors,
}

impl Default for WeatherRuntime {
    fn default() -> Self {
        let profile = WeatherProfile::clear();
        Self {
            active_profile: profile.clone(),
            target_profile: profile,
            transition: WeatherTransitionState::default(),
            wind: WindState::default(),
            precipitation: PrecipitationState::default(),
            visibility: WeatherVisibility::default(),
            storm: StormState::default(),
            factors: WeatherFactors::default(),
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WeatherDiagnostics {
    pub active_profile_label: Option<String>,
    pub target_profile_label: Option<String>,
    pub transition_progress: f32,
    pub transition_active: bool,
    pub active_zone_count: usize,
    pub current_wind: Vec3,
    pub current_fog_density: f32,
    pub current_visibility_distance: f32,
    pub current_precipitation_kind: PrecipitationKind,
    pub primary_camera_name: Option<String>,
    pub primary_zone_label: Option<String>,
    pub last_transition_started_at: Option<f32>,
    pub last_transition_finished_at: Option<f32>,
    pub last_lightning_flash_id: Option<u64>,
    pub transition_started_count: u32,
    pub transition_finished_count: u32,
    pub profile_changed_count: u32,
    pub lightning_flash_count: u32,
}

impl Default for WeatherDiagnostics {
    fn default() -> Self {
        Self {
            active_profile_label: Some("Clear".into()),
            target_profile_label: Some("Clear".into()),
            transition_progress: 1.0,
            transition_active: false,
            active_zone_count: 0,
            current_wind: Vec3::ZERO,
            current_fog_density: 0.0,
            current_visibility_distance: 500.0,
            current_precipitation_kind: PrecipitationKind::None,
            primary_camera_name: None,
            primary_zone_label: None,
            last_transition_started_at: None,
            last_transition_finished_at: None,
            last_lightning_flash_id: None,
            transition_started_count: 0,
            transition_finished_count: 0,
            profile_changed_count: 0,
            lightning_flash_count: 0,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct WeatherVisualDiagnostics {
    pub quality: WeatherQuality,
    pub active_emitters: usize,
    pub precipitation_particles_estimate: usize,
    pub managed_screen_overlays: usize,
}

impl Default for WeatherVisualDiagnostics {
    fn default() -> Self {
        Self {
            quality: WeatherQuality::High,
            active_emitters: 0,
            precipitation_particles_estimate: 0,
            managed_screen_overlays: 0,
        }
    }
}
