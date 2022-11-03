use bevy::prelude::*;

/*
*
* CONSTANTS
*
*/
// Number of updates per second
const TICK_RATE: f32 = 3.;
const BLOCK_SIZE: f32 = 30.;
const WALL: f32 = 20.;
const HEAD_COLOR: Color = Color::rgb(125., 125., 125.);
const TAIL_COLOR: Color = Color::rgb(75., 75., 75.);
const FOOD_COLOR: Color = Color::rgb(175., 15., 15.);

fn main() {
    App::new()
        .add_startup_system(setup)
        .add_startup_system(add_snake)
        .insert_resource(TickTimer(Timer::from_seconds(1. / 1.25, true)))
        .add_system(update_snake)
        .add_system(turn_snake)
        .add_system_to_stage(CoreStage::PostUpdate, position_translation)
        .add_system_to_stage(CoreStage::PostUpdate, size_scaling)
        .add_plugins(DefaultPlugins)
        .run();
}

/*
*
* COMPONENTS
*
*/

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Snake {
    rot: i8, // 0 -> north, 1 -> east, 2 -> south, 3 -> west
}

#[derive(Component)]
struct TailSegment;

struct TickTimer(Timer);

/*
*
* SYSTEMS
*
*/

fn add_snake(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: HEAD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Snake { rot: 3 })
        .insert(Position { x: 10, y: 10 })
        .insert(Size::square(0.8));
}

fn update_snake(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TickTimer>,
    mut query: Query<(&mut Position, &Snake)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut pos, snake) in query.iter_mut() {
            match snake.rot {
                0 => {
                    pos.y += 1;
                }
                1 => {
                    pos.x += 1;
                }
                2 => {
                    pos.y -= 1;
                }
                3 => {
                    pos.x -= 1;
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

fn turn_snake(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Snake>) {
    for mut snake in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            snake.rot = 0;
        } else if keyboard_input.pressed(KeyCode::Right) {
            snake.rot = 1;
        } else if keyboard_input.pressed(KeyCode::Down) {
            snake.rot = 2;
        } else if keyboard_input.pressed(KeyCode::Left) {
            snake.rot = 3;
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / WALL as f32 * window.width() as f32,
            sprite_size.height / WALL as f32 * window.height() as f32,
            1.,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, WALL as f32),
            convert(pos.y as f32, window.height() as f32, WALL as f32),
            0.,
        )
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
