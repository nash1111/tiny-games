use bevy::prelude::*;
use rand::Rng;

const GRID_SIZE: i32 = 10;
const CELL_SIZE: f32 = 0.5;
const CUBE_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE;
const MOVE_INTERVAL: f32 = 0.2;
const CAMERA_DISTANCE: f32 = 12.0;

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeSegment;

#[derive(Component)]
struct Food;

#[derive(Resource, Default)]
struct SnakeBody(Vec<Entity>);

#[derive(Resource)]
struct MoveTimer(Timer);

#[derive(Clone, Copy, PartialEq, Debug)]
enum LocalDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Resource)]
struct CurrentDirection(LocalDirection);

#[derive(Resource)]
struct NextDirection(LocalDirection);

#[derive(Clone, Copy, PartialEq, Debug)]
enum CubeFace {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

#[derive(Component, Clone, Copy, Debug)]
struct GridPosition {
    face: CubeFace,
    x: i32,
    y: i32,
}

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct GameOver(bool);

#[derive(Resource)]
struct GrowPending(bool);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake 3D - Cube".to_string(),
                resolution: (800, 600).into(),
                canvas: Some("#game-canvas".to_string()),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)))
        .insert_resource(MoveTimer(Timer::from_seconds(
            MOVE_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(CurrentDirection(LocalDirection::Up))
        .insert_resource(NextDirection(LocalDirection::Up))
        .insert_resource(SnakeBody::default())
        .insert_resource(Score(0))
        .insert_resource(GameOver(false))
        .insert_resource(GrowPending(false))
        .add_systems(Startup, (setup, spawn_snake, spawn_food).chain())
        .add_systems(
            Update,
            (
                input_handler,
                snake_movement,
                check_food_collision,
                check_self_collision,
                update_camera,
                update_score_text,
                rotate_food,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_DISTANCE, CAMERA_DISTANCE * 0.8, CAMERA_DISTANCE)
            .looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 2000.0,
        affects_lightmapped_meshes: false,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.3, 0.5, 1.0),
        ..default()
    });

    let half_cube = CUBE_SIZE / 2.0;
    let line_thickness = 0.015;

    for face in [CubeFace::Top, CubeFace::Bottom, CubeFace::Front, CubeFace::Back, CubeFace::Left, CubeFace::Right] {
        draw_face_grid(&mut commands, &mut meshes, &grid_material, face, half_cube, line_thickness);
    }

    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));
}

fn draw_face_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
    face: CubeFace,
    half_cube: f32,
    thickness: f32,
) {
    let grid_len = CUBE_SIZE;

    for i in 0..=GRID_SIZE {
        let offset = (i as f32 / GRID_SIZE as f32 - 0.5) * CUBE_SIZE;

        match face {
            CubeFace::Top => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(grid_len, thickness, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, half_cube, offset),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, thickness, grid_len))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(offset, half_cube, 0.0),
                ));
            }
            CubeFace::Bottom => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(grid_len, thickness, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, -half_cube, offset),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, thickness, grid_len))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(offset, -half_cube, 0.0),
                ));
            }
            CubeFace::Front => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(grid_len, thickness, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, offset, half_cube),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, grid_len, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(offset, 0.0, half_cube),
                ));
            }
            CubeFace::Back => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(grid_len, thickness, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0.0, offset, -half_cube),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, grid_len, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(offset, 0.0, -half_cube),
                ));
            }
            CubeFace::Left => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, grid_len, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(-half_cube, 0.0, offset),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, thickness, grid_len))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(-half_cube, offset, 0.0),
                ));
            }
            CubeFace::Right => {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, grid_len, thickness))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(half_cube, 0.0, offset),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(thickness, thickness, grid_len))),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(half_cube, offset, 0.0),
                ));
            }
        }
    }
}

fn grid_to_world(pos: &GridPosition) -> Vec3 {
    let half_cube = CUBE_SIZE / 2.0;
    let x_offset = (pos.x as f32 + 0.5) / GRID_SIZE as f32 * CUBE_SIZE - half_cube;
    let y_offset = (pos.y as f32 + 0.5) / GRID_SIZE as f32 * CUBE_SIZE - half_cube;

    match pos.face {
        CubeFace::Top => Vec3::new(x_offset, half_cube + 0.1, -y_offset),
        CubeFace::Bottom => Vec3::new(x_offset, -half_cube - 0.1, y_offset),
        CubeFace::Front => Vec3::new(x_offset, y_offset, half_cube + 0.1),
        CubeFace::Back => Vec3::new(-x_offset, y_offset, -half_cube - 0.1),
        CubeFace::Left => Vec3::new(-half_cube - 0.1, y_offset, -x_offset),
        CubeFace::Right => Vec3::new(half_cube + 0.1, y_offset, x_offset),
    }
}

fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut snake_body: ResMut<SnakeBody>,
) {
    let head_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 0.2),
        emissive: LinearRgba::new(0.1, 0.5, 0.1, 1.0),
        ..default()
    });

    let segment_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.7, 0.1),
        emissive: LinearRgba::new(0.05, 0.3, 0.05, 1.0),
        ..default()
    });

    let head_mesh = meshes.add(Sphere::new(CELL_SIZE * 0.4));
    let segment_mesh = meshes.add(Sphere::new(CELL_SIZE * 0.35));

    let start_pos = GridPosition {
        face: CubeFace::Top,
        x: GRID_SIZE / 2,
        y: GRID_SIZE / 2,
    };

    let head = commands
        .spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(head_material),
            Transform::from_translation(grid_to_world(&start_pos)),
            SnakeHead,
            start_pos,
        ))
        .id();

    snake_body.0.push(head);

    for i in 1..3 {
        let segment_pos = GridPosition {
            face: CubeFace::Top,
            x: start_pos.x,
            y: start_pos.y - i,
        };
        let segment = commands
            .spawn((
                Mesh3d(segment_mesh.clone()),
                MeshMaterial3d(segment_material.clone()),
                Transform::from_translation(grid_to_world(&segment_pos)),
                SnakeSegment,
                segment_pos,
            ))
            .id();
        snake_body.0.push(segment);
    }
}

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    snake_body: Res<SnakeBody>,
    positions: Query<&GridPosition>,
    food_query: Query<Entity, With<Food>>,
) {
    for entity in food_query.iter() {
        commands.entity(entity).despawn();
    }

    let mut rng = rand::rng();
    let faces = [CubeFace::Top, CubeFace::Bottom, CubeFace::Front, CubeFace::Back, CubeFace::Left, CubeFace::Right];
    let mut food_pos: GridPosition;

    loop {
        food_pos = GridPosition {
            face: faces[rng.random_range(0..6)],
            x: rng.random_range(0..GRID_SIZE),
            y: rng.random_range(0..GRID_SIZE),
        };

        let mut collision = false;
        for &entity in snake_body.0.iter() {
            if let Ok(pos) = positions.get(entity) {
                if pos.face == food_pos.face && pos.x == food_pos.x && pos.y == food_pos.y {
                    collision = true;
                    break;
                }
            }
        }
        if !collision {
            break;
        }
    }

    let food_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.1, 0.1),
        emissive: LinearRgba::new(0.5, 0.05, 0.05, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(CELL_SIZE * 0.4, CELL_SIZE * 0.4, CELL_SIZE * 0.4))),
        MeshMaterial3d(food_material),
        Transform::from_translation(grid_to_world(&food_pos)),
        Food,
        food_pos,
    ));
}

fn rotate_food(time: Res<Time>, mut query: Query<&mut Transform, With<Food>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(time.delta_secs() * 2.0);
        transform.rotate_x(time.delta_secs() * 1.5);
    }
}

