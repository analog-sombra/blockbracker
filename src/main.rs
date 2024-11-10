use std::default;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2d, Mesh2dHandle},
    window::WindowLevel,
};

use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

// #[cfg(any(target_os = "macos", target_os = "linux"))]
// use bevy::window::CompositeAlphaMode;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Obstacle;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Just Move".to_string(),

                    // visible: true,
                    // skip_taskbar: false,
                    // decorations: false,
                    // transparent: true,
                    // resizable: false,
                    window_level: WindowLevel::AlwaysOnTop,
                    // mode: bevy::window::WindowMode::BorderlessFullscreen,
                    #[cfg(target_os = "macos")]
                    composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                    #[cfg(target_os = "linux")]
                    composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(InputManagerPlugin::<Action>::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, move_player_system)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum Action {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    RLEFT,
    RRIGHT,
}

fn player_input_map() -> InputMap<Action> {
    let mut map = InputMap::default();

    map.insert_multiple([
        (Action::LEFT, KeyCode::KeyA),
        (Action::RIGHT, KeyCode::KeyD),
        (Action::UP, KeyCode::KeyW),
        (Action::DOWN, KeyCode::KeyS),
        (Action::RLEFT, KeyCode::KeyQ),
        (Action::RRIGHT, KeyCode::KeyE),
    ]);

    return map;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Add a 2D camera
    commands.spawn(Camera2dBundle::default());

    let shape: Mesh2dHandle = Mesh2dHandle(meshes.add(Rectangle::new(50.0, 50.0)));
    let color = Color::linear_rgb(0.0, 131.0, 132.0);

    // Spawn the player with a collider and a dynamic rigid body
    let player_size = Vec2::new(50.0, 50.0);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Player,
        RigidBody::Fixed,
        Collider::cuboid(player_size.x / 2.0, player_size.y / 2.0),
        InputManagerBundle::<Action> {
            input_map: player_input_map(),
            ..default()
        },
    ));

    // Spawn obstacles
    let obstacle_size = Vec2::new(100.0, 100.0);
    for i in -2..=2 {
        let position = Vec3::new(i as f32 * 150.0, 100.0, 0.0);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(50.0, 50.0))),
                material: materials.add(Color::linear_rgb(232.0, 131.0, 132.0)),
                transform: Transform::from_translation(position),
                ..default()
            },
            Obstacle,
            RigidBody::Fixed,
            Collider::cuboid(obstacle_size.x / 2.0, obstacle_size.y / 2.0),
        ));
    }
}

fn move_player_system(
    mut query: Query<(&mut Transform, &ActionState<Action>), With<Player>>,
    time: Res<Time>,
    mut window: Query<&Window>,
) {
    let window = window.single();
    let half_width = window.resolution.width() / 2.0;
    let half_height = window.resolution.height() / 2.0;
    let player_half_size = 25.0;
    let speed = 200.0; // Adjust speed as needed
    let rotation_speed = std::f32::consts::PI / 2.0; // Rotation speed in radians per second

    for (mut transform, action_state) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if action_state.pressed(&Action::UP) {
            direction.y += 1.0;
        }
        if action_state.pressed(&Action::DOWN) {
            direction.y -= 1.0;
        }
        if action_state.pressed(&Action::LEFT) {
            direction.x -= 1.0;
        }
        if action_state.pressed(&Action::RIGHT) {
            direction.x += 1.0;
        }

        // Normalize direction vector to avoid faster diagonal movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // Move the player based on direction and delta time
        transform.translation += direction * speed * time.delta_seconds();

        // Rotation handling
        if action_state.pressed(&Action::RLEFT) {
            transform.rotation =
                transform.rotation * Quat::from_rotation_z(rotation_speed * time.delta_seconds());
        }
        if action_state.pressed(&Action::RRIGHT) {
            transform.rotation =
                transform.rotation * Quat::from_rotation_z(-rotation_speed * time.delta_seconds());
        }

        // Calculate rotated bounds based on current rotation
        let rotation_matrix = Mat3::from_quat(transform.rotation);
        let rotated_x_extent = rotation_matrix.x_axis.abs() * player_half_size;
        let rotated_y_extent = rotation_matrix.y_axis.abs() * player_half_size;

        // Calculate clamping bounds considering rotation
        let clamped_x = transform.translation.x.clamp(
            -half_width + rotated_x_extent.length(),
            half_width - rotated_x_extent.length(),
        );
        let clamped_y = transform.translation.y.clamp(
            -half_height + rotated_y_extent.length(),
            half_height - rotated_y_extent.length(),
        );

        // Apply clamped translation
        transform.translation.x = clamped_x;
        transform.translation.y = clamped_y;
        // // Prevent player from going out of bounds by clamping their position
        // transform.translation.x = transform.translation.x.clamp(
        //     -half_width + player_half_size,
        //     half_width - player_half_size,
        // );
        // transform.translation.y = transform.translation.y.clamp(
        //     -half_height + player_half_size,
        //     half_height - player_half_size,
        // );
    }
}
