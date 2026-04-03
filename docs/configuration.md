# Configuration

## `WeatherConfig`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `initial_profile` | `WeatherProfile` | `WeatherProfile::clear()` | Any clamped profile | Starting authored weather profile used on activation |
| `quality` | `WeatherQuality` | `High` | `Low`, `Medium`, `High` | Controls particle budgets, screen-FX participation, and overlay texture resolution |
| `seed` | `u64` | `0x00C0FFEE_u64` | Any `u64` | Drives deterministic gust sampling and lightning timing |
| `diagnostics_enabled` | `bool` | `true` | `true` or `false` | Intended switch for downstream diagnostic cost and verbosity policy |
| `screen_fx_mode` | `WeatherScreenFxMode` | `BuiltInOverlay` | `BuiltInOverlay`, `StateOnly` | Chooses whether the crate renders its own full-screen overlay cues or only publishes resolved screen-FX state for downstream consumers |
| `default_transition_duration_secs` | `f32` | `4.0` | `>= 0.0` recommended | Default smooth transition duration for downstream callers that do not provide one |
| `pending_request` | `Option<WeatherTransitionRequest>` | `None` | `None` or one request | Internal command slot consumed by the request application system |

`WeatherConfig` helpers:

- `queue_transition(profile, duration_secs)`: request a smooth transition
- `queue_immediate(profile)`: request an immediate switch

## `WeatherQuality`

| Variant | Max particles / camera | Screen FX | Overlay resolution | Intended target |
|--------|-------------------------|-----------|--------------------|-----------------|
| `Low` | `48` | Off | `64` | lower-end desktop, mobile, conservative WASM |
| `Medium` | `120` | On | `96` | mid-range baseline |
| `High` | `220` | On | `128` | showcase and desktop default |

## `WeatherScreenFxMode`

| Variant | Effect | Best for |
|--------|--------|----------|
| `BuiltInOverlay` | Spawns and updates the crate's lightweight overlay sprites on opted-in cameras | Standalone demos and apps that want weather lens cues without another post-process crate |
| `StateOnly` | Disables built-in overlay entities but still resolves `WeatherCameraState.screen_fx_factor`, tint, droplets, frost, and lightning cues | Integrations that want to route weather into another screen-effects pipeline |

## `WeatherProfile`

`WeatherProfile` is the authored input surface. All fields are clamped before runtime use.

### `PrecipitationProfile`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `label` | `Option<String>` | `None` | Any | Optional debug/display label for the precipitation subprofile |
| `kind` | `PrecipitationKind` | `None` | `None`, `Rain`, `Snow`, `Particulate` | Presentation mode used by emitters |
| `intensity` | `f32` | `0.0` | `0.0..=1.0` | Main precipitation strength |
| `density` | `f32` | `0.0` | `0.0..=1.0` | Density multiplier for local particle counts |
| `fall_speed` | `f32` | `0.0` | `>= 0.0` | Downward particle speed |
| `particle_size` | `Vec2` | `Vec2(0.04, 0.7)` | `>= 0.01` each axis | Mesh scale for local precipitation particles |
| `wind_influence` | `f32` | `0.0` | `0.0..=2.0` | How strongly wind displaces precipitation |
| `near_radius` | `f32` | `12.0` | `>= 1.0` | Horizontal emitter radius around camera |
| `near_height` | `f32` | `10.0` | `>= 1.0` | Vertical emitter extent around camera |
| `far_density` | `f32` | `0.0` | `0.0..=1.0` | Atmospheric density hint for far-field weather feel |
| `tint` | `Color` | `Color::WHITE` | Any color | Particle color and alpha tint |

### `FogProfile`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `color` | `Color` | neutral grey-blue | Any color | Base fog tint |
| `density` | `f32` | `0.04` | `0.0..=1.0` | Overall fog thickness |
| `visibility_distance` | `f32` | `220.0` | `>= 5.0` | Distance used for visibility hints and `DistanceFog` shaping |
| `volumetric_intensity` | `f32` | `0.0` | `0.0..=1.0` | Extra participating-media hint for downstream systems |

### `WindProfile`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `direction` | `Vec3` | `(0.6, 0.0, 0.2)` normalized | non-zero horizontal vector | Base wind direction; Y is forced to `0.0` during clamping |
| `base_speed` | `f32` | `2.0` | `>= 0.0` | Baseline wind speed |
| `gust_amplitude` | `f32` | `0.15` | `0.0..=1.0` | Strength of gust modulation |
| `gust_frequency_hz` | `f32` | `0.22` | `>= 0.0` | Gust cadence |
| `sway` | `f32` | `0.2` | `0.0..=2.0` | Extra downstream motion hint |

### `ScreenFxProfile`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `intensity` | `f32` | `0.0` | `0.0..=1.0` | Master screen-space cue intensity |
| `droplet_intensity` | `f32` | `0.0` | `0.0..=1.0` | Lens droplet contribution hint |
| `frost_intensity` | `f32` | `0.0` | `0.0..=1.0` | Frost cue hint |
| `streak_intensity` | `f32` | `0.0` | `0.0..=1.0` | Streaking cue hint |
| `tint` | `Color` | `Color::WHITE` | Any color | Overlay tint |

### `StormProfile`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `intensity` | `f32` | `0.0` | `0.0..=1.0` | Storm severity factor |
| `lightning_frequency_hz` | `f32` | `0.0` | `>= 0.0` | Average lightning opportunity frequency |
| `lightning_duration_secs` | `f32` | `0.12` | `0.02..=1.0` | Flash duration |
| `lightning_brightness` | `f32` | `0.0` | `>= 0.0` | Brightness multiplier for the flash cue |
| `wetness_bonus` | `f32` | `0.0` | `0.0..=1.0` | Extra wetness added on top of precipitation |