fn input_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_dir: Res<CurrentDirection>,
    mut next_dir: ResMut<NextDirection>,
) {
    let new_dir = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        Some(LocalDirection::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(LocalDirection::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(LocalDirection::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(LocalDirection::Right)
    } else {
        None
    };

    if let Some(dir) = new_dir {
        let is_opposite = matches!(
            (current_dir.0, dir),
            (LocalDirection::Up, LocalDirection::Down)
                | (LocalDirection::Down, LocalDirection::Up)
                | (LocalDirection::Left, LocalDirection::Right)
                | (LocalDirection::Right, LocalDirection::Left)
        );

        if !is_opposite {
            next_dir.0 = dir;
        }
    }
}

// Returns (new_position, new_direction)
fn move_position(pos: GridPosition, dir: LocalDirection) -> (GridPosition, LocalDirection) {
    let mut new_pos = pos;
    let mut new_dir = dir;

    match dir {
        LocalDirection::Up => new_pos.y += 1,
        LocalDirection::Down => new_pos.y -= 1,
        LocalDirection::Left => new_pos.x -= 1,
        LocalDirection::Right => new_pos.x += 1,
    }

    // Handle face transitions
    if new_pos.x < 0 {
        let (face, x, y, d) = transition_left(pos.face, pos.y);
        new_pos.face = face;
        new_pos.x = x;
        new_pos.y = y;
        new_dir = d;
    } else if new_pos.x >= GRID_SIZE {
        let (face, x, y, d) = transition_right(pos.face, pos.y);
        new_pos.face = face;
        new_pos.x = x;
        new_pos.y = y;
        new_dir = d;
    } else if new_pos.y < 0 {
        let (face, x, y, d) = transition_down(pos.face, pos.x);
        new_pos.face = face;
        new_pos.x = x;
        new_pos.y = y;
        new_dir = d;
    } else if new_pos.y >= GRID_SIZE {
        let (face, x, y, d) = transition_up(pos.face, pos.x);
        new_pos.face = face;
        new_pos.x = x;
        new_pos.y = y;
        new_dir = d;
    }

    (new_pos, new_dir)
}

fn transition_up(face: CubeFace, x: i32) -> (CubeFace, i32, i32, LocalDirection) {
    match face {
        CubeFace::Top => (CubeFace::Back, x, GRID_SIZE - 1, LocalDirection::Up),
        CubeFace::Bottom => (CubeFace::Front, x, GRID_SIZE - 1, LocalDirection::Up),
        CubeFace::Front => (CubeFace::Top, x, 0, LocalDirection::Up),
        CubeFace::Back => (CubeFace::Top, GRID_SIZE - 1 - x, GRID_SIZE - 1, LocalDirection::Down),
        CubeFace::Left => (CubeFace::Top, 0, x, LocalDirection::Right),
        CubeFace::Right => (CubeFace::Top, GRID_SIZE - 1, GRID_SIZE - 1 - x, LocalDirection::Left),
    }
}

fn transition_down(face: CubeFace, x: i32) -> (CubeFace, i32, i32, LocalDirection) {
    match face {
        CubeFace::Top => (CubeFace::Front, x, 0, LocalDirection::Down),
        CubeFace::Bottom => (CubeFace::Back, x, 0, LocalDirection::Down),
        CubeFace::Front => (CubeFace::Bottom, x, 0, LocalDirection::Down),
        CubeFace::Back => (CubeFace::Bottom, GRID_SIZE - 1 - x, GRID_SIZE - 1, LocalDirection::Up),
        CubeFace::Left => (CubeFace::Bottom, 0, GRID_SIZE - 1 - x, LocalDirection::Right),
        CubeFace::Right => (CubeFace::Bottom, GRID_SIZE - 1, x, LocalDirection::Left),
    }
}

fn transition_left(face: CubeFace, y: i32) -> (CubeFace, i32, i32, LocalDirection) {
    match face {
        CubeFace::Top => (CubeFace::Left, y, GRID_SIZE - 1, LocalDirection::Up),
        CubeFace::Bottom => (CubeFace::Left, GRID_SIZE - 1 - y, 0, LocalDirection::Down),
        CubeFace::Front => (CubeFace::Left, GRID_SIZE - 1, y, LocalDirection::Left),
        CubeFace::Back => (CubeFace::Right, GRID_SIZE - 1, y, LocalDirection::Left),
        CubeFace::Left => (CubeFace::Back, GRID_SIZE - 1, y, LocalDirection::Left),
        CubeFace::Right => (CubeFace::Front, GRID_SIZE - 1, y, LocalDirection::Left),
    }
}

fn transition_right(face: CubeFace, y: i32) -> (CubeFace, i32, i32, LocalDirection) {
    match face {
        CubeFace::Top => (CubeFace::Right, GRID_SIZE - 1 - y, GRID_SIZE - 1, LocalDirection::Up),
        CubeFace::Bottom => (CubeFace::Right, y, 0, LocalDirection::Down),
        CubeFace::Front => (CubeFace::Right, 0, y, LocalDirection::Right),
        CubeFace::Back => (CubeFace::Left, 0, y, LocalDirection::Right),
        CubeFace::Left => (CubeFace::Front, 0, y, LocalDirection::Right),
        CubeFace::Right => (CubeFace::Back, 0, y, LocalDirection::Right),
    }
}

fn snake_movement(
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    mut current_dir: ResMut<CurrentDirection>,
    mut next_dir: ResMut<NextDirection>,
    mut snake_body: ResMut<SnakeBody>,
    mut positions: Query<&mut GridPosition>,
    mut transforms: Query<&mut Transform>,
    game_over: Res<GameOver>,
    mut grow_pending: ResMut<GrowPending>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if game_over.0 {
        return;
    }

    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    current_dir.0 = next_dir.0;

    let mut prev_positions: Vec<GridPosition> = Vec::new();
    for &entity in snake_body.0.iter() {
        if let Ok(pos) = positions.get(entity) {
            prev_positions.push(*pos);
        }
    }

    if let Some(&head_entity) = snake_body.0.first() {
        if let Ok(mut head_pos) = positions.get_mut(head_entity) {
            let (new_pos, new_dir) = move_position(*head_pos, current_dir.0);
            *head_pos = new_pos;
            current_dir.0 = new_dir;
            next_dir.0 = new_dir;

            if let Ok(mut transform) = transforms.get_mut(head_entity) {
                transform.translation = grid_to_world(&head_pos);
            }
        }
    }

    for (i, &entity) in snake_body.0.iter().enumerate().skip(1) {
        if let Ok(mut pos) = positions.get_mut(entity) {
            *pos = prev_positions[i - 1];
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = grid_to_world(&pos);
            }
        }
    }

    if grow_pending.0 {
        if let Some(&last_pos) = prev_positions.last() {
            let segment_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.7, 0.1),
                emissive: LinearRgba::new(0.05, 0.3, 0.05, 1.0),
                ..default()
            });
            let segment_mesh = meshes.add(Sphere::new(CELL_SIZE * 0.35));

            let new_segment = commands
                .spawn((
                    Mesh3d(segment_mesh),
                    MeshMaterial3d(segment_material),
                    Transform::from_translation(grid_to_world(&last_pos)),
                    SnakeSegment,
                    last_pos,
                ))
                .id();
            snake_body.0.push(new_segment);
        }
        grow_pending.0 = false;
    }
}

