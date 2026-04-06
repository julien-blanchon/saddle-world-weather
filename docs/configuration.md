# Configuration

## `WeatherConfig`

Core weather-state configuration.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `initial_profile` | `WeatherProfile` | `WeatherProfile::clear()` | Starting authored weather profile used on activation |
| `seed` | `u64` | `0x00C0FFEE_u64` | Drives deterministic gust sampling and lightning timing |
| `diagnostics_enabled` | `bool` | `true` | Enables non-essential diagnostics work in the bundled adapters |
| `default_transition_duration_secs` | `f32` | `4.0` | Default smooth transition duration for queued requests |
| `pending_request` | `Option<WeatherTransitionRequest>` | `None` | Internal command slot consumed by the request system |

Helpers:

- `queue_transition(profile, duration_secs)`
- `queue_immediate(profile)`

## `WeatherVisualsConfig`

Bundled camera-side presentation adapter configuration.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `quality` | `WeatherQuality` | `High` | Controls particle budget and whether built-in screen overlays are enabled |
| `screen_fx_mode` | `WeatherScreenFxMode` | `BuiltInOverlay` | Chooses whether the adapter spawns screen overlays or only publishes `WeatherCameraVisualState` |
| `screen_fx` | `WeatherScreenFxSettings` | sensible defaults | Global heuristic mapping from resolved weather state into overlay cues |

### `WeatherQuality`

| Variant | Max particles / camera | Screen overlays | Intended target |
|--------|-------------------------|-----------------|-----------------|
| `Low` | `48` | Off | lower-end desktop, mobile, conservative WASM |
| `Medium` | `120` | On | mid-range baseline |
| `High` | `220` | On | showcase and desktop default |

### `WeatherScreenFxMode`

| Variant | Effect |
|--------|--------|
| `BuiltInOverlay` | Spawns and updates the bundled overlay mesh/material for opted-in cameras |
| `StateOnly` | Skips overlay entities and only publishes resolved screen state through `WeatherCameraVisualState` |

### `WeatherScreenFxSettings`

`WeatherScreenFxSettings` is adapter-side, not authored per weather profile. It exposes global tuning for the bundled overlay response:

- `base_intensity`
- `rain_intensity`
- `snow_intensity`
- `fog_intensity`
- `storm_intensity`
- `droplet_intensity`
- `frost_intensity`
- `streak_intensity`
- `rain_tint`
- `snow_tint`
- `fog_tint`
- `storm_tint`

## `WeatherProfile`

`WeatherProfile` is the authored weather input surface used by the core solver.

### `PrecipitationProfile`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `label` | `Option<String>` | `None` | Optional debug/display label |
| `kind` | `PrecipitationKind` | `None` | Presentation mode hint for downstream adapters |
| `intensity` | `f32` | `0.0` | Main precipitation strength |
| `density` | `f32` | `0.0` | Density multiplier for local particle backends |
| `fall_speed` | `f32` | `0.0` | Downward particle speed hint |
| `particle_size` | `Vec2` | `Vec2(0.04, 0.7)` | Particle scale hint for local adapters |
| `wind_influence` | `f32` | `0.0` | How strongly wind displaces precipitation |
| `near_radius` | `f32` | `12.0` | Horizontal local presentation radius |
| `near_height` | `f32` | `10.0` | Vertical local presentation extent |
| `far_density` | `f32` | `0.0` | Atmospheric density hint for far-field weather feel |
| `tint` | `Color` | `Color::WHITE` | Particle tint hint for local adapters |

### `FogProfile`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `color` | `Color` | neutral grey-blue | Base fog tint |
| `density` | `f32` | `0.04` | Overall fog thickness |
| `visibility_distance` | `f32` | `220.0` | Visibility hint and default fog shaping distance |
| `volumetric_intensity` | `f32` | `0.0` | Participating-media hint for downstream systems |

### `WindProfile`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `direction` | `Vec3` | `(0.6, 0.0, 0.2)` normalized | Base wind direction; `Y` is forced to `0.0` during clamping |
| `base_speed` | `f32` | `2.0` | Baseline wind speed |
| `gust_amplitude` | `f32` | `0.15` | Strength of gust modulation |
| `gust_frequency_hz` | `f32` | `0.22` | Gust cadence |
| `sway` | `f32` | `0.2` | Extra downstream motion hint |

### `StormProfile`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `intensity` | `f32` | `0.0` | Storm severity factor |
| `lightning_frequency_hz` | `f32` | `0.0` | Average lightning opportunity frequency |
| `lightning_duration_secs` | `f32` | `0.12` | Flash duration |
| `lightning_brightness` | `f32` | `0.0` | Brightness multiplier for the resolved flash cue |
| `wetness_bonus` | `f32` | `0.0` | Extra wetness added on top of precipitation |

