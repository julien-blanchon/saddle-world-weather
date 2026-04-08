# Weather Lab

Crate-local lab app for validating the shared `weather` crate in a real Bevy application.

## Purpose

- verify authored profiles, transitions, camera-local precipitation, fog sync, local zones, shelter suppression, and diagnostics in one scene
- keep deterministic screenshot checkpoints for smoke, transition gallery, windy snow, localized zones, camera screen-fx splits, shelter occlusion, storm flashes, and quality scaling
- expose weather runtime, camera state, message counts, and particle estimates through a readable overlay for BRP and E2E inspection

## Status

Working

## Run

```bash
cargo run -p saddle-world-weather-lab
```

## E2E

```bash
cargo run -p saddle-world-weather-lab --features e2e -- weather_smoke
cargo run -p saddle-world-weather-lab --features e2e -- weather_transition_gallery
cargo run -p saddle-world-weather-lab --features e2e -- weather_windy_snow
cargo run -p saddle-world-weather-lab --features e2e -- weather_localized_zones
cargo run -p saddle-world-weather-lab --features e2e -- weather_camera_screen_fx
cargo run -p saddle-world-weather-lab --features e2e -- weather_shelter_occlusion
cargo run -p saddle-world-weather-lab --features e2e -- weather_storm_flash
cargo run -p saddle-world-weather-lab --features e2e -- weather_quality_compare
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-world-weather-lab
uv run --project .codex/skills/bevy-brp/script brp resource get weather::WeatherRuntime
uv run --project .codex/skills/bevy-brp/script brp resource get weather::WeatherDiagnostics
uv run --project .codex/skills/bevy-brp/script brp world query weather::WeatherCameraState
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle-world-weather-lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Notes

- The lab keeps one named primary camera, stable zone names, and a named shelter occlusion volume so BRP and E2E can target deterministic entities.
- The scene is entirely procedural, so the shared crate remains self-contained and does not need project assets.
- Lightning is represented as a deterministic screen-space flash cue plus messages rather than a full audio or sky-lightning system.
