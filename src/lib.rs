mod components;
mod messages;
mod profiles;
mod resources;
mod solver;
mod surface_materials;
mod surfaces;
mod systems;
mod visuals;

pub use components::{
    WeatherCamera, WeatherCameraState, WeatherCameraVisualState, WeatherOcclusionVolume,
    WeatherSurface, WeatherSurfaceStandardMaterial, WeatherSurfaceState, WeatherVolumeShape,
    WeatherZone,
};
pub use messages::{
    LightningFlashEmitted, WeatherProfileChanged, WeatherTransitionFinished,
    WeatherTransitionStarted,
};
pub use profiles::{
    FogProfile, PrecipitationKind, PrecipitationProfile, StormProfile, WeatherProfile,
    WeatherQuality, WeatherQualityPlan, WindProfile,
};
pub use resources::{
    PrecipitationState, StormState, VisibilityClass, WeatherConfig, WeatherDiagnostics,
    WeatherFactors, WeatherRuntime, WeatherScreenFxMode, WeatherScreenFxSettings,
    WeatherScreenState, WeatherTransitionMode, WeatherTransitionRequest, WeatherTransitionState,
    WeatherVisibility, WeatherVisualDiagnostics, WeatherVisualsConfig, WindState,
};
pub use solver::{
    LightningSample, OcclusionContribution, OcclusionResult, ZoneBlendResult, ZoneContribution,
    resolve_occlusion, resolve_runtime, resolve_zone_profile, sample_gust,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum WeatherSystems {
    ApplyRequests,
    AdvanceTransition,
    ResolveBaseState,
    SyncSurfaces,
    ResolveCameraState,
    EmitMessages,
    Diagnostics,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum WeatherVisualSystems {
    ResolveState,
    SyncEmitters,
    SyncFog,
    SyncScreenEffects,
    Diagnostics,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum WeatherSurfaceMaterialSystems {
    ApplyMaterials,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct WeatherPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub config: WeatherConfig,
}

impl WeatherPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            config: WeatherConfig::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_config(mut self, config: WeatherConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for WeatherPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.insert_resource(self.config.clone())
            .init_resource::<WeatherRuntime>()
            .init_resource::<WeatherDiagnostics>()
            .init_resource::<systems::PendingWeatherMessages>()
            .init_resource::<systems::WeatherInternalState>()
            .add_message::<WeatherTransitionStarted>()
            .add_message::<WeatherTransitionFinished>()
            .add_message::<WeatherProfileChanged>()
            .add_message::<LightningFlashEmitted>()
            .register_type::<FogProfile>()
            .register_type::<PrecipitationKind>()
            .register_type::<PrecipitationProfile>()
            .register_type::<PrecipitationState>()
            .register_type::<StormProfile>()
            .register_type::<StormState>()
            .register_type::<VisibilityClass>()
            .register_type::<WeatherCamera>()
            .register_type::<WeatherCameraState>()
            .register_type::<WeatherConfig>()
            .register_type::<WeatherDiagnostics>()
            .register_type::<WeatherFactors>()
            .register_type::<WeatherOcclusionVolume>()
            .register_type::<WeatherProfile>()
            .register_type::<WeatherRuntime>()
            .register_type::<WeatherSurface>()
            .register_type::<WeatherSurfaceState>()
            .register_type::<WeatherTransitionMode>()
            .register_type::<WeatherTransitionRequest>()
            .register_type::<WeatherTransitionState>()
            .register_type::<WeatherVisibility>()
            .register_type::<WeatherVolumeShape>()
            .register_type::<WeatherZone>()
            .register_type::<WindProfile>()
            .register_type::<WindState>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(
                self.deactivate_schedule,
                (
                    systems::deactivate_runtime,
                    systems::cleanup_runtime,
                    surfaces::reset_surface_states,
                )
                    .chain(),
            )
            .configure_sets(
                self.update_schedule,
                (
                    WeatherSystems::ApplyRequests,
                    WeatherSystems::AdvanceTransition,
                    WeatherSystems::ResolveBaseState,
                    WeatherSystems::SyncSurfaces,
                    WeatherSystems::ResolveCameraState,
                    WeatherSystems::EmitMessages,
                    WeatherSystems::Diagnostics,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                systems::apply_weather_requests
                    .in_set(WeatherSystems::ApplyRequests)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::advance_transition
                    .in_set(WeatherSystems::AdvanceTransition)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::resolve_base_runtime
                    .in_set(WeatherSystems::ResolveBaseState)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                surfaces::sync_surface_states
                    .in_set(WeatherSystems::SyncSurfaces)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::resolve_camera_states
                    .in_set(WeatherSystems::ResolveCameraState)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::emit_pending_messages
                    .in_set(WeatherSystems::EmitMessages)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::publish_diagnostics
                    .in_set(WeatherSystems::Diagnostics)
                    .run_if(systems::runtime_is_active),
            );
    }
}

pub struct WeatherVisualsPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub config: WeatherVisualsConfig,
}

impl WeatherVisualsPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            config: WeatherVisualsConfig::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_config(mut self, config: WeatherVisualsConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for WeatherVisualsPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for WeatherVisualsPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.insert_resource(self.config.clone())
            .init_resource::<WeatherVisualDiagnostics>()
            .init_resource::<visuals::WeatherVisualAssets>()
            .register_type::<WeatherCameraVisualState>()
            .register_type::<WeatherQuality>()
            .register_type::<WeatherScreenFxMode>()
            .register_type::<WeatherScreenFxSettings>()
            .register_type::<WeatherScreenState>()
            .register_type::<WeatherVisualDiagnostics>()
            .register_type::<WeatherVisualsConfig>()
            .add_systems(self.activate_schedule, visuals::activate_visuals)
            .add_systems(self.deactivate_schedule, visuals::cleanup_visuals)
            .configure_sets(
                self.update_schedule,
                (
                    WeatherVisualSystems::ResolveState,
                    WeatherVisualSystems::SyncEmitters,
                    WeatherVisualSystems::SyncFog,
                    WeatherVisualSystems::SyncScreenEffects,
                    WeatherVisualSystems::Diagnostics,
                )
                    .chain(),
            )
            .configure_sets(
                self.update_schedule,
                WeatherVisualSystems::ResolveState.after(WeatherSystems::ResolveCameraState),
            )
            .add_systems(
                self.update_schedule,
                visuals::resolve_camera_visual_states
                    .in_set(WeatherVisualSystems::ResolveState)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                visuals::sync_precipitation_emitters
                    .in_set(WeatherVisualSystems::SyncEmitters)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                visuals::sync_distance_fog
                    .in_set(WeatherVisualSystems::SyncFog)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                visuals::sync_screen_effect_overlays
                    .in_set(WeatherVisualSystems::SyncScreenEffects)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                visuals::publish_visual_diagnostics
                    .in_set(WeatherVisualSystems::Diagnostics)
                    .run_if(systems::runtime_is_active),
            );
    }
}

pub struct WeatherSurfaceMaterialsPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl WeatherSurfaceMaterialsPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for WeatherSurfaceMaterialsPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for WeatherSurfaceMaterialsPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.register_type::<WeatherSurfaceStandardMaterial>()
            .add_systems(
                self.deactivate_schedule,
                surface_materials::reset_surface_materials,
            )
            .configure_sets(
                self.update_schedule,
                WeatherSurfaceMaterialSystems::ApplyMaterials.after(WeatherSystems::SyncSurfaces),
            )
            .add_systems(
                self.update_schedule,
                surface_materials::sync_surface_materials
                    .in_set(WeatherSurfaceMaterialSystems::ApplyMaterials)
                    .run_if(systems::runtime_is_active),
            );
    }
}
