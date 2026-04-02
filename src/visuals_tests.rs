use super::*;

fn rain_camera_state() -> WeatherCameraState {
    WeatherCameraState {
        precipitation_kind: PrecipitationKind::Rain,
        precipitation_factor: 1.0,
        precipitation_density: 1.0,
        fall_speed: 16.0,
        particle_size: Vec2::new(0.04, 0.8),
        wind_influence: 0.0,
        near_radius: 12.0,
        near_height: 10.0,
        far_density: 0.35,
        wind_vector: Vec3::new(10.0, 0.0, 4.0),
        ..default()
    }
}

#[test]
fn wind_influence_changes_precipitation_drift() {
    let particle = WeatherParticle { index: 3, seed: 99 };
    let calm = rain_camera_state();

    let calm_transform = particle_transform(&particle, &calm, PrecipitationKind::Rain, 6.0);

    let mut windy = calm.clone();
    windy.wind_influence = 1.0;
    let windy_transform = particle_transform(&particle, &windy, PrecipitationKind::Rain, 6.0);

    assert_ne!(calm_transform.translation.x, windy_transform.translation.x);
    assert_ne!(calm_transform.translation.z, windy_transform.translation.z);
}

#[test]
fn particle_count_respects_quality_bias_and_budget() {
    let weather_camera = WeatherCamera {
        quality_bias: 1.8,
        ..default()
    };
    let state = WeatherCameraState {
        precipitation_kind: PrecipitationKind::Rain,
        precipitation_factor: 1.0,
        precipitation_density: 1.0,
        ..default()
    };

    let particle_count = desired_particle_count(WeatherQuality::Medium, &weather_camera, &state);

    assert_eq!(
        particle_count,
        WeatherQuality::Medium.plan().max_particles_per_camera
    );
}
