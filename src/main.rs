use bevy::prelude::*;
use rand::prelude::random;

/*
*
* CONSTANTS
*
*/
const TICK_RATE: f32 = 3.; // Number of updates per second
const TICK_INCREASE: f32 = 0.25; // How much to increase tick rate by on eat
const BLOCK_SIZE: f32 = 0.8;
const WALL: f32 = 20.;

fn main() {
    App::new()
        .add_startup_system_set(
            SystemSet::new()
                .with_system(food_spawner)
                .with_system(setup.after(food_spawner))
                .with_system(add_snake.after(setup)),
        )
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(TickTimer(Timer::from_seconds(1. / TICK_RATE, true)))
        .insert_resource(WindowDescriptor {
            title: "ShakeSnake".to_string(),
            width: 800.,
            height: 800.,
            ..default()
        })
        .insert_resource(ClearColor(Color::hex("1d2021").unwrap()))
        .insert_resource(NextDirection::default())
        .insert_resource(Score::default())
        .add_event::<EatEvent>()
        .add_event::<GameOverEvent>()
        .add_system(food_spawner)
        .add_system_set(
            SystemSet::new()
                .with_system(turn_snake.before(update_snake))
                .with_system(update_snake)
                .with_system(snake_eating.after(update_snake))
                .with_system(snake_growth.after(snake_eating)),
        )
        .add_system(game_over)
        .add_system(score_update)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}

/*
*
* NON-COMPONENT STRUCTS
*
*/

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Direction {
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

#[derive(Default, Debug, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default, Debug)]
struct LastTailPosition(Option<Position>);

#[derive(Default, Debug)]
struct NextDirection(Option<Direction>);

#[derive(Default)]
struct Score(i32);

struct EatEvent;
struct GameOverEvent;
struct TickTimer(Timer);

/*
*
* COMPONENTS
*
*/

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
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
struct SnakeHead {
    rot: Direction,
}

#[derive(Component)]
struct Segment;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct ScoreText;

/*
*
* SYSTEMS
*
*/

fn add_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
) {
    *segments = SnakeSegments(vec![commands
        .spawn_bundle(SpriteBundle {
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
        .insert(Size::square(BLOCK_SIZE))
        .id()]);
    *last_tail_position = LastTailPosition(Some(Position { x: 11, y: 10 }));
}

fn generate_food_coords() -> Position {
    let x = (random::<f32>() * WALL as f32) as i32;
    let y = (random::<f32>() * WALL as f32) as i32;

    Position { x, y }
}
fn food_spawner(
    mut commands: Commands,
    mut eat_reader: EventReader<EatEvent>,
    segments: Res<SnakeSegments>,
    mut positions: Query<&mut Position, With<Segment>>,
) {
    // check if eat_reader event chain has value
    if eat_reader.iter().next().is_some() {
        // grab positions of snake segments
        let segment_positions = segments
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        // generate food pos until it doesn't overlap with snake
        let mut food_pos = generate_food_coords();
        while segment_positions.contains(&food_pos) {
            food_pos = generate_food_coords();
        }
        // spawn food
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(175., 0., 0.),
                    ..default()
                },
                ..default()
            })
            .insert(Food)
            .insert(food_pos)
            .insert(Size::square(BLOCK_SIZE));
    }
}

fn snake_eating(
    mut commands: Commands,
    mut eat_writer: EventWriter<EatEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
    mut score: ResMut<Score>,
    mut tick_timer: ResMut<TickTimer>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                eat_writer.send(EatEvent);
                score.0 += 1;

                let new_speed =
                    TICK_RATE + (score.0 as f32 * TICK_INCREASE) as f32;
                tick_timer.0 = Timer::from_seconds(1. / new_speed, true);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut eat_reader: EventReader<EatEvent>,
) {
    if eat_reader.iter().next().is_some() {
        segments.push(add_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn add_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(75., 75., 75.),
                ..default()
            },
            ..default()
        })
        .insert(Segment)
        .insert(position)
        .insert(Size::square(BLOCK_SIZE))
        .id()
}

fn update_snake(
    mut timer: ResMut<TickTimer>,
    time: Res<Time>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    next_direction: Res<NextDirection>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if let Some((head_entity, mut head)) = heads.iter_mut().next() {
            match next_direction.0 {
                Some(dir) => {
                    head.rot = dir;
                }
                None => {}
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
                || head_pos.x > WALL as i32
                || head_pos.y < 0
                || head_pos.y > WALL as i32
            {
                game_over_writer.send(GameOverEvent);
            }

            if segment_positions.contains(&head_pos) {
                game_over_writer.send(GameOverEvent);
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

fn turn_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_direction: ResMut<NextDirection>,
    query: Query<&SnakeHead>,
) {
    let dir = if keyboard_input.pressed(KeyCode::Up) {
        Some(Direction::Up)
    } else if keyboard_input.pressed(KeyCode::Right) {
        Some(Direction::Right)
    } else if keyboard_input.pressed(KeyCode::Down) {
        Some(Direction::Down)
    } else if keyboard_input.pressed(KeyCode::Left) {
        Some(Direction::Left)
    } else {
        next_direction.0
    };

    let head = query.single();

    match dir {
        Some(x) => {
            if x != head.rot.opposite() {
                next_direction.0 = dir;
            }
        }
        None => {}
    }
}

fn size_scaling(
    windows: Res<Windows>,
    mut query: Query<(&Size, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / WALL as f32
                * window.width() as f32
                * (window.height() / window.width()),
            sprite_size.height / WALL as f32 * window.height() as f32,
            1.,
        )
    }
}

fn position_translation(
    windows: Res<Windows>,
    mut query: Query<(&Position, &mut Transform)>,
) {
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

fn setup(
    mut commands: Commands,
    mut eat_writer: EventWriter<EatEvent>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    eat_writer.send(EatEvent);

    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        font: asset_server.load("fonts/roboto-thin.ttf"),
                        font_size: 50.,
                        color: Color::BEIGE,
                    },
                ),
                TextSection::new(
                    "0",
                    TextStyle {
                        font: asset_server.load("fonts/roboto-thin.ttf"),
                        font_size: 50.,
                        color: Color::BEIGE,
                    },
                ),
            ])
            .with_text_alignment(TextAlignment::TOP_LEFT)
            .with_style(Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.),
                    left: Val::Px(5.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ScoreText);
}

fn score_update(
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut text in query.iter_mut() {
        text.sections[1].value = format!("{}", score.0)
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<Segment>>,
    mut eat_writer: EventWriter<EatEvent>,
    last_tail_position: ResMut<LastTailPosition>,
    mut next_direction: ResMut<NextDirection>,
    mut tick_timer: ResMut<TickTimer>,
    mut score: ResMut<Score>,
) {
    if reader.iter().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        add_snake(commands, segments_res, last_tail_position);
        eat_writer.send(EatEvent);
        next_direction.0 = Some(Direction::Left);
        tick_timer.0 = Timer::from_seconds(1. / TICK_RATE, true);
        score.0 = 0;
    }
}
