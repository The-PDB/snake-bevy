mod constants;

use crate::constants::*;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake Game".to_owned(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, handle_eat))
        .add_systems(
            FixedUpdate,
            (handle_movement, handle_death.after(handle_movement)),
        )
        .add_systems(
            PostUpdate,
            (
                translate_position.after(spawn_food),
                spawn_food.run_if(if_food_gone),
            ),
        )
        .insert_resource(Time::<Fixed>::from_seconds(SPEED))
        .run();
}

#[derive(PartialEq, Eq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Component, Clone, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeBody;

#[derive(Component)]
struct Food;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        SnakeHead {
            direction: Direction::RIGHT,
        },
        Position { x: 0, y: 0 },
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: SNAKE_BODY_COLOR,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        SnakeBody,
        Position { x: -1, y: 0 },
    ));
}

fn spawn_food(
    mut commands: Commands,
    snake_query: Query<&Position, (With<SnakeBody>, With<SnakeHead>)>,
) {
    fn get_random_position() -> Position {
        let width = (WINDOW_WIDTH / GRID_SQUARE_SIZE) as i32;
        let height = (WINDOW_HEIGHT / GRID_SQUARE_SIZE) as i32;

        Position {
            x: rand::thread_rng().gen_range((-width / 2) + 1..(width / 2) - 1),
            y: rand::thread_rng().gen_range((-height / 2) + 1..(height / 2) - 1),
        }
    }

    let mut random_pos = get_random_position();
    while snake_query.iter().any(|pos| pos == &random_pos) {
        random_pos = get_random_position();
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        Food,
        random_pos,
    ));
}

fn if_food_gone(food_query: Query<&Food>) -> bool {
    match food_query.iter().next() {
        Some(_) => false,
        None => true,
    }
}

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut SnakeHead>) {
    let mut head = query.iter_mut().next().unwrap();

    if keys.pressed(KeyCode::ArrowUp) && head.direction != Direction::DOWN {
        head.direction = Direction::UP;
    } else if keys.pressed(KeyCode::ArrowDown) && head.direction != Direction::UP {
        head.direction = Direction::DOWN;
    } else if keys.pressed(KeyCode::ArrowLeft) && head.direction != Direction::RIGHT {
        head.direction = Direction::LEFT;
    } else if keys.pressed(KeyCode::ArrowRight) && head.direction != Direction::LEFT {
        head.direction = Direction::RIGHT;
    }
}

fn handle_movement(
    mut query: Query<(&SnakeHead, &mut Position)>,
    mut body_query: Query<&mut Position, (With<SnakeBody>, Without<SnakeHead>)>,
) {
    let (head, mut pos) = query.iter_mut().next().unwrap();
    let prev_pos = pos.clone();

    match head.direction {
        Direction::UP => pos.y += 1,
        Direction::DOWN => pos.y -= 1,
        Direction::LEFT => pos.x -= 1,
        Direction::RIGHT => pos.x += 1,
    }

    let mut prev_body = prev_pos;
    for mut body_pos in body_query.iter_mut() {
        let prev = body_pos.clone();
        body_pos.x = prev_body.x;
        body_pos.y = prev_body.y;

        prev_body = prev;
    }
}

fn handle_eat(
    mut commands: Commands,
    head_query: Query<&Position, With<SnakeHead>>,
    food_query: Query<(Entity, &Position), With<Food>>,
) {
    let head_pos = head_query.single();

    for food in food_query.iter() {
        if head_pos == food.1 {
            commands.entity(food.0).despawn();
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: SNAKE_BODY_COLOR,
                        ..default()
                    },
                    transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
                    ..default()
                },
                SnakeBody,
                Position {
                    x: food.1.x,
                    y: food.1.y,
                },
            ));
        }
    }
}

fn handle_death(
    mut exit: EventWriter<AppExit>,
    head_query: Query<&Position, With<SnakeHead>>,
    body_query: Query<&Position, (With<SnakeBody>, Without<SnakeHead>)>,
) {
    let head_pos = head_query.single();
    let bound_width = (WINDOW_WIDTH / GRID_SQUARE_SIZE) as i32 / 2;
    let bound_height = (WINDOW_HEIGHT / GRID_SQUARE_SIZE) as i32 / 2;

    if head_pos.x <= -bound_width
        || head_pos.x >= bound_width
        || head_pos.y <= -bound_height
        || head_pos.y >= bound_height
        || body_query.iter().any(|pos| pos == head_pos)
    {
        exit.send(AppExit);
    }
}

fn translate_position(mut q: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            pos.x as f32 * GRID_SQUARE_SIZE,
            pos.y as f32 * GRID_SQUARE_SIZE,
            0.0,
        );
    }
}
