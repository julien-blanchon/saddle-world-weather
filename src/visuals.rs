use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    color::LinearRgba,
    color::palettes::css,
    light::{NotShadowCaster, NotShadowReceiver},
    math::primitives::{Cuboid, Rectangle},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    PrecipitationKind, WeatherCamera, WeatherCameraState, WeatherConfig, WeatherQuality,
    WeatherScreenFxMode,
    profiles::WeatherQualityPlan, solver::hash01,
};

#[derive(Resource, Default)]
pub(crate) struct WeatherVisualAssets {
    initialized: bool,
    rain_mesh: Handle<Mesh>,
    snow_mesh: Handle<Mesh>,
    overlay_mesh: Handle<Mesh>,
    rain_material: Handle<StandardMaterial>,
    snow_material: Handle<StandardMaterial>,
    overlay_texture: Handle<Image>,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct WeatherEmitterRoot {
    pub camera: Entity,
    pub kind: PrecipitationKind,
    pub particle_count: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct WeatherParticle {
    pub index: usize,
    pub seed: u64,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct WeatherScreenOverlay {
    pub camera: Entity,
    pub material: Handle<StandardMaterial>,
}

fn overlay_alpha(state: &WeatherCameraState) -> f32 {
    (state.screen_fx_factor + state.lightning_flash_intensity * 0.35).clamp(0.0, 1.0)
}

fn overlay_emissive(state: &WeatherCameraState) -> LinearRgba {
    LinearRgba::WHITE * (state.lightning_flash_intensity * 0.55).clamp(0.0, 1.0)
}

pub(crate) fn sync_precipitation_emitters(
    mut commands: Commands,
    config: Res<WeatherConfig>,
    internal: Res<crate::systems::WeatherInternalState>,
    mut visual_assets: ResMut<WeatherVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<
        (
            Entity,
            &WeatherCamera,
            &mut WeatherCameraState,
            &GlobalTransform,
        ),
        With<Camera>,
    >,
    emitters: Query<(Entity, &WeatherEmitterRoot, Option<&Children>)>,
    mut emitter_transforms: Query<
        &mut Transform,
        (With<WeatherEmitterRoot>, Without<WeatherParticle>),
    >,
    mut particles: Query<(&WeatherParticle, &mut Transform), Without<WeatherEmitterRoot>>,
) {
    ensure_visual_assets(
        config.quality.plan(),
        config.seed,
        visual_assets.as_mut(),
        meshes.as_mut(),
        materials.as_mut(),
        images.as_mut(),
    );

    let existing_emitters: HashMap<Entity, (Entity, PrecipitationKind, usize, Vec<Entity>)> =
        emitters
            .iter()
            .map(|(entity, emitter, children)| {
                (
                    emitter.camera,
                    (
                        entity,
                        emitter.kind.clone(),
                        emitter.particle_count,
                        children
                            .map(|children| children.iter().collect())
                            .unwrap_or_default(),
                    ),
                )
            })
            .collect();

    let mut desired_emitters = HashSet::new();

    for (camera_entity, weather_camera, mut state, global_transform) in &mut cameras {
        if !weather_camera.enabled || !weather_camera.receive_precipitation {
            state.active_particles = 0;
            continue;
        }

        let desired_count = desired_particle_count(config.quality, weather_camera, &state);
        if desired_count == 0 || matches!(state.precipitation_kind, PrecipitationKind::None) {
            state.active_particles = 0;
            continue;
        }

        state.active_particles = desired_count;
        desired_emitters.insert(camera_entity);

        let (root_entity, current_kind, current_count, children) =
            if let Some(existing) = existing_emitters.get(&camera_entity) {
                existing.clone()
            } else {
                let root = commands
                    .spawn((
                        Name::new(format!("Weather Emitter {}", camera_entity.index())),
                        WeatherEmitterRoot {
                            camera: camera_entity,
                            kind: state.precipitation_kind.clone(),
                            particle_count: desired_count,
                        },
                        Transform::from_translation(global_transform.translation()),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                    ))
                    .id();
                (root, state.precipitation_kind.clone(), 0, Vec::new())
            };

        if current_kind != state.precipitation_kind || current_count != desired_count {
            commands.entity(root_entity).despawn_related::<Children>();
            commands.entity(root_entity).insert(WeatherEmitterRoot {
                camera: camera_entity,
                kind: state.precipitation_kind.clone(),
                particle_count: desired_count,
            });
            spawn_particles(
                &mut commands,
                &visual_assets,
                root_entity,
                &state,
                config.seed ^ camera_entity.to_bits(),
                desired_count,
                internal.elapsed_time_secs,
            );
        } else if !children.is_empty() {
            for child in children {
                if let Ok((particle, mut transform)) = particles.get_mut(child) {
                    *transform = particle_transform(
                        particle,
                        &state,
                        state.precipitation_kind.clone(),
                        internal.elapsed_time_secs,
                    );
                }
            }
        }

        if let Ok(mut transform) = emitter_transforms.get_mut(root_entity) {
            transform.translation = global_transform.translation();
        }
    }

    for (camera, (entity, _, _, _)) in existing_emitters {
        if !desired_emitters.contains(&camera) {
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn sync_screen_effect_overlays(
    mut commands: Commands,
    config: Res<WeatherConfig>,
    mut visual_assets: ResMut<WeatherVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    cameras: Query<(Entity, &WeatherCamera, &WeatherCameraState), With<Camera>>,
    overlays: Query<(Entity, &WeatherScreenOverlay)>,
) {
    ensure_visual_assets(
        config.quality.plan(),
        config.seed,
        visual_assets.as_mut(),
        meshes.as_mut(),
        materials.as_mut(),
        images.as_mut(),
    );

    let existing_overlays: HashMap<Entity, (Entity, Handle<StandardMaterial>)> = overlays
        .iter()
        .map(|(entity, overlay)| (overlay.camera, (entity, overlay.material.clone())))
        .collect();

    if matches!(config.screen_fx_mode, WeatherScreenFxMode::StateOnly) {
        for (_, (entity, _)) in existing_overlays {
            commands.entity(entity).despawn();
        }
        return;
    }

    let mut desired_overlays = HashSet::new();
    for (camera_entity, weather_camera, state) in &cameras {
        let should_show = weather_camera.enabled
            && weather_camera.receive_screen_fx
            && config.quality.plan().enable_screen_fx
            && state.screen_fx_factor > 0.01;
        if !should_show {
            continue;
        }

        desired_overlays.insert(camera_entity);
        if let Some((_, handle)) = existing_overlays.get(&camera_entity) {
            if let Some(material) = materials.get_mut(handle) {
                material.base_color = state.screen_tint.with_alpha(overlay_alpha(state));
                material.emissive = overlay_emissive(state);
            }
            continue;
        }

        let material = materials.add(StandardMaterial {
            base_color: state.screen_tint.with_alpha(overlay_alpha(state)),
            base_color_texture: Some(visual_assets.overlay_texture.clone()),
            emissive: overlay_emissive(state),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            double_sided: true,
            ..default()
        });

        commands.entity(camera_entity).with_children(|parent| {
            parent.spawn((
                Name::new(format!("Weather Screen Overlay {}", camera_entity.index())),
                WeatherScreenOverlay {
                    camera: camera_entity,
                    material: material.clone(),
                },
                Mesh3d(visual_assets.overlay_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, 0.0, -1.2),
                Visibility::Visible,
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });
    }

    for (camera, (entity, _)) in existing_overlays {
        if !desired_overlays.contains(&camera) {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_particles(
    commands: &mut Commands,
    visual_assets: &WeatherVisualAssets,
    root_entity: Entity,
    state: &WeatherCameraState,
    base_seed: u64,
    desired_count: usize,
    time_secs: f32,
) {
    let (mesh, material) = match state.precipitation_kind {
        PrecipitationKind::Snow => (
            visual_assets.snow_mesh.clone(),
            visual_assets.snow_material.clone(),
        ),
        _ => (
            visual_assets.rain_mesh.clone(),
            visual_assets.rain_material.clone(),
        ),
    };

    commands.entity(root_entity).with_children(|parent| {
        for index in 0..desired_count {
            let seed = base_seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            parent.spawn((
                Name::new(format!("Weather Particle {}", index + 1)),
                WeatherParticle { index, seed },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                particle_transform(
                    &WeatherParticle { index, seed },
                    state,
                    state.precipitation_kind.clone(),
                    time_secs,
                ),
                Visibility::Visible,
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    });
}

fn particle_transform(
    particle: &WeatherParticle,
    state: &WeatherCameraState,
    kind: PrecipitationKind,
    time_secs: f32,
) -> Transform {
    let height = state.near_height.max(1.0);
    let radius = state.near_radius.max(1.0);
    let fall_speed = state.fall_speed.max(0.1);
    let wind_vector = state.wind_vector * state.wind_influence.clamp(0.0, 2.0);

    let x = ((hash01(particle.seed, particle.index as u64, 1) + time_secs * wind_vector.x * 0.03)
        .fract()
        - 0.5)
        * radius
        * 2.0;
    let z = ((hash01(particle.seed, particle.index as u64, 2) + time_secs * wind_vector.z * 0.03)
        .fract()
        - 0.5)
        * radius
        * 2.0;
    let y = (0.5
        - (hash01(particle.seed, particle.index as u64, 3) + time_secs * fall_speed / height)
            .fract())
        * height;

    let lateral_sway = match kind {
        PrecipitationKind::Snow => {
            Vec3::new(
                (time_secs * 0.8 + hash01(particle.seed, 1, 5) * std::f32::consts::TAU).sin(),
                0.0,
                (time_secs * 1.1 + hash01(particle.seed, 1, 6) * std::f32::consts::TAU).cos(),
            ) * (0.6 + state.precipitation_factor * 0.8)
        }
        _ => {
            Vec3::new(
                (time_secs * 1.4 + hash01(particle.seed, 1, 7) * std::f32::consts::TAU).sin(),
                0.0,
                (time_secs * 1.2 + hash01(particle.seed, 1, 8) * std::f32::consts::TAU).cos(),
            ) * 0.08
        }
    };

    let velocity =
        Vec3::new(wind_vector.x * 0.35, -fall_speed, wind_vector.z * 0.35).normalize_or_zero();

    let rotation = match kind {
        PrecipitationKind::Snow => Quat::from_euler(
            EulerRot::XYZ,
            time_secs * 0.4 + hash01(particle.seed, 1, 9) * std::f32::consts::TAU,
            time_secs * 0.3 + hash01(particle.seed, 1, 10) * std::f32::consts::TAU,
            time_secs * 0.5 + hash01(particle.seed, 1, 11) * std::f32::consts::TAU,
        ),
        _ => Quat::from_rotation_arc(Vec3::Y, velocity),
    };

    let scale = match kind {
        PrecipitationKind::Snow => {
            Vec3::splat((state.particle_size.x + state.particle_size.y) * 0.5)
        }
        _ => Vec3::new(
            state.particle_size.x.max(0.01),
            state.particle_size.y.max(0.1) * (0.8 + state.precipitation_factor * 0.7),
            state.particle_size.x.max(0.01),
        ),
    };

    Transform {
        translation: Vec3::new(x, y, z) + lateral_sway,
        rotation,
        scale,
    }
}

#[cfg(test)]
#[path = "visuals_tests.rs"]
mod tests;

fn desired_particle_count(
    quality: WeatherQuality,
    weather_camera: &WeatherCamera,
    state: &WeatherCameraState,
) -> usize {
    let quality_plan = quality.plan();
    let base = quality_plan.max_particles_per_camera as f32
        * state
            .precipitation_factor
            .max(state.precipitation_density * 0.5)
        * weather_camera.quality_bias.clamp(0.25, 2.0);
    if base <= 1.0 {
        0
    } else {
        base.round()
            .clamp(8.0, quality_plan.max_particles_per_camera as f32) as usize
    }
}

fn ensure_visual_assets(
    quality_plan: WeatherQualityPlan,
    seed: u64,
    visual_assets: &mut WeatherVisualAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    if visual_assets.initialized {
        return;
    }

    let overlay_image = generate_overlay_texture(quality_plan.overlay_resolution, seed);
    visual_assets.overlay_texture = images.add(overlay_image);
    visual_assets.rain_mesh = meshes.add(Mesh::from(Cuboid::new(0.03, 0.9, 0.03)));
    visual_assets.snow_mesh = meshes.add(Mesh::from(Cuboid::new(0.08, 0.08, 0.08)));
    visual_assets.overlay_mesh = meshes.add(Mesh::from(Rectangle::new(2.7, 1.6)));
    visual_assets.rain_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.92, 1.0).with_alpha(0.45),
        emissive: Color::srgb(0.16, 0.18, 0.22).to_linear() * 0.15,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.95,
        unlit: true,
        ..default()
    });
    visual_assets.snow_material = materials.add(StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.94),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });
    visual_assets.initialized = true;
}

fn generate_overlay_texture(resolution: u32, seed: u64) -> Image {
    let size = resolution.max(32);
    let mut image = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let data = image
        .data
        .as_mut()
        .expect("generated weather overlay image should have backing storage");

    for y in 0..size {
        for x in 0..size {
            let uv = Vec2::new(x as f32 / size as f32, y as f32 / size as f32);
            let mut alpha = 0.0_f32;

            for droplet in 0..18_u64 {
                let center = Vec2::new(hash01(seed, droplet, 31), hash01(seed, droplet, 37));
                let radius = 0.018 + hash01(seed, droplet, 41) * 0.05;
                let distance = uv.distance(center);
                if distance < radius {
                    alpha = alpha.max(1.0 - distance / radius);
                }
            }

            let streak =
                ((uv.x * 14.0 + hash01(seed, x as u64, 43) * 2.0 + uv.y * 4.0).sin() * 0.5 + 0.5)
                    .powf(18.0)
                    * (0.2 + uv.y * 0.4);
            alpha = alpha.max(streak * 0.65);

            let color = Color::from(css::WHITE).with_alpha(alpha.clamp(0.0, 1.0));
            let linear = color.to_srgba();
            let index = ((y * size + x) * 4) as usize;
            data[index] = (linear.red * 255.0) as u8;
            data[index + 1] = (linear.green * 255.0) as u8;
            data[index + 2] = (linear.blue * 255.0) as u8;
            data[index + 3] = (linear.alpha * 255.0) as u8;
        }
    }

    image
}
