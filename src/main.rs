use bevy::prelude::*;
use rand::Rng;

const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 20;
const CELL_SIZE: f32 = 20.0;
const MOVE_INTERVAL: f32 = 0.15;

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

#[derive(Resource, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Resource)]
struct CurrentDirection(Direction);

#[derive(Resource)]
struct NextDirection(Direction);

#[derive(Component, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Resource)]
struct Score(u32);

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
struct GameOver(bool);

#[derive(Resource)]
struct GrowPending(bool);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake Game".to_string(),
                resolution: (
                    (GRID_WIDTH as f32 * CELL_SIZE + 40.0) as u32,
                    (GRID_HEIGHT as f32 * CELL_SIZE + 80.0) as u32,
                )
                    .into(),
                canvas: Some("#game-canvas".to_string()),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(MoveTimer(Timer::from_seconds(
            MOVE_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(CurrentDirection(Direction::Right))
        .insert_resource(NextDirection(Direction::Right))
        .insert_resource(SnakeBody::default())
        .insert_resource(Score(0))
        .insert_resource(GameOver(false))
        .insert_resource(GrowPending(false))
        .add_systems(Startup, (setup, spawn_snake, spawn_initial_food).chain())
        .add_systems(
            Update,
            (
                input_handler,
                snake_movement,
                check_food_collision,
                check_wall_collision,
                check_self_collision,
                update_score_text,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let offset_x = -(GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let offset_y = -(GRID_HEIGHT as f32 * CELL_SIZE) / 2.0 - 20.0;

    for x in 0..=GRID_WIDTH {
        let x_pos = offset_x + x as f32 * CELL_SIZE;
        commands.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(2.0, GRID_HEIGHT as f32 * CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(x_pos, offset_y + (GRID_HEIGHT as f32 * CELL_SIZE) / 2.0, 0.0),
        ));
    }
    for y in 0..=GRID_HEIGHT {
        let y_pos = offset_y + y as f32 * CELL_SIZE;
        commands.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(GRID_WIDTH as f32 * CELL_SIZE, 2.0)),
                ..default()
            },
            Transform::from_xyz(offset_x + (GRID_WIDTH as f32 * CELL_SIZE) / 2.0, y_pos, 0.0),
        ));
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

fn grid_to_world(pos: &Position) -> Vec3 {
    let offset_x = -(GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
    let offset_y = -(GRID_HEIGHT as f32 * CELL_SIZE) / 2.0 - 20.0;
    Vec3::new(
        offset_x + pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        offset_y + pos.y as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        1.0,
    )
}

fn spawn_snake(mut commands: Commands, mut snake_body: ResMut<SnakeBody>) {
    let start_pos = Position {
        x: GRID_WIDTH / 2,
        y: GRID_HEIGHT / 2,
    };

    let head = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.0, 0.8, 0.0),
                custom_size: Some(Vec2::splat(CELL_SIZE - 2.0)),
                ..default()
            },
            Transform::from_translation(grid_to_world(&start_pos)),
            SnakeHead,
            start_pos,
        ))
        .id();

    snake_body.0.push(head);

    for i in 1..3 {
        let segment_pos = Position {
            x: start_pos.x - i,
            y: start_pos.y,
        };
        let segment = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.6, 0.0),
                    custom_size: Some(Vec2::splat(CELL_SIZE - 4.0)),
                    ..default()
                },
                Transform::from_translation(grid_to_world(&segment_pos)),
                SnakeSegment,
                segment_pos,
            ))
            .id();
        snake_body.0.push(segment);
    }
}

fn spawn_initial_food(mut commands: Commands) {
    let mut rng = rand::rng();
    let food_pos = Position {
        x: rng.random_range(0..GRID_WIDTH),
        y: rng.random_range(0..GRID_HEIGHT),
    };

    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.0, 0.0),
            custom_size: Some(Vec2::splat(CELL_SIZE - 4.0)),
            ..default()
        },
        Transform::from_translation(grid_to_world(&food_pos)),
        Food,
        food_pos,
    ));
}

