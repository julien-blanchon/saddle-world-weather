use bevy::{color::LinearRgba, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum WeatherQuality {
    Low,
    Medium,
    #[default]
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherQualityPlan {
    pub max_particles_per_camera: usize,
    pub enable_screen_fx: bool,
    pub overlay_resolution: u32,
}

impl WeatherQuality {
    pub const fn plan(self) -> WeatherQualityPlan {
        match self {
            Self::Low => WeatherQualityPlan {
                max_particles_per_camera: 48,
                enable_screen_fx: false,
                overlay_resolution: 64,
            },
            Self::Medium => WeatherQualityPlan {
                max_particles_per_camera: 120,
                enable_screen_fx: true,
                overlay_resolution: 96,
            },
            Self::High => WeatherQualityPlan {
                max_particles_per_camera: 220,
                enable_screen_fx: true,
                overlay_resolution: 128,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect, Default)]
pub enum PrecipitationKind {
    #[default]
    None,
    Rain,
    Snow,
    Particulate,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct PrecipitationProfile {
    pub label: Option<String>,
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

impl Default for PrecipitationProfile {
    fn default() -> Self {
        Self::none()
    }
}

impl PrecipitationProfile {
    pub fn none() -> Self {
        Self {
            label: None,
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

    pub fn rain() -> Self {
        Self {
            label: Some("Rain".into()),
            kind: PrecipitationKind::Rain,
            intensity: 0.8,
            density: 0.8,
            fall_speed: 18.0,
            particle_size: Vec2::new(0.03, 0.85),
            wind_influence: 0.85,
            near_radius: 12.0,
            near_height: 11.0,
            far_density: 0.35,
            tint: Color::srgb(0.82, 0.90, 1.0).with_alpha(0.42),
        }
    }

    pub fn snow() -> Self {
        Self {
            label: Some("Snow".into()),
            kind: PrecipitationKind::Snow,
            intensity: 0.7,
            density: 0.75,
            fall_speed: 4.0,
            particle_size: Vec2::splat(0.08),
            wind_influence: 0.55,
            near_radius: 10.0,
            near_height: 9.0,
            far_density: 0.28,
            tint: Color::WHITE.with_alpha(0.88),
        }
    }

    pub fn clamped(mut self) -> Self {
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.density = self.density.clamp(0.0, 1.0);
        self.fall_speed = self.fall_speed.max(0.0);
        self.particle_size = self.particle_size.max(Vec2::splat(0.01));
        self.wind_influence = self.wind_influence.clamp(0.0, 2.0);
        self.near_radius = self.near_radius.max(1.0);
        self.near_height = self.near_height.max(1.0);
        self.far_density = self.far_density.clamp(0.0, 1.0);
        if self.intensity <= 0.001 || self.density <= 0.001 {
            self.kind = PrecipitationKind::None;
        }
        if matches!(self.kind, PrecipitationKind::None) {
            self.intensity = 0.0;
            self.density = 0.0;
            self.fall_speed = 0.0;
            self.far_density = 0.0;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct FogProfile {
    pub color: Color,
    pub density: f32,
    pub visibility_distance: f32,
    pub volumetric_intensity: f32,
}

impl Default for FogProfile {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.70, 0.74, 0.80),
            density: 0.04,
            visibility_distance: 220.0,
            volumetric_intensity: 0.0,
        }
    }
}

impl FogProfile {
    pub fn clamped(mut self) -> Self {
        self.density = self.density.clamp(0.0, 1.0);
        self.visibility_distance = self.visibility_distance.max(5.0);
        self.volumetric_intensity = self.volumetric_intensity.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WindProfile {
    pub direction: Vec3,
    pub base_speed: f32,
    pub gust_amplitude: f32,
    pub gust_frequency_hz: f32,
    pub sway: f32,
}

impl Default for WindProfile {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.6, 0.0, 0.2),
            base_speed: 2.0,
            gust_amplitude: 0.15,
            gust_frequency_hz: 0.22,
            sway: 0.2,
        }
    }
}

impl WindProfile {
    pub fn clamped(mut self) -> Self {
        if self.direction.length_squared() <= f32::EPSILON {
            self.direction = Vec3::X;
        }
        self.direction.y = 0.0;
        self.direction = self.direction.normalize_or_zero();
        if self.direction == Vec3::ZERO {
            self.direction = Vec3::X;
        }
        self.base_speed = self.base_speed.max(0.0);
        self.gust_amplitude = self.gust_amplitude.clamp(0.0, 1.0);
        self.gust_frequency_hz = self.gust_frequency_hz.max(0.0);
        self.sway = self.sway.clamp(0.0, 2.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ScreenFxProfile {
    pub intensity: f32,
    pub droplet_intensity: f32,
    pub frost_intensity: f32,
    pub streak_intensity: f32,
    pub tint: Color,
}

impl Default for ScreenFxProfile {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            droplet_intensity: 0.0,
            frost_intensity: 0.0,
            streak_intensity: 0.0,
            tint: Color::WHITE,
        }
    }
}

impl ScreenFxProfile {
    pub fn clamped(mut self) -> Self {
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.droplet_intensity = self.droplet_intensity.clamp(0.0, 1.0);
        self.frost_intensity = self.frost_intensity.clamp(0.0, 1.0);
        self.streak_intensity = self.streak_intensity.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct StormProfile {
    pub intensity: f32,
    pub lightning_frequency_hz: f32,
    pub lightning_duration_secs: f32,
    pub lightning_brightness: f32,
    pub wetness_bonus: f32,
}

impl Default for StormProfile {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            lightning_frequency_hz: 0.0,
            lightning_duration_secs: 0.12,
            lightning_brightness: 0.0,
            wetness_bonus: 0.0,
        }
    }
}

impl StormProfile {
    pub fn clamped(mut self) -> Self {
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.lightning_frequency_hz = self.lightning_frequency_hz.max(0.0);
        self.lightning_duration_secs = self.lightning_duration_secs.clamp(0.02, 1.0);
        self.lightning_brightness = self.lightning_brightness.max(0.0);
        self.wetness_bonus = self.wetness_bonus.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WeatherProfile {
    pub label: Option<String>,
    pub precipitation: PrecipitationProfile,
    pub fog: FogProfile,
    pub wind: WindProfile,
    pub screen_fx: ScreenFxProfile,
    pub storm: StormProfile,
}

impl Default for WeatherProfile {
    fn default() -> Self {
        Self::clear()
    }
}

impl WeatherProfile {
    pub fn clear() -> Self {
        Self {
            label: Some("Clear".into()),
            precipitation: PrecipitationProfile::none(),
            fog: FogProfile::default(),
            wind: WindProfile::default(),
            screen_fx: ScreenFxProfile::default(),
            storm: StormProfile::default(),
        }
    }

    pub fn rain() -> Self {
        Self {
            label: Some("Rain".into()),
            precipitation: PrecipitationProfile::rain(),
            fog: FogProfile {
                color: Color::srgb(0.62, 0.68, 0.76),
                density: 0.18,
                visibility_distance: 95.0,
                volumetric_intensity: 0.18,
            },
            wind: WindProfile {
                direction: Vec3::new(0.8, 0.0, 0.4),
                base_speed: 5.0,
                gust_amplitude: 0.28,
                gust_frequency_hz: 0.38,
                sway: 0.42,
            },
            screen_fx: ScreenFxProfile {
                intensity: 0.34,
                droplet_intensity: 0.40,
                frost_intensity: 0.0,
                streak_intensity: 0.28,
                tint: Color::srgb(0.90, 0.96, 1.0),
            },
            storm: StormProfile::default(),
        }
    }

    pub fn snow() -> Self {
        Self {
            label: Some("Snow".into()),
            precipitation: PrecipitationProfile::snow(),
            fog: FogProfile {
                color: Color::srgb(0.83, 0.86, 0.90),
                density: 0.16,
                visibility_distance: 105.0,
                volumetric_intensity: 0.10,
            },
            wind: WindProfile {
                direction: Vec3::new(-0.5, 0.0, 0.3),
                base_speed: 4.0,
                gust_amplitude: 0.42,
                gust_frequency_hz: 0.28,
                sway: 0.60,
            },
            screen_fx: ScreenFxProfile {
                intensity: 0.26,
                droplet_intensity: 0.0,
                frost_intensity: 0.42,
                streak_intensity: 0.12,
                tint: Color::srgb(0.90, 0.94, 1.0),
            },
            storm: StormProfile {
                intensity: 0.16,
                wetness_bonus: 0.08,
                ..default()
            },
        }
    }

    pub fn foggy() -> Self {
        Self {
            label: Some("Foggy".into()),
            precipitation: PrecipitationProfile::none(),
            fog: FogProfile {
                color: Color::srgb(0.74, 0.78, 0.81),
                density: 0.42,
                visibility_distance: 42.0,
                volumetric_intensity: 0.24,
            },
            wind: WindProfile {
                direction: Vec3::new(0.3, 0.0, 0.1),
                base_speed: 1.0,
                gust_amplitude: 0.08,
                gust_frequency_hz: 0.12,
                sway: 0.1,
            },
            screen_fx: ScreenFxProfile {
                intensity: 0.10,
                droplet_intensity: 0.0,
                frost_intensity: 0.0,
                streak_intensity: 0.0,
                tint: Color::WHITE,
            },
            storm: StormProfile::default(),
        }
    }

    pub fn storm() -> Self {
        Self {
            label: Some("Storm".into()),
            precipitation: PrecipitationProfile {
                intensity: 1.0,
                density: 1.0,
                fall_speed: 20.0,
                far_density: 0.55,
                ..PrecipitationProfile::rain()
            },
            fog: FogProfile {
                color: Color::srgb(0.52, 0.58, 0.66),
                density: 0.28,
                visibility_distance: 55.0,
                volumetric_intensity: 0.30,
            },
            wind: WindProfile {
                direction: Vec3::new(1.0, 0.0, 0.25),
                base_speed: 12.0,
                gust_amplitude: 0.70,
                gust_frequency_hz: 0.55,
                sway: 0.95,
            },
            screen_fx: ScreenFxProfile {
                intensity: 0.54,
                droplet_intensity: 0.52,
                frost_intensity: 0.0,
                streak_intensity: 0.46,
                tint: Color::srgb(0.92, 0.95, 1.0),
            },
            storm: StormProfile {
                intensity: 1.0,
                lightning_frequency_hz: 0.18,
                lightning_duration_secs: 0.14,
                lightning_brightness: 1.0,
                wetness_bonus: 0.20,
            },
        }
    }

    pub fn clamped(mut self) -> Self {
        self.precipitation = self.precipitation.clamped();
        self.fog = self.fog.clamped();
        self.wind = self.wind.clamped();
        self.screen_fx = self.screen_fx.clamped();
        self.storm = self.storm.clamped();

        if self.storm.intensity > 0.7 && matches!(self.precipitation.kind, PrecipitationKind::None)
        {
            self.precipitation = PrecipitationProfile::rain();
            self.precipitation.intensity = (self.storm.intensity * 0.6).clamp(0.2, 1.0);
            self.precipitation.density = self.precipitation.intensity;
            self.precipitation.far_density = (self.precipitation.intensity * 0.5).clamp(0.0, 1.0);
        }

        self
    }

    pub fn blend(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        if t <= 0.0 {
            return self.clone().clamped();
        }
        if t >= 1.0 {
            return other.clone().clamped();
        }
        let precip_weight_a = self.precipitation.intensity * (1.0 - t);
        let precip_weight_b = other.precipitation.intensity * t;
        let precipitation_kind = if precip_weight_a.max(precip_weight_b) <= 0.001 {
            PrecipitationKind::None
        } else if precip_weight_b > precip_weight_a {
            other.precipitation.kind.clone()
        } else {
            self.precipitation.kind.clone()
        };

        Self {
            label: if t < 0.5 {
                self.label.clone()
            } else {
                other.label.clone()
            },
            precipitation: PrecipitationProfile {
                label: if t < 0.5 {
                    self.precipitation.label.clone()
                } else {
                    other.precipitation.label.clone()
                },
                kind: precipitation_kind,
                intensity: lerp_scalar(
                    self.precipitation.intensity,
                    other.precipitation.intensity,
                    t,
                ),
                density: lerp_scalar(self.precipitation.density, other.precipitation.density, t),
                fall_speed: lerp_scalar(
                    self.precipitation.fall_speed,
                    other.precipitation.fall_speed,
                    t,
                ),
                particle_size: self
                    .precipitation
                    .particle_size
                    .lerp(other.precipitation.particle_size, t),
                wind_influence: lerp_scalar(
                    self.precipitation.wind_influence,
                    other.precipitation.wind_influence,
                    t,
                ),
                near_radius: lerp_scalar(
                    self.precipitation.near_radius,
                    other.precipitation.near_radius,
                    t,
                ),
                near_height: lerp_scalar(
                    self.precipitation.near_height,
                    other.precipitation.near_height,
                    t,
                ),
                far_density: lerp_scalar(
                    self.precipitation.far_density,
                    other.precipitation.far_density,
                    t,
                ),
                tint: lerp_color(self.precipitation.tint, other.precipitation.tint, t),
            }
            .clamped(),
            fog: FogProfile {
                color: lerp_color(self.fog.color, other.fog.color, t),
                density: lerp_scalar(self.fog.density, other.fog.density, t),
                visibility_distance: lerp_scalar(
                    self.fog.visibility_distance,
                    other.fog.visibility_distance,
                    t,
                ),
                volumetric_intensity: lerp_scalar(
                    self.fog.volumetric_intensity,
                    other.fog.volumetric_intensity,
                    t,
                ),
            }
            .clamped(),
            wind: WindProfile {
                direction: self.wind.direction.lerp(other.wind.direction, t),
                base_speed: lerp_scalar(self.wind.base_speed, other.wind.base_speed, t),
                gust_amplitude: lerp_scalar(self.wind.gust_amplitude, other.wind.gust_amplitude, t),
                gust_frequency_hz: lerp_scalar(
                    self.wind.gust_frequency_hz,
                    other.wind.gust_frequency_hz,
                    t,
                ),
                sway: lerp_scalar(self.wind.sway, other.wind.sway, t),
            }
            .clamped(),
            screen_fx: ScreenFxProfile {
                intensity: lerp_scalar(self.screen_fx.intensity, other.screen_fx.intensity, t),
                droplet_intensity: lerp_scalar(
                    self.screen_fx.droplet_intensity,
                    other.screen_fx.droplet_intensity,
                    t,
                ),
                frost_intensity: lerp_scalar(
                    self.screen_fx.frost_intensity,
                    other.screen_fx.frost_intensity,
                    t,
                ),
                streak_intensity: lerp_scalar(
                    self.screen_fx.streak_intensity,
                    other.screen_fx.streak_intensity,
                    t,
                ),
                tint: lerp_color(self.screen_fx.tint, other.screen_fx.tint, t),
            }
            .clamped(),
            storm: StormProfile {
                intensity: lerp_scalar(self.storm.intensity, other.storm.intensity, t),
                lightning_frequency_hz: lerp_scalar(
                    self.storm.lightning_frequency_hz,
                    other.storm.lightning_frequency_hz,
                    t,
                ),
                lightning_duration_secs: lerp_scalar(
                    self.storm.lightning_duration_secs,
                    other.storm.lightning_duration_secs,
                    t,
                ),
                lightning_brightness: lerp_scalar(
                    self.storm.lightning_brightness,
                    other.storm.lightning_brightness,
                    t,
                ),
                wetness_bonus: lerp_scalar(self.storm.wetness_bonus, other.storm.wetness_bonus, t),
            }
            .clamped(),
        }
        .clamped()
    }
}

pub(crate) fn lerp_scalar(left: f32, right: f32, t: f32) -> f32 {
    left + (right - left) * t.clamp(0.0, 1.0)
}

pub(crate) fn lerp_color(left: Color, right: Color, t: f32) -> Color {
    let left = LinearRgba::from(left);
    let right = LinearRgba::from(right);
    Color::linear_rgba(
        lerp_scalar(left.red, right.red, t),
        lerp_scalar(left.green, right.green, t),
        lerp_scalar(left.blue, right.blue, t),
        lerp_scalar(left.alpha, right.alpha, t),
    )
}