## Camera Authoring

### `WeatherCamera`

`WeatherCamera` is the sampling point for local weather resolution. Some fields are consumed only by the bundled visual adapter.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `true` | Enables all crate processing for that camera |
| `priority` | `i32` | `0` | Chooses which camera populates “primary” diagnostics |
| `receive_precipitation` | `bool` | `true` | Bundled visual adapter: enables local precipitation emitters |
| `receive_screen_fx` | `bool` | `true` | Bundled visual adapter: enables screen overlay cues |
| `apply_distance_fog` | `bool` | `true` | Bundled visual adapter: allows `DistanceFog` synchronization |
| `insert_missing_components` | `bool` | `true` | Bundled visual adapter: inserts `DistanceFog` if missing |
| `quality_bias` | `f32` | `1.0` | Bundled visual adapter: scales particle count relative to quality budget |
| `precipitation_blocked_factor` | `f32` | `0.0` | Additional local suppression before authored occlusion volumes are applied |

### `WeatherCameraState`

Core per-camera weather state:

- base and resolved profile labels
- dominant zone label
- precipitation kind, factor, density, fall speed, particle sizing hints
- precipitation and screen occlusion multipliers
- wetness factor
- fog density, fog color, visibility distance
- resolved wind vector
- lightning flash intensity

### `WeatherCameraVisualState`

Bundled visual adapter output:

- `screen: WeatherScreenState`
- `active_particles: usize`

## Surface Authoring

### `WeatherSurface`

Core accumulation settings. These control how fast wetness, puddles, and snow coverage react to resolved weather.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `true` | Enables surface weather accumulation |
| `wetness_response` | `f32` | `1.0` | Scales resolved wetness input |
| `puddle_response` | `f32` | `0.85` | Scales puddle growth from rain/storm input |
| `snow_response` | `f32` | `1.0` | Scales snow accumulation from snow weather input |
| `wetting_speed` | `f32` | `0.55` | Speed at which wetness rises |
| `drying_speed` | `f32` | `0.08` | Speed at which wetness fades |
| `puddle_fill_speed` | `f32` | `0.30` | Speed at which puddles accumulate |
| `puddle_drain_speed` | `f32` | `0.06` | Speed at which puddles recede |
| `snow_accumulation_speed` | `f32` | `0.22` | Speed at which snow coverage builds |
| `snow_melt_speed` | `f32` | `0.10` | Speed at which snow coverage fades |
| `puddle_threshold` | `f32` | `0.35` | Minimum rain factor before puddles start accumulating |
| `max_puddle_coverage` | `f32` | `0.7` | Upper cap for puddle coverage |
| `max_snow_coverage` | `f32` | `1.0` | Upper cap for snow coverage |

### `WeatherSurfaceStandardMaterial`

Bundled `StandardMaterial` response adapter.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `true` | Enables material modulation |
| `wet_roughness` | `f32` | `0.18` | Roughness target used by wetness |
| `puddle_roughness` | `f32` | `0.04` | Roughness target used by puddled areas |
| `snow_roughness` | `f32` | `0.92` | Roughness target used by snow-covered surfaces |
| `wet_reflectance` | `f32` | `0.34` | Reflectance target as a surface becomes wet |
| `puddle_reflectance` | `f32` | `0.52` | Reflectance target as puddle coverage increases |
| `snow_reflectance` | `f32` | `0.16` | Reflectance target as snow coverage increases |
| `wet_darkening` | `f32` | `0.18` | Base-color darkening from wetness |
| `puddle_darkening` | `f32` | `0.28` | Additional darkening from puddles |
| `snow_tint` | `Color` | pale snow white-blue | Tint blended into the material as snow coverage rises |

### `WeatherSurfaceState`

Resolved per-entity accumulation record published by the core plugin.

- `base_profile_label`
- `resolved_profile_label`
- `zone_label`
- `precipitation_kind`
- `rain_factor`
- `snow_factor`
- `wetness_factor`
- `wetness`
- `puddle_coverage`
- `snow_coverage`

## Local Overrides

### `WeatherZone`

Priority-weighted local weather override volume with fields:

- `label`
- `enabled`
- `profile`
- `shape`
- `blend_distance`
- `priority`
- `weight`

### `WeatherOcclusionVolume`

Local shelter / suppression volume with fields:

- `label`
- `enabled`
- `shape`
- `blend_distance`
- `precipitation_multiplier`
- `screen_fx_multiplier`
