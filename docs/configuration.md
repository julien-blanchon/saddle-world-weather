# Configuration

## `WeatherConfig`

| Field | Type | Default | Valid range | Effect |
|------|------|---------|-------------|--------|
| `initial_profile` | `WeatherProfile` | `WeatherProfile::clear()` | Any clamped profile | Starting authored weather profile used on activation |
| `quality` | `WeatherQuality` | `High` | `Low`, `Medium`, `High` | Controls particle budgets, screen-FX participation, and overlay texture resolution |
| `seed` | `u64` | `0x00C0FFEE_u64` | Any `u64` | Drives deterministic gust sampling and lightning timing |
| `diagnostics_enabled` | `bool` | `true` | `true` or `false` | Intended switch for downstream diagnostic cost and verbosity policy |
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
