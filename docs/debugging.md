# Debugging

## Diagnostics Resource

`WeatherDiagnostics` is the main high-level summary.

Important fields:

- `active_profile_label`: resolved global profile label
- `target_profile_label`: current transition target label
- `quality`: active quality tier used for particle and screen-FX budgeting
- `transition_progress`: `0.0..=1.0`
- `transition_active`: whether a transition is still running
- `active_emitters`: number of live precipitation emitter roots
- `precipitation_particles_estimate`: summed active particle count across weather cameras
- `active_zone_count`: number of currently contributing zones on the primary camera
- `current_wind`: resolved global wind vector
- `current_fog_density`: resolved global fog density after authored blending
- `current_visibility_distance`: resolved global visibility hint
- `current_precipitation_kind`: resolved global precipitation mode
- `primary_camera_name`: highest-priority active weather camera
- `primary_zone_label`: dominant local zone on that camera
- `managed_screen_overlays`: number of live overlay entities
- `last_transition_started_at`: elapsed solver time of the last smooth transition start
- `last_transition_finished_at`: elapsed solver time of the last completion
- `last_lightning_flash_id`: last deterministic lightning flash id emitted
- `transition_started_count`, `transition_finished_count`, `profile_changed_count`, `lightning_flash_count`: cumulative crate-level message counts since activation

For camera-specific inspection, query `WeatherCameraState` on the camera entity rather than relying only on `WeatherDiagnostics`.

Key `WeatherCameraState` fields for local debugging:

- `base_profile_label`: global transitioned label before local overrides
- `resolved_profile_label`: actual label after zone blending for that camera
- `zone_label`: dominant contributing zone
- `wind_influence`: authored precipitation wind response used by the local emitter
- `far_density`: far-field atmospheric density hint for that camera

## BRP Workflow

Launch the lab:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-weather-lab
```

Useful queries:

```bash
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherRuntime
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get saddle_world_weather::resources::WeatherConfig
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherCamera
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherCameraState
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherZone
uv run --project .codex/skills/bevy-brp/script brp world query saddle_world_weather::WeatherOcclusionVolume
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/weather_debug.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

Type paths to remember:

- `saddle_world_weather::resources::WeatherRuntime`
- `saddle_world_weather::resources::WeatherDiagnostics`
- `saddle_world_weather::resources::WeatherConfig`
- `saddle_world_weather::WeatherCamera`
- `saddle_world_weather::WeatherCameraState`
- `saddle_world_weather::WeatherZone`
- `saddle_world_weather::WeatherOcclusionVolume`

## Common Failure Modes

### Missing precipitation

Check:

- the camera has `WeatherCamera`
- `receive_precipitation = true`
- the resolved `WeatherCameraState.precipitation_factor > 0.0`
- `WeatherDiagnostics.active_emitters > 0`
- quality is not set to a tiny budget by `WeatherQuality` plus camera `quality_bias`
- a shelter volume or `precipitation_blocked_factor` is not suppressing the camera

### Fog looks wrong or too dense

Check:

- `WeatherCamera.apply_distance_fog`
- `WeatherCamera.insert_missing_components`
- `WeatherCameraState.fog_density`
- `WeatherCameraState.visibility_distance`
- whether a local `WeatherZone` is overriding the global profile on the active camera

If the fog reads heavier than expected, inspect the active local zone first. The per-camera state is authoritative.

### Screen FX are too intrusive

Check:

- `WeatherCamera.receive_screen_fx`
- `WeatherQuality` because `Low` disables screen FX
- `WeatherCameraState.screen_fx_factor`
- whether the camera is inside a `WeatherOcclusionVolume`

For gameplay cameras, disabling `receive_screen_fx` is the intended coarse control.

### Transitions pop or never finish

Check:

- that you are using `queue_transition` instead of repeatedly overwriting `pending_request` every frame
- `WeatherRuntime.transition.active`
- `WeatherRuntime.transition.progress`
- `WeatherDiagnostics.last_transition_started_at`
- `WeatherDiagnostics.last_transition_finished_at`

If transitions are not deterministic in tests, verify the app is stepping time manually and that the seed remains fixed.

### Lightning messages arrive but visuals do not change enough

Current scope:

- lightning drives `LightningFlashEmitted`
- the active camera state exposes `lightning_flash_intensity`
- cameras that allow screen FX receive a brief overlay flash

The crate does not currently modulate world lights or clouds. If a downstream app needs stronger scene-wide flashes, read `lightning_flash_intensity` or `LightningFlashEmitted` and layer its own lighting response.
