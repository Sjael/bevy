use bevy::{
    core_pipeline::{bloom::BloomSettings, prepass::DepthPrepass},
    pbr::{NotShadowCaster, NotShadowReceiver, MaterialPipelineKey, MaterialPipeline},
    prelude::*,
    reflect::TypeUuid,
    render::{render_resource::{AsBindGroup, ShaderRef, SpecializedMeshPipelineError, RenderPipelineDescriptor}, mesh::MeshVertexBufferLayout}, scene::SceneInstance,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(MaterialPlugin::<ForceFieldMaterial>::default())
        .add_plugin(MaterialPlugin::<WaterMaterial>::default())
        .add_startup_system(setup)
        .add_system(prepare_scene)
        .add_plugin(CameraControllerPlugin)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut force_field_materials: ResMut<Assets<ForceFieldMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane{size: 5.0, subdivisions: 2 }.into()),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     ..default()
    // });
    //wall back
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(5.0).into()),
    //     material: materials.add(Color::WHITE.into()),
    //     transform: Transform::from_rotation(Quat::from_axis_angle(
    //         Vec3::X,
    //         std::f32::consts::FRAC_PI_2,
    //     ))
    //     .with_translation(Vec3::new(0.0, 0.0, -0.5)),
    //     ..default()
    // });
    // //wall right
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(5.0).into()),
    //     material: materials.add(Color::WHITE.into()),
    //     transform: Transform::from_rotation(Quat::from_axis_angle(
    //         Vec3::Z,
    //         std::f32::consts::FRAC_PI_2,
    //     ))
    //     .with_translation(Vec3::new(0.5, 0.0, 0.0)),
    //     ..default()
    // });
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube::new(0.3).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        ..default()
    });
    // sphere
    // commands.spawn((
    //     MaterialMeshBundle {
    //         mesh: meshes.add(
    //             shape::UVSphere {
    //                 radius: 1.0,
    //                 sectors: 42,
    //                 stacks: 42,
    //             }
    //             .into(),
    //         ),
    //         material: force_field_materials.add(ForceFieldMaterial {}),
    //         transform: Transform::from_xyz(0.0, 0.25, 0.0)
    //             .with_rotation(Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2)),
    //         ..default()
    //     },
    //     NotShadowReceiver,
    //     NotShadowCaster,
    // ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        DepthPrepass,
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        },
    ));
    // spawn the scene
    commands.spawn(SceneBundle {
        scene: asset_server.load("scenes/water.glb#Scene0"),
        ..default()
    });
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "30be97fb-a62f-4000-a9f9-f85ca7607272"]
pub struct ForceFieldMaterial {}

impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/force_field.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(label) = &mut descriptor.label {
            *label = format!("water__{}", *label).into();
        }
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f053e0"]
pub struct WaterMaterial {
    #[uniform(0)]
    color: Color,
    // #[texture(1)]
    // #[sampler(2)]
    // color_texture: Option<Handle<Image>>,
}

fn prepare_scene(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Scene>>,
    scene_root_nodes: Query<&Children>,
    objects: Query<(Entity, &Name)>,
    scenes: Query<&Children, With<SceneInstance>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for _event in ev_asset.iter() {
        for scene_root in scenes.iter() {
            info!("finished loading scene");
            for &root_node in scene_root.iter() {
                for &scene_objects in scene_root_nodes.get(root_node).unwrap() {
                    if let Ok((e, name)) = objects.get(scene_objects) {
                        if name.contains("water") {
                            for mesh_entity in scene_root_nodes.get(e).unwrap() {
                                let water_material = water_materials.add(WaterMaterial {
                                    color: Color::CYAN,
                                });
                                commands
                                    .entity(*mesh_entity)
                                    .remove::<Handle<StandardMaterial>>();
                                commands.entity(*mesh_entity).insert((
                                    water_material,
                                    NotShadowCaster,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}


use bevy::{
    input::mouse::{
        MouseMotion, MouseScrollUnit, MouseWheel,
    },
};

/// Provides basic movement functionality to the attached camera
#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: MouseButton,
    pub keyboard_key_enable_mouse: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
    pub orbit_focus: Vec3,
    pub orbit_mode: bool,
    pub scroll_wheel_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 0.25,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::F,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            mouse_key_enable_mouse: MouseButton::Left,
            keyboard_key_enable_mouse: KeyCode::M,
            walk_speed: 5.0,
            run_speed: 15.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            orbit_focus: Vec3::ZERO,
            orbit_mode: false,
            scroll_wheel_speed: 0.1,
        }
    }
}

pub fn camera_controller(
    time: Res<Time>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut scroll_evr: EventReader<MouseWheel>,
    key_input: Res<Input<KeyCode>>,
    mut move_toggled: Local<bool>,
    mut query: Query<
        (&mut Transform, &mut CameraController),
        With<Camera>,
    >,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) =
        query.get_single_mut()
    {
        if !options.initialized {
            let (_roll, yaw, pitch) =
                transform.rotation.to_euler(EulerRot::ZYX);
            options.yaw = yaw;
            options.pitch = pitch;
            options.initialized = true;
        }
        if !options.enabled {
            return;
        }

        let mut scroll_distance = 0.0;

        // Handle scroll input
        for ev in scroll_evr.iter() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    scroll_distance = ev.y;
                }
                MouseScrollUnit::Pixel => (),
            }
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }
        if key_input
            .just_pressed(options.keyboard_key_enable_mouse)
        {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed =
                if key_input.pressed(options.key_run) {
                    options.run_speed
                } else {
                    options.walk_speed
                };
            options.velocity =
                axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        let translation_delta =
            options.velocity.x * dt * right
                + options.velocity.y * dt * Vec3::Y
                + options.velocity.z * dt * forward;
        let mut scroll_translation = Vec3::ZERO;
        if options.orbit_mode
            && options.scroll_wheel_speed > 0.0
        {
            scroll_translation = scroll_distance
                * transform
                    .translation
                    .distance(options.orbit_focus)
                * options.scroll_wheel_speed
                * forward;
        }
        transform.translation +=
            translation_delta + scroll_translation;
        options.orbit_focus += translation_delta;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input
            .pressed(options.mouse_key_enable_mouse)
            || *move_toggled
        {
            for mouse_event in mouse_events.iter() {
                mouse_delta += mouse_event.delta;
            }
        }

        if mouse_delta != Vec2::ZERO {
            let sensitivity = if options.orbit_mode {
                options.sensitivity * 2.0
            } else {
                options.sensitivity
            };
            let (pitch, yaw) = (
                (options.pitch
                    - mouse_delta.y
                        * 0.5
                        * sensitivity
                        * dt)
                    .clamp(
                        -0.99 * std::f32::consts::FRAC_PI_2,
                        0.99 * std::f32::consts::FRAC_PI_2,
                    ),
                options.yaw
                    - mouse_delta.x * sensitivity * dt,
            );

            // Apply look update
            transform.rotation = Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                yaw,
                pitch,
            );
            options.pitch = pitch;
            options.yaw = yaw;

            if options.orbit_mode {
                let rot_matrix =
                    Mat3::from_quat(transform.rotation);
                transform.translation = options.orbit_focus
                    + rot_matrix.mul_vec3(Vec3::new(
                        0.0,
                        0.0,
                        options.orbit_focus.distance(
                            transform.translation,
                        ),
                    ));
            }
        }
    }
}

/// Simple flying camera plugin.
/// In order to function, the [`CameraController`] component should be attached to the camera entity.
#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_controller);
    }
}
