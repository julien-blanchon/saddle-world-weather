# Saddle World Weather

Reusable Bevy weather orchestration centered on a pure weather-state engine plus opt-in presentation adapters.

Core features:

- authored `WeatherProfile` blending for precipitation, fog, wind, and storms
- deterministic transitions, gust sampling, lightning timing, zone blending, and shelter suppression
- per-camera resolved weather state for downstream rendering or gameplay
- per-surface wetness / puddle / snow accumulation state without forcing a render policy

Optional built-in adapters:

- `WeatherVisualsPlugin` for camera-local precipitation, `DistanceFog`, and screen overlays
- `WeatherSurfaceMaterialsPlugin` for `StandardMaterial` wetness / snow modulation

## Quick Start

```rust
use bevy::prelude::*;
use saddle_world_weather::{
    WeatherConfig, WeatherPlugin, WeatherProfile, WeatherSurfaceMaterialsPlugin,
    WeatherVisualsConfig, WeatherVisualsPlugin,
};

let weather = WeatherConfig {
    initial_profile: WeatherProfile::rain(),
    ..default()
};

let visuals = WeatherVisualsConfig::default();

let mut app = App::new();
app.add_plugins(DefaultPlugins);
app.add_plugins((
    WeatherPlugin::new(OnEnter(MyState::Gameplay), OnExit(MyState::Gameplay), Update)
        .with_config(weather),
    WeatherVisualsPlugin::new(OnEnter(MyState::Gameplay), OnExit(MyState::Gameplay), Update)
        .with_config(visuals),
    WeatherSurfaceMaterialsPlugin::new(
        OnEnter(MyState::Gameplay),
        OnExit(MyState::Gameplay),
        Update,
    ),
));
```

Use only `WeatherPlugin` if another crate owns precipitation, fog, post-processing, or material response. Add `WeatherVisualsPlugin` and `WeatherSurfaceMaterialsPlugin` only where you want the bundled adapters.

## Public API

Plugins:

- `WeatherPlugin`
- `WeatherVisualsPlugin`
- `WeatherSurfaceMaterialsPlugin`

Core resources:

- `WeatherConfig`
- `WeatherRuntime`
- `WeatherDiagnostics`
- `WeatherTransitionRequest`
- `WeatherTransitionState`
- `WindState`
- `PrecipitationState`
- `WeatherVisibility`
- `StormState`
- `WeatherFactors`

Visual adapter resources:

- `WeatherVisualsConfig`
- `WeatherVisualDiagnostics`
- `WeatherQuality`
- `WeatherScreenFxMode`
- `WeatherScreenFxSettings`
- `WeatherScreenState`

Components:

- `WeatherCamera`
- `WeatherCameraState`
- `WeatherCameraVisualState`
- `WeatherSurface`
- `WeatherSurfaceState`
- `WeatherSurfaceStandardMaterial`
- `WeatherZone`
- `WeatherOcclusionVolume`
- `WeatherVolumeShape`

System sets:

- `WeatherSystems`
- `WeatherVisualSystems`
- `WeatherSurfaceMaterialSystems`

Messages:

- `WeatherTransitionStarted`
- `WeatherTransitionFinished`
- `WeatherProfileChanged`
- `LightningFlashEmitted`

## Examples

| Example | Purpose |
|--------|---------|
| `basic` | Minimal clear-to-rain loop with one weather camera and the bundled visual adapters |
| `transitions` | Cycles through clear, foggy, rain, storm, and snow authored profiles |
| `windy_snow` | Shows a gust-heavy snow profile with stronger lateral drift |
| `localized_zones` | Demonstrates fog and storm pockets blending around the active camera |
| `camera_screen_fx` | Compares gameplay and cinematic cameras side by side |
| `shelter_and_occlusion` | Moves through a roofed shelter to suppress precipitation and screen cues |
| `saddle-world-weather-lab` | Crate-local BRP/E2E verification app with deterministic screenshot scenarios |

## Design Notes

- `WeatherPlugin` no longer owns rendering policy. It resolves weather state only.
- `WeatherCameraState` is the core signal surface for local weather. `WeatherCameraVisualState` is adapter output.
- `WeatherSurfaceState` stays in the core crate so gameplay and rendering can both consume wetness data.
- `WeatherSurfaceStandardMaterial` is explicitly adapter-side. It is safe to omit if another material system owns the response.
- Built-in precipitation remains camera-local and CPU-managed. The public API is structured so another adapter can replace it.

## Limitations

- No volumetric cloud or sky rendering.
- No GPU precipitation backend in the bundled adapter.
- The built-in material adapter only targets `MeshMaterial3d<StandardMaterial>`.

## Documentation

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/configuration.md`](docs/configuration.md)
- [`docs/debugging.md`](docs/debugging.md)