fn check_food_collision(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    snake_body: Res<SnakeBody>,
    head_query: Query<&GridPosition, With<SnakeHead>>,
    food_query: Query<(Entity, &GridPosition), With<Food>>,
    mut score: ResMut<Score>,
    positions: Query<&GridPosition>,
    mut grow_pending: ResMut<GrowPending>,
) {
    let Ok(head_pos) = head_query.single() else {
        return;
    };

    let Ok((food_entity, food_pos)) = food_query.single() else {
        return;
    };

    if head_pos.face == food_pos.face && head_pos.x == food_pos.x && head_pos.y == food_pos.y {
        score.0 += 10;
        grow_pending.0 = true;

        commands.entity(food_entity).despawn();

        let mut rng = rand::rng();
        let faces = [CubeFace::Top, CubeFace::Bottom, CubeFace::Front, CubeFace::Back, CubeFace::Left, CubeFace::Right];
        let mut new_food_pos: GridPosition;

        loop {
            new_food_pos = GridPosition {
                face: faces[rng.random_range(0..6)],
                x: rng.random_range(0..GRID_SIZE),
                y: rng.random_range(0..GRID_SIZE),
            };

            let mut collision = false;
            for &entity in snake_body.0.iter() {
                if let Ok(pos) = positions.get(entity) {
                    if pos.face == new_food_pos.face && pos.x == new_food_pos.x && pos.y == new_food_pos.y {
                        collision = true;
                        break;
                    }
                }
            }
            if !collision {
                break;
            }
        }

        let food_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.1, 0.1),
            emissive: LinearRgba::new(0.5, 0.05, 0.05, 1.0),
            ..default()
        });

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(CELL_SIZE * 0.4, CELL_SIZE * 0.4, CELL_SIZE * 0.4))),
            MeshMaterial3d(food_material),
            Transform::from_translation(grid_to_world(&new_food_pos)),
            Food,
            new_food_pos,
        ));
    }
}

fn check_self_collision(
    snake_body: Res<SnakeBody>,
    positions: Query<&GridPosition>,
    mut game_over: ResMut<GameOver>,
) {
    if snake_body.0.len() < 5 {
        return;
    }

    let Some(&head_entity) = snake_body.0.first() else {
        return;
    };

    let Ok(head_pos) = positions.get(head_entity) else {
        return;
    };

    for &entity in snake_body.0.iter().skip(1) {
        if let Ok(pos) = positions.get(entity) {
            if pos.face == head_pos.face && pos.x == head_pos.x && pos.y == head_pos.y {
                game_over.0 = true;
                return;
            }
        }
    }
}

fn update_camera(
    head_query: Query<&GridPosition, With<SnakeHead>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    time: Res<Time>,
) {
    let Ok(head_pos) = head_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let head_world = grid_to_world(head_pos);

    let face_normal = match head_pos.face {
        CubeFace::Top => Vec3::Y,
        CubeFace::Bottom => Vec3::NEG_Y,
        CubeFace::Front => Vec3::Z,
        CubeFace::Back => Vec3::NEG_Z,
        CubeFace::Left => Vec3::NEG_X,
        CubeFace::Right => Vec3::X,
    };

    let target_camera_pos = head_world + face_normal * CAMERA_DISTANCE * 0.7;

    camera_transform.translation = camera_transform
        .translation
        .lerp(target_camera_pos, time.delta_secs() * 2.0);

    let up = if face_normal.abs().y > 0.9 {
        Vec3::Z
    } else {
        Vec3::Y
    };

    camera_transform.look_at(Vec3::ZERO, up);
}

fn update_score_text(
    score: Res<Score>,
    game_over: Res<GameOver>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    for mut text in query.iter_mut() {
        if game_over.0 {
            **text = format!("Game Over! Final Score: {}", score.0);
        } else {
            **text = format!("Score: {}", score.0);
        }
    }
}
