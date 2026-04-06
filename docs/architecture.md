# Architecture

## Core vs Adapters

The crate is split into one core weather-state plugin and two bundled adapters:

- `WeatherPlugin`: authored profile blending, deterministic transitions, zone blending, shelter suppression, runtime weather state, camera weather state, surface accumulation state, and message emission
- `WeatherVisualsPlugin`: camera-local precipitation emitters, `DistanceFog` sync, screen overlay cues, and visual diagnostics
- `WeatherSurfaceMaterialsPlugin`: `StandardMaterial` mutation driven by `WeatherSurfaceState`

This is the main architectural boundary:

- core owns weather meaning
- adapters own rendering policy

If another project wants GPU precipitation, a custom post-process stack, or a different material system, it can skip the bundled adapters and consume the core state directly.

## Authored Model

`WeatherProfile` is the authored input surface for the core solver. It owns four authored subdomains:

- `PrecipitationProfile`
- `FogProfile`
- `WindProfile`
- `StormProfile`

Notably absent: screen-space FX and material response. Those moved out of the authored weather profile so the weather engine no longer dictates lens-cue or shader policy.

## Runtime Model

`WeatherRuntime` is the resolved global weather state after transition blending and seeded sampling. It contains:

- `active_profile`
- `target_profile`
- `transition`
- `wind`
- `precipitation`
- `visibility`
- `storm`
- `factors`

`WeatherCameraState` is the per-camera resolved core state after local zones and occlusion volumes are applied. It keeps both:

- `base_profile_label`
- `resolved_profile_label`

and also exposes:

- precipitation hints for local rendering backends
- precipitation and screen occlusion multipliers
- fog density / color / visibility
- wetness factor
- wind vector
- lightning flash intensity

`WeatherSurfaceState` is the per-surface accumulation record. It stays in the core crate so gameplay systems and render adapters can both read the same wetness / puddle / snow data.

## Adapter Model

### Visual Adapter

`WeatherVisualsPlugin` consumes `WeatherCameraState` and publishes `WeatherCameraVisualState`.

It owns:

- `WeatherVisualsConfig`
- `WeatherVisualDiagnostics`
- precipitation emitter entities
- screen overlay entities
- `DistanceFog` synchronization

The adapter computes a bundled screen response from the core weather state using `WeatherScreenFxSettings`. That keeps screen cues configurable without putting them back into `WeatherProfile`.

### StandardMaterial Adapter

`WeatherSurfaceMaterialsPlugin` consumes:

- `WeatherSurfaceState`
- `WeatherSurfaceStandardMaterial`
- `MeshMaterial3d<StandardMaterial>`

On first sync it clones the referenced `StandardMaterial` handle so weather response stays entity-local instead of mutating shared authored assets in place.

## Solver Split

Pure Rust logic stays isolated:

- `profiles.rs`: presets, clamping, interpolation
- `solver.rs`: gust sampling, lightning scheduling, profile resolution, zone blending, occlusion blending

Bevy integration stays thin:

- `systems.rs`: activation, transitions, runtime resolution, camera resolution, diagnostics, messages
- `surfaces.rs`: surface accumulation state only
- `visuals.rs`: bundled camera-side rendering adapter
- `surface_materials.rs`: bundled `StandardMaterial` adapter

## System Ordering

Core ordering:

1. `WeatherSystems::ApplyRequests`
2. `WeatherSystems::AdvanceTransition`
3. `WeatherSystems::ResolveBaseState`
4. `WeatherSystems::SyncSurfaces`
5. `WeatherSystems::ResolveCameraState`
6. `WeatherSystems::EmitMessages`
7. `WeatherSystems::Diagnostics`

Bundled visual adapter ordering:

1. `WeatherVisualSystems::ResolveState`
2. `WeatherVisualSystems::SyncEmitters`
3. `WeatherVisualSystems::SyncFog`
4. `WeatherVisualSystems::SyncScreenEffects`
5. `WeatherVisualSystems::Diagnostics`

`WeatherVisualSystems::ResolveState` is ordered after `WeatherSystems::ResolveCameraState`, so the visual adapter always reads final local weather state.

The material adapter runs in `WeatherSurfaceMaterialSystems::ApplyMaterials` after `WeatherSystems::SyncSurfaces`.

## Zone Blending Rules

`WeatherZone` resolution is deterministic:

1. Gather every zone affecting the sample position.
2. Find the highest active priority.
3. Blend only zones in that winning priority band, weighted by `weight * influence`.
4. Blend the result against the base global profile.

This keeps low-priority ambience from diluting an intentional high-priority storm pocket.

## Shelter Rules

`WeatherOcclusionVolume` is evaluated per camera after zone resolution.

- `precipitation_multiplier` attenuates precipitation presentation
- `screen_fx_multiplier` attenuates bundled screen cues
- the strongest suppression wins as influence increases

The core crate still resolves both multipliers because they are weather-exposure signals. Only the actual fog, particle, and overlay rendering lives in the adapter.

## Precipitation Strategy

The bundled visual adapter uses a camera-local illusion:

- particles are spawned in a bounded volume near the active camera
- transforms are deterministic from seed plus elapsed time
- the emitter root follows the camera instead of simulating map-wide coverage
- far-field weather scale is carried mostly by fog, visibility, and density hints

This keeps the bundled implementation cheap while leaving room for a future GPU or world-scale adapter.

## Extensibility

The intended integration patterns are:

- core only: use `WeatherPlugin` and consume `WeatherRuntime`, `WeatherCameraState`, and `WeatherSurfaceState`
- core + visuals: add `WeatherVisualsPlugin`
- core + material bridge: add `WeatherSurfaceMaterialsPlugin`
- full bundled stack: add all three plugins

That composition model is the key design change in this crate revision.