## Camera Authoring

### `WeatherCamera`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `true` | Enables all crate processing for that camera |
| `priority` | `i32` | `0` | Chooses which camera populates “primary” diagnostics when several are active |
| `receive_precipitation` | `bool` | `true` | Enables local precipitation emitters |
| `receive_screen_fx` | `bool` | `true` | Enables screen overlay cues and lightning flash cue |
| `apply_distance_fog` | `bool` | `true` | Allows `DistanceFog` synchronization |
| `insert_missing_components` | `bool` | `true` | Allows the crate to insert `DistanceFog` when missing |
| `quality_bias` | `f32` | `1.0` | Scales particle count relative to the global quality budget |
| `precipitation_blocked_factor` | `f32` | `0.0` | Per-camera suppression before authored occlusion volumes are applied |

## Surface Authoring

`WeatherSurface` is an opt-in bridge for `StandardMaterial` scene materials. On first sync, the crate clones the referenced material handle so weather modulation stays local to that entity instead of mutating a shared authoring material in place.

### `WeatherSurface`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `enabled` | `bool` | `true` | Enables surface weather accumulation and material modulation |
| `wetness_response` | `f32` | `1.0` | Scales how strongly resolved wetness influences this surface |
| `puddle_response` | `f32` | `0.85` | Scales puddle growth from rain/storm input |
| `snow_response` | `f32` | `1.0` | Scales snow accumulation from snow weather input |
| `wetting_speed` | `f32` | `0.55` | Speed at which wetness rises under rain/storm conditions |
| `drying_speed` | `f32` | `0.08` | Speed at which wetness fades when the weather clears |
| `puddle_fill_speed` | `f32` | `0.30` | Speed at which puddles accumulate once rain is strong enough |
| `puddle_drain_speed` | `f32` | `0.06` | Speed at which puddles recede after rain stops |
| `snow_accumulation_speed` | `f32` | `0.22` | Speed at which snow coverage builds during snowy weather |
| `snow_melt_speed` | `f32` | `0.10` | Speed at which snow coverage fades outside snowy weather |
| `puddle_threshold` | `f32` | `0.35` | Minimum rain factor needed before puddles start accumulating |
| `max_puddle_coverage` | `f32` | `0.7` | Upper cap for puddle coverage |
| `max_snow_coverage` | `f32` | `1.0` | Upper cap for snow coverage |
| `wet_roughness` | `f32` | `0.18` | Roughness target used by general wetness darkening |
| `puddle_roughness` | `f32` | `0.04` | Roughness target used by puddled areas |
| `snow_roughness` | `f32` | `0.92` | Roughness target used by snow-covered surfaces |
| `wet_reflectance` | `f32` | `0.34` | Reflectance target applied as a surface becomes wet |
| `puddle_reflectance` | `f32` | `0.52` | Reflectance target applied as puddle coverage increases |
| `snow_reflectance` | `f32` | `0.16` | Reflectance target applied as snow coverage increases |
| `wet_darkening` | `f32` | `0.18` | Amount of base-color darkening applied by wetness |
| `puddle_darkening` | `f32` | `0.28` | Amount of additional base-color darkening applied by puddles |
| `snow_tint` | `Color` | pale snow white-blue | Tint blended into the material as snow coverage rises |

### `WeatherSurfaceState`

`WeatherSurfaceState` is the resolved per-entity accumulation record. It is updated by the crate and can be inspected over BRP, tests, or custom gameplay systems.

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `base_profile_label` | `Option<String>` | `None` | Global transitioned weather label before zone overrides at the surface position |
| `resolved_profile_label` | `Option<String>` | `None` | Final authored label after zone blending at the surface position |
| `zone_label` | `Option<String>` | `None` | Most relevant zone label contributing to the surface weather |
| `precipitation_kind` | `PrecipitationKind` | `None` | Active precipitation kind at the surface position |
| `rain_factor` | `f32` | `0.0` | Normalized rain intensity used for wetness and puddle accumulation |
| `snow_factor` | `f32` | `0.0` | Normalized snow intensity used for snow accumulation |
| `wetness_factor` | `f32` | `0.0` | Resolved wetness input before per-surface response scaling |
| `wetness` | `f32` | `0.0` | Smoothed wetness accumulation after response and speed settings |
| `puddle_coverage` | `f32` | `0.0` | Smoothed puddle coverage |
| `snow_coverage` | `f32` | `0.0` | Smoothed snow coverage |

## Local Overrides

### `WeatherZone`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `label` | `Option<String>` | `"Weather Zone"` | Debug and diagnostics label |
| `enabled` | `bool` | `true` | Enables the zone |
| `profile` | `WeatherProfile` | `foggy()` | Authored local weather override |
| `shape` | `WeatherVolumeShape` | sphere radius `8.0` | Zone volume |
| `blend_distance` | `f32` | `6.0` | Feather distance outside the volume |
| `priority` | `i32` | `0` | Higher values win against lower-priority overlaps |
| `weight` | `f32` | `1.0` | Relative blend strength within the winning priority band |

### `WeatherOcclusionVolume`

| Field | Type | Default | Effect |
|------|------|---------|--------|
| `label` | `Option<String>` | `"Weather Occlusion"` | Debug and diagnostics label |
| `enabled` | `bool` | `true` | Enables the volume |
| `shape` | `WeatherVolumeShape` | sphere radius `8.0` | Occlusion volume |
| `blend_distance` | `f32` | `3.0` | Feather distance outside the volume |
| `precipitation_multiplier` | `f32` | `0.0` | Scales precipitation presentation under shelter |
| `screen_fx_multiplier` | `f32` | `0.0` | Scales screen-space weather cues under shelter |
