use crate::{cheats::ScoreBlocker, Position};
use bevy::prelude::*;

use std::collections::VecDeque;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Up
    }
}

#[derive(Resource, Default)]
pub struct LastPressed(pub Direction);

#[derive(Resource, Default)]
pub struct InputQueue(pub VecDeque<Direction>);

#[derive(Resource, Default)]
pub struct InputQueueTimer(pub Timer);

#[derive(Resource, Default)]
pub struct TickAccum(pub f32);

#[derive(Component)]
pub struct SnakeHead {
    pub rot: Direction,
}

#[derive(Component)]
pub struct Segment;

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct SnakeSegments(Vec<Entity>);

#[derive(Resource, Default, Debug)]
pub struct LastTailPosition(pub Option<Position>);

#[derive(Resource, Default, Debug)]
pub struct NextDirection(pub Option<Direction>);

#[derive(Event)]
pub struct EatEvent;

pub fn add_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
) {
    *segments = SnakeSegments(vec![commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(125., 125., 175.),
                ..default()
            },
            ..default()
        })
        .insert(SnakeHead {
            rot: Direction::Left,
        })
        .insert(Segment)
        .insert(Position { x: 10, y: 10 })
        .insert(crate::Size::square(crate::BLOCK_SIZE))
        .id()]);
    *last_tail_position = LastTailPosition(Some(Position { x: 11, y: 10 }));
}

pub fn snake_eating(
    mut commands: Commands,
    mut eat_writer: EventWriter<EatEvent>,
    food_positions: Query<(Entity, &Position), With<crate::food::Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
    mut score: ResMut<crate::score::Score>,
    mut tick_timer: ResMut<crate::TickTimer>,
    mut tick_accum: ResMut<TickAccum>,
    score_blocker: Res<ScoreBlocker>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                // let eat_sound = asset_server.load("eat_01.mp3");
                // let speed_up_sound = asset_server.load("speed_up.mp3");
                commands.entity(ent).despawn();
                eat_writer.send(EatEvent);
                score.0 += 1;
                if score.0 <= score_blocker.0 {
                    return
                }

                if score.0 % 10 == 0 {
                    // commands.spawn(AudioBundle {
                    //     source: speed_up_sound,
                    //     ..Default::default()
                    // });
                    // commands.spawn(AudioBundle {
                    //     source: eat_sound,
                    //     ..Default::default()
                    // });
                    tick_accum.0 += crate::BIG_TICK_INCREASE;
                } else {
                    // commands.spawn(AudioBundle {
                    //     source: eat_sound,
                    //     ..Default::default()
                    // });
                    tick_accum.0 += crate::TICK_INCREASE;
                }

                tick_timer.0 = Timer::from_seconds(
                    1. / tick_accum.0,
                    TimerMode::Repeating,
                );
            }
        }
    }
}

pub fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut eat_reader: EventReader<EatEvent>,
) {
    if eat_reader.read().next().is_some() {
        segments.push(add_segment(commands, last_tail_position.0.unwrap()));
    }
}

pub fn update_snake(
    mut timer: ResMut<crate::TickTimer>,
    time: Res<Time>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<crate::GameOverEvent>,
    mut input_queue: ResMut<InputQueue>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if let Some((head_entity, mut head)) = heads.iter_mut().next() {
            if let Some(dir) = input_queue.0.pop_front() {
                if dir != head.rot.opposite() {
                    head.rot = dir;
                }
            }

            let segment_positions = segments
                .iter()
                .map(|e| *positions.get_mut(*e).unwrap())
                .collect::<Vec<Position>>();
            let mut head_pos = positions.get_mut(head_entity).unwrap();

            match &head.rot {
                Direction::Up => head_pos.y += 1,
                Direction::Right => head_pos.x += 1,
                Direction::Down => head_pos.y -= 1,
                Direction::Left => head_pos.x -= 1,
            };

            if head_pos.x < 0
                || head_pos.x > crate::WALL as i32
                || head_pos.y < 0
                || head_pos.y > crate::WALL as i32
            {
                game_over_writer.send(crate::GameOverEvent);
            }

            if segment_positions.contains(&head_pos) {
                game_over_writer.send(crate::GameOverEvent);
            }

            segment_positions
                .iter()
                .zip(segments.iter().skip(1))
                .for_each(|(pos, segment)| {
                    *positions.get_mut(*segment).unwrap() = *pos;
                });

            *last_tail_position =
                LastTailPosition(Some(*segment_positions.last().unwrap()));
        }
    }
}

pub fn control_snake(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_queue: ResMut<InputQueue>,
    mut input_timer: ResMut<InputQueueTimer>,
    time: Res<Time>,
) {
    let dir = if keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::KeyW)
    {
        Direction::Up
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyD)
    {
        Direction::Right
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::KeyS)
    {
        Direction::Down
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::KeyA)
    {
        Direction::Left
    } else {
        if input_timer.0.finished() {
            input_queue.0.clear();
        }
        return;
    };

    input_timer.0.tick(time.delta());

    if input_timer.0.finished() {
        input_timer.0.reset();
        input_queue.0.clear();
        input_queue.0.push_back(dir);
    } else {
        input_timer.0.reset();
        input_queue.0.push_back(dir);
    }
}

fn add_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(75., 75., 75.),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                position.x as f32,
                position.y as f32,
                0.,
            )),
            ..default()
        })
        .insert(Segment)
        .insert(position)
        .insert(crate::Size::square(crate::BLOCK_SIZE))
        .id()
}