fn input_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_dir: Res<CurrentDirection>,
    mut next_dir: ResMut<NextDirection>,
) {
    let new_dir = if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    if let Some(dir) = new_dir {
        let is_opposite = matches!(
            (current_dir.0, dir),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );

        if !is_opposite {
            next_dir.0 = dir;
        }
    }
}

fn snake_movement(
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    mut current_dir: ResMut<CurrentDirection>,
    next_dir: Res<NextDirection>,
    mut snake_body: ResMut<SnakeBody>,
    mut positions: Query<&mut Position>,
    mut transforms: Query<&mut Transform>,
    game_over: Res<GameOver>,
    mut grow_pending: ResMut<GrowPending>,
    mut commands: Commands,
) {
    if game_over.0 {
        return;
    }

    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    current_dir.0 = next_dir.0;

    let mut prev_positions: Vec<Position> = Vec::new();
    for &entity in snake_body.0.iter() {
        if let Ok(pos) = positions.get(entity) {
            prev_positions.push(*pos);
        }
    }

    if let Some(&head_entity) = snake_body.0.first() {
        if let Ok(mut head_pos) = positions.get_mut(head_entity) {
            match current_dir.0 {
                Direction::Up => head_pos.y += 1,
                Direction::Down => head_pos.y -= 1,
                Direction::Left => head_pos.x -= 1,
                Direction::Right => head_pos.x += 1,
            }

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
            let new_segment = commands
                .spawn((
                    Sprite {
                        color: Color::srgb(0.0, 0.6, 0.0),
                        custom_size: Some(Vec2::splat(CELL_SIZE - 4.0)),
                        ..default()
                    },
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
    snake_body: Res<SnakeBody>,
    head_query: Query<&Position, With<SnakeHead>>,
    food_query: Query<(Entity, &Position), With<Food>>,
    mut score: ResMut<Score>,
    positions: Query<&Position>,
    mut grow_pending: ResMut<GrowPending>,
) {
    let Ok(head_pos) = head_query.single() else {
        return;
    };

    let Ok((food_entity, food_pos)) = food_query.single() else {
        return;
    };

    if head_pos.x == food_pos.x && head_pos.y == food_pos.y {
        score.0 += 10;
        grow_pending.0 = true;

        commands.entity(food_entity).despawn();

        let mut rng = rand::rng();
        let mut new_food_pos: Position;
        loop {
            new_food_pos = Position {
                x: rng.random_range(0..GRID_WIDTH),
                y: rng.random_range(0..GRID_HEIGHT),
            };

            let mut collision = false;
            for &entity in snake_body.0.iter() {
                if let Ok(pos) = positions.get(entity) {
                    if pos.x == new_food_pos.x && pos.y == new_food_pos.y {
                        collision = true;
                        break;
                    }
                }
            }
            if !collision {
                break;
            }
        }

        commands.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.0, 0.0),
                custom_size: Some(Vec2::splat(CELL_SIZE - 4.0)),
                ..default()
            },
            Transform::from_translation(grid_to_world(&new_food_pos)),
            Food,
            new_food_pos,
        ));
    }
}

fn check_wall_collision(
    head_query: Query<&Position, With<SnakeHead>>,
    mut game_over: ResMut<GameOver>,
) {
    let Ok(head_pos) = head_query.single() else {
        return;
    };

    if head_pos.x < 0 || head_pos.x >= GRID_WIDTH || head_pos.y < 0 || head_pos.y >= GRID_HEIGHT {
        game_over.0 = true;
    }
}

fn check_self_collision(
    snake_body: Res<SnakeBody>,
    positions: Query<&Position>,
    mut game_over: ResMut<GameOver>,
) {
    if snake_body.0.len() < 2 {
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
            if pos.x == head_pos.x && pos.y == head_pos.y {
                game_over.0 = true;
                return;
            }
        }
    }
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
