# Saddle World Weather

Reusable Bevy weather orchestration for authored profiles, deterministic transitions, camera-local precipitation, fog sync, wind, shelter suppression, local override zones, and optional screen-space weather cues.

The crate stays shared-crate safe:

- no project-specific imports
- `bevy = "0.18"` pinned directly
- injectable activate, deactivate, and update schedules
- public `WeatherSystems` for downstream ordering
- pure-Rust profile blending, wind sampling, zone blending, occlusion resolution, and lightning timing logic

## Quick Start

```rust
use bevy::prelude::*;
use saddle_world_weather::{WeatherConfig, WeatherPlugin, WeatherProfile, WeatherSystems};

let mut app = App::new();
app.add_plugins(DefaultPlugins);
app.add_plugins(
    WeatherPlugin::new(
        OnEnter(MyState::Gameplay),
        OnExit(MyState::Gameplay),
        Update,
    )
    .with_config(WeatherConfig {
        initial_profile: WeatherProfile::rain(),
        ..default()
    }),
);

app.configure_sets(
    Update,
    WeatherSystems::ResolveBaseState.in_set(MyGameSet::Simulation),
);
```

Attach `WeatherCamera` to any camera that should receive precipitation, fog sync, or screen-space cues. Add `WeatherZone` and `WeatherOcclusionVolume` when you need local overrides or shelter suppression.

## Public API

### Plugin

- `WeatherPlugin`
- `WeatherPlugin::new(activate_schedule, deactivate_schedule, update_schedule)`
- `WeatherPlugin::always_on(update_schedule)`
- `WeatherPlugin::with_config(config)`

### Resources

- `WeatherConfig`
- `WeatherRuntime`
- `WeatherDiagnostics`
- `WeatherTransitionRequest`
- `WeatherTransitionState`
- `WindState`
- `PrecipitationState`
- `WeatherVisibility`
- `WeatherScreenState`
- `StormState`
- `WeatherFactors`

### Components

- `WeatherCamera`
- `WeatherCameraState` (base and resolved labels, local precipitation/fog/wind state)
- `WeatherZone`
- `WeatherOcclusionVolume`
- `WeatherVolumeShape`

### Messages

- `WeatherTransitionStarted`
- `WeatherTransitionFinished`
- `WeatherProfileChanged`
- `LightningFlashEmitted`

### System Sets

- `WeatherSystems::ApplyRequests`
- `WeatherSystems::AdvanceTransition`
- `WeatherSystems::ResolveBaseState`
- `WeatherSystems::ResolveCameraState`
- `WeatherSystems::SyncEmitters`
- `WeatherSystems::SyncFog`
- `WeatherSystems::SyncScreenEffects`
- `WeatherSystems::EmitMessages`
- `WeatherSystems::Diagnostics`

## Examples

| Example | Purpose |
|--------|---------|
| `basic` | Minimal clear-to-rain loop with one weather camera and overlay diagnostics |
| `transitions` | Cycles through clear, foggy, rain, storm, and snow authored profiles |
| `windy_snow` | Shows a gust-heavy snow profile with stronger lateral drift |
| `localized_zones` | Demonstrates fog and storm pockets blending around the active camera |
| `camera_screen_fx` | Compares gameplay and cinematic cameras side by side |
| `shelter_and_occlusion` | Moves through a roofed shelter to suppress precipitation and screen cues |
| `saddle-world-weather-lab` | Crate-local BRP/E2E verification app with deterministic screenshot scenarios |

## Design Notes

- Precipitation is camera-local and recycled. The crate does not attempt full-map literal particle simulation.
- `WeatherCameraState` separates the global transitioned label from the camera-local resolved label so BRP inspection can distinguish “global clear” from “local storm pocket”.
- Fog sync uses Bevy `DistanceFog` on opted-in cameras.
- Lightning is currently a deterministic screen-space flash cue plus message surface, not a sky-lightning or thunder system.
- Quality scaling changes particle budgets and screen-effect participation without leaking backend details into the public API, and `WeatherDiagnostics` exposes the active quality tier plus message counters for runtime debugging.

## Limitations

- No volumetric cloud or sky rendering. Pair this crate with a dedicated sky or day-night system if needed.
- No GPU particle backend. The default implementation uses crate-owned CPU-managed precipitation emitters.
- The crate emits wetness and visibility hints but does not mutate arbitrary scene materials on your behalf.

## Documentation

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/configuration.md`](docs/configuration.md)
- [`docs/debugging.md`](docs/debugging.md)
