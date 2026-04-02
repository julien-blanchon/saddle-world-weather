# Architecture

## Authored Model

The authored surface is `WeatherProfile`. It is intentionally composable rather than a closed master enum.

Each profile owns five authored subdomains:

- `PrecipitationProfile`: kind, density, fall speed, wind influence, near-camera bounds, far density, tint
- `FogProfile`: fog color, density, visibility distance, volumetric hint
- `WindProfile`: direction, base speed, gust amplitude, gust frequency, sway hint
- `ScreenFxProfile`: overlay intensity, droplets, frost, streaks, tint
- `StormProfile`: storm intensity, lightning frequency, flash duration, brightness, wetness bonus

Helper constructors such as `WeatherProfile::clear()`, `rain()`, `snow()`, `foggy()`, and `storm()` are convenience presets only. Runtime logic reads fields, not a fixed enum.

## Runtime Model

`WeatherRuntime` is the resolved global weather state after transition blending and seeded sampling. It contains:

- `active_profile`: the currently blended authored profile
- `target_profile`: the authored target profile
- `transition`: progress, elapsed time, labels, active flag
- `wind`: resolved direction, gust factor, final vector
- `precipitation`: resolved kind, intensity, density, fall speed, presentation bounds
- `visibility`: fog density, visibility distance, classification, fog color
- `screen_fx`: resolved overlay and cue intensity
- `storm`: resolved lightning state and current flash id
- `factors`: normalized reaction hooks for rain, snow, fog, storm, wind, wetness, and screen FX

`WeatherCameraState` is the per-camera resolved state after local zones and occlusion volumes are applied. It keeps both:

- `base_profile_label`: the global transitioned label before camera-local overrides
- `resolved_profile_label`: the actual authored label after zone blending on that camera

This makes BRP inspection clearer when the global weather is calm but one camera is inside a storm pocket or shelter.

## Solver Split

The crate keeps the core solver mostly pure Rust:

- `profiles.rs`: presets, clamping, interpolation rules
- `solver.rs`: gust sampling, lightning scheduling, profile resolution, zone blending, occlusion blending

The Bevy integration layer lives in:

- `systems.rs`: activation, transitions, runtime resolution, camera resolution, fog sync, diagnostics, message emission
- `visuals.rs`: precipitation emitter management and optional screen overlays

The emitter presentation consumes `WeatherCameraState.wind_influence`, so authored precipitation wind response changes the actual visual drift instead of staying metadata-only.

This split keeps the hard logic testable without a full `App`.

## System Ordering

The public `WeatherSystems` order is fixed and chained:

1. `ApplyRequests`
2. `AdvanceTransition`
3. `ResolveBaseState`
4. `ResolveCameraState`
5. `SyncEmitters`
6. `SyncFog`
7. `SyncScreenEffects`
8. `EmitMessages`
9. `Diagnostics`

Implications:

- profile transition requests are consumed before the frame advances
- global weather is resolved before any camera-local weather state
- precipitation and fog consume camera-local state, not raw authored profiles
- messages fire after runtime resolution, so downstream readers see the resolved state in the same frame

## Zone Blending Rules

`WeatherZone` uses:

- `priority`: higher priority wins over lower priority when zones overlap
- `weight`: relative blend strength within the winning priority band
- `shape + blend_distance`: geometric influence falloff

Resolution flow:

1. Gather every zone affecting the camera position.
2. Find the highest active priority.
3. Blend only zones in that winning band, weighted by `weight * influence`.
4. Blend the result against the base global profile.

This keeps the result deterministic and avoids low-priority fog pockets diluting an intentional high-priority storm cell.

## Shelter / Occlusion Rules

`WeatherOcclusionVolume` is evaluated per camera after zone resolution.

- `precipitation_multiplier` attenuates precipitation presentation
- `screen_fx_multiplier` attenuates screen-space cues
- the strongest suppression wins as influence increases

This is intentionally generic. The crate does not define “indoors” or “under a roof” semantically; it only resolves authored suppression volumes and camera-local blocking factors.

## Precipitation Strategy

The crate uses a production-style illusion:

- precipitation is spawned in a local volume around each opted-in camera
- particle transforms are deterministic from seed plus elapsed time
- the emitter root tracks the active camera instead of simulating world-scale weather coverage
- far-field “weather scale” is represented by fog, visibility, and density hints rather than infinitely large particle swarms

This keeps the default implementation portable and cheap while preserving a backend-private public API.

## Fog Strategy

The crate does not replace Bevy fog. It drives per-camera `DistanceFog` for cameras with `WeatherCamera::apply_distance_fog = true`.

This gives downstream apps a useful baseline that composes well with:

- scene lighting
- atmosphere systems
- additional volumetric rendering from other crates

## Quality Model

`WeatherQuality` currently controls:

- max particles per camera
- whether screen-space overlays are enabled
- overlay texture resolution

Why this model:

- it gives a clear mobile / WASM / lower-end fallback path
- it preserves the same authored profiles across quality tiers
- it keeps the public API stable if a future GPU precipitation backend is added

## Lightning Scope

Lightning currently resolves as:

- deterministic flash scheduling in the solver
- `LightningFlashEmitted` messages
- per-camera `lightning_flash_intensity`
- an optional screen-space flash cue on cameras that allow screen effects

This is intentionally smaller than a full sky-lightning system. Audio timing, cloud illumination, and bespoke scene-light modulation remain downstream responsibilities.
