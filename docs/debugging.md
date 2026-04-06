# Debugging

## Core Diagnostics

`WeatherDiagnostics` summarizes the core solver state.

Important fields:

- `active_profile_label`
- `target_profile_label`
- `transition_progress`
- `transition_active`
- `active_zone_count`
- `current_wind`
- `current_fog_density`
- `current_visibility_distance`
- `current_precipitation_kind`
- `primary_camera_name`
- `primary_zone_label`
- `last_transition_started_at`
- `last_transition_finished_at`
- `last_lightning_flash_id`
- `transition_started_count`
- `transition_finished_count`
- `profile_changed_count`
- `lightning_flash_count`

Use `WeatherCameraState` when you need local weather rather than the global summary.

## Visual Diagnostics

`WeatherVisualDiagnostics` belongs to the bundled visual adapter.

Important fields:

- `quality`
- `active_emitters`
- `precipitation_particles_estimate`
- `managed_screen_overlays`

Use `WeatherCameraVisualState` for per-camera bundled visual state such as overlay intensity and active particle count.

## BRP Workflow

Launch the lab:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-weather-lab
```

Useful queries:

```bash
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherRuntime
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherVisualDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherConfig
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherVisualsConfig
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherCamera
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherCameraState
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherCameraVisualState
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherSurfaceState
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherZone
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherOcclusionVolume
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/weather_debug.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Common Failure Modes

### Missing precipitation

Check:

- the camera has `WeatherCamera`
- `WeatherCamera.receive_precipitation = true`
- `WeatherCameraState.precipitation_factor > 0.0`
- `WeatherVisualDiagnostics.active_emitters > 0`
- `WeatherVisualsConfig.quality` plus camera `quality_bias`
- a shelter volume or `precipitation_blocked_factor` is not suppressing the camera

### Fog looks wrong

Check:

- `WeatherCamera.apply_distance_fog`
- `WeatherCamera.insert_missing_components`
- `WeatherCameraState.fog_density`
- `WeatherCameraState.visibility_distance`
- whether a local `WeatherZone` is overriding the global profile

### Screen overlays are too intrusive

Check:

- `WeatherCamera.receive_screen_fx`
- `WeatherVisualsConfig.quality`
- `WeatherVisualsConfig.screen_fx_mode`
- `WeatherCameraVisualState.screen.overlay_intensity`
- whether the camera is inside a `WeatherOcclusionVolume`

### Material response is missing

Check:

- the entity has `WeatherSurface`
- the entity has `WeatherSurfaceStandardMaterial`
- the entity has `MeshMaterial3d<StandardMaterial>`
- `WeatherSurfaceState` is being published

### Transitions pop or never finish

Check:

- that you are using `queue_transition` instead of rewriting `pending_request` every frame
- `WeatherRuntime.transition.active`
- `WeatherRuntime.transition.progress`
- `WeatherDiagnostics.last_transition_started_at`
- `WeatherDiagnostics.last_transition_finished_at`

### Lightning messages arrive but the scene still looks calm

Current scope:

- lightning drives `LightningFlashEmitted`
- `WeatherCameraState.lightning_flash_intensity` exposes the resolved flash
- the bundled visual adapter may turn that into a brief overlay cue

If a downstream app needs stronger whole-scene flashes, read the core lightning signal and layer its own light or atmosphere response.
