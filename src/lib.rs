#![allow(clippy::too_many_arguments)]

pub mod food;
pub mod score;
pub mod snake;
pub mod debug;

use bevy::prelude::*;
use futures::Future;
use score::LeaderboardEarned;
use score::HIGHSCORES;
use snake::TickAccum;

pub const TICK_RATE: f32 = 5.; // Number of updates per second
pub const TICK_INCREASE: f32 = 0.20; // How much to increase tick rate by on eat
pub const BIG_TICK_INCREASE: f32 = 0.50; // How much to increase tick rate by on eat every 10th score
pub const BLOCK_SIZE: f32 = 0.8;
pub const WALL: f32 = 20.;

#[cfg(debug_assertions)]
#[global_allocator]
pub static PEAK_ALLOC: peak_alloc::PeakAlloc = peak_alloc::PeakAlloc;

#[derive(Default, States, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
    ViewingLeaderboard,
    EnterName,
    ReadyToReset,
}

#[derive(Default, States, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum WindowState {
    #[default]
    NeedsScaling,
    Scaled,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
pub struct Size {
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

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Event)]
pub struct ViewLeaderboardEvent;

#[derive(Event)]
pub struct EnterNameEvent;

#[derive(Event)]
pub struct RestartEvent;

#[derive(Resource)]
pub struct TickTimer(pub Timer);

#[cfg(not(target_arch = "wasm32"))]
pub fn run_async<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot start tokio runtime");

        rt.block_on(async move {
            let local = tokio::task::LocalSet::new();
            local
                .run_until(async move {
                    tokio::task::spawn_local(future).await.unwrap();
                })
                .await;
        });
    });
}

#[cfg(target_arch = "wasm32")]
pub fn run_async<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    wasm_bindgen_futures::spawn_local(async move {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                tokio::task::spawn_local(future).await.unwrap();
            })
            .await;
    });
}

pub fn size_scaling(
    mut windows: Query<&mut Window>,
    mut query: Query<(&Size, &mut Transform)>,
    mut next_state: ResMut<NextState<WindowState>>,
) {
    let window = windows.single_mut();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / WALL
                * window.width()
                * (window.height() / window.width()),
            sprite_size.height / WALL * window.height(),
            1.,
        )
    }

    next_state.set(WindowState::Scaled);
}

fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
}

pub fn position_translation(
    mut windows: Query<&mut Window>,
    mut query: Query<(&Position, &mut Transform)>,
) {
    let window = windows.single_mut();
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width(), WALL),
            convert(pos.y as f32, window.height(), WALL),
            0.,
        )
    }
}

pub fn setup(
    mut commands: Commands,
    mut eat_writer: EventWriter<crate::snake::EatEvent>,
    mut acquire_highscores: EventWriter<crate::score::AcquireHighscores>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    eat_writer.send(crate::snake::EatEvent);
    acquire_highscores.send(crate::score::AcquireHighscores);

    // Preload assets before the game begins
    let _ = asset_server.load::<Font>("fonts/roboto-thin.ttf");
    // let _ = asset_server.load::<AudioSource>("eat_01.ogg");
    // let _ = asset_server.load::<AudioSource>("speed_up.ogg");

    commands
        .spawn(
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
            .with_text_justify(JustifyText::Left)
            .with_style(Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                ..default()
            }),
        )
        .insert(crate::score::ScoreText);
}

#[allow(clippy::too_many_arguments)]
pub fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    food: Query<Entity, With<crate::food::Food>>,
    text: Query<Entity, With<crate::score::ScoreText>>,
    segments: Query<Entity, With<crate::snake::Segment>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    asset_server: Res<AssetServer>,
    score: Res<crate::score::Score>,
    mut enter_name_event: EventWriter<EnterNameEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // despawn all text, snake segments, and food
    if reader.read().next().is_some() {
        next_state.set(GameState::GameOver);

        for ent in food.iter().chain(segments.iter()).chain(text.iter()) {
            commands.entity(ent).despawn();
        }

        // Game Over screen
        commands
            .spawn(
                TextBundle::from_sections([
                    TextSection::new(
                        "Game Over\n",
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 50.,
                            color: Color::BEIGE,
                        },
                    ),
                    TextSection::new(
                        "Score: ",
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 50.,
                            color: Color::BEIGE,
                        },
                    ),
                    TextSection::new(
                        format!("{}", score.0),
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 50.,
                            color: Color::BEIGE,
                        },
                    ),
                    TextSection::new(
                        "\nPress any key to continue",
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 35.,
                            color: Color::BEIGE,
                        },
                    ),
                ])
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                }),
            )
            .insert(crate::score::ScoreText);
    }

    if state.get() == &GameState::GameOver
        && keyboard_input.get_pressed().next().is_some()
    {
        next_state.set(GameState::EnterName);
        enter_name_event.send(EnterNameEvent);
    }
}

pub fn enter_name_screen(
    score: Res<crate::score::Score>,
    text: Query<Entity, With<crate::score::ScoreText>>,
    asset_server: Res<AssetServer>,
    mut leaderboard_place_earned: ResMut<crate::score::LeaderboardEarned>,
    mut enter_name_event: EventReader<EnterNameEvent>,
    mut commands: Commands,
) {
    if enter_name_event.read().next().is_some() {
        for ent in text.iter() {
            commands.entity(ent).despawn();
        }

        let hs_arc = HIGHSCORES.get().unwrap();
        let highscores = hs_arc.lock().unwrap();
        let mut i = 0;
        while i < 5 && score.0 > highscores.highscores[i].score {
            i += 1;
        }

        let mut highscore_text =
            format!("Your score: {}\nEnter Name:\n", score.0);
        if i > 0 {
            *leaderboard_place_earned = LeaderboardEarned::Placed(i as u8);
            highscore_text =
                format!("New Highscore!: {}\nEnter Name:", score.0);
        } else {
            *leaderboard_place_earned = LeaderboardEarned::NotPlaced;
        }

        commands
            .spawn(
                TextBundle::from_sections([
                    TextSection::new(
                        highscore_text,
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 50.,
                            color: Color::BEIGE,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: asset_server.load("fonts/roboto-thin.ttf"),
                            font_size: 35.,
                            color: Color::BEIGE,
                        },
                    ),
                ])
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                }),
            )
            .insert(crate::score::ScoreText);
    }
}

pub fn enter_name(
    mut next_state: ResMut<NextState<GameState>>,
    mut ev_char: EventReader<ReceivedCharacter>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut string: Local<String>,
    mut text: Query<&mut Text, With<crate::score::ScoreText>>,
    mut leaderboard_event: EventWriter<ViewLeaderboardEvent>,
    mut acquire_highscores: EventWriter<crate::score::AcquireHighscores>,
    mut send_highscores: EventWriter<crate::score::SendHighscores>,
    score: Res<crate::score::Score>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        *string = string.chars().filter(|c| c.is_alphanumeric()).collect();
        if (*string).is_empty() {
            *string = "Anonymous".to_string();
        }
        let highscore = crate::score::Highscore {
            name: string.clone(),
            score: score.0,
        };

        // send highscore to server
        send_highscores.send(crate::score::SendHighscores(highscore));
        acquire_highscores.send(crate::score::AcquireHighscores);

        string.clear();

        // show leaderboard
        next_state.set(GameState::ViewingLeaderboard);
        leaderboard_event.send(ViewLeaderboardEvent);
    }

    // wasm does not send ReceivedCharacter events for backspace
    #[cfg(target_arch = "wasm32")]
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        string.pop();
    }

    for ev in ev_char.read() {
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            string.pop();
            continue;
        }
        // if keyboard_input.just_pressed(KeyCode::Backspace) {
        // }
        if string.len() <= 10 && ev.char.is_ascii() {
            if ev.char == "\n" || ev.char == "\r" {
                continue;
            }
            string.push(ev.char.chars().next().unwrap());
        }
    }

    for mut text in text.iter_mut() {
        if string.is_empty() {
            text.sections[1].value = " ".to_string();
        } else {
            text.sections[1].value = string.to_string();
        }
    }
}

pub fn leaderboard(
    mut commands: Commands,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    text: Query<Entity, With<crate::score::ScoreText>>,
    asset_server: Res<AssetServer>,
    mut leaderboard_event: EventReader<ViewLeaderboardEvent>,
) {
    if leaderboard_event.read().next().is_some() {
        for ent in text.iter() {
            commands.entity(ent).despawn();
        }
        let mut texts = vec![];
        let hs_arc = HIGHSCORES.get().unwrap();
        let highscores = hs_arc.lock().unwrap();
        for score in highscores.highscores.iter() {
            texts.push(TextSection::new(
                format!("{}: {}\n", score.name, score.score),
                TextStyle {
                    font: asset_server.load("fonts/roboto-thin.ttf"),
                    font_size: 40.,
                    color: Color::BEIGE,
                },
            ));
        }

        // if not viewing leaderboard and state is game over,
        // pressing any key advances the state to view the leaderboard

        if state.get() == &GameState::GameOver
            && keyboard_input.get_pressed().next().is_some()
        {
            next_state.set(GameState::ViewingLeaderboard);
        }

        if state.get() == &GameState::ViewingLeaderboard {
            // despawn all text
            // spawn leaderboard text
            for ent in text.iter() {
                commands.entity(ent).despawn();
            }

            let mut texts = texts.into_iter();

            // leaderboard screen
            commands
                .spawn(
                    TextBundle::from_sections([
                        TextSection::new(
                            "High Scores\n\n",
                            TextStyle {
                                font: asset_server
                                    .load("fonts/roboto-thin.ttf"),
                                font_size: 50.,
                                color: Color::BEIGE,
                            },
                        ),
                        texts.next().unwrap(),
                        texts.next().unwrap(),
                        texts.next().unwrap(),
                        texts.next().unwrap(),
                        texts.next().unwrap(),
                        TextSection::new(
                            "\nPress any key to continue",
                            TextStyle {
                                font: asset_server
                                    .load("fonts/roboto-thin.ttf"),
                                font_size: 35.,
                                color: Color::BEIGE,
                            },
                        ),
                    ])
                    .with_text_justify(JustifyText::Center)
                    .with_style(Style {
                        justify_self: JustifySelf::Center,
                        align_self: AlignSelf::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    }),
                )
                .insert(crate::score::ScoreText);
        }
    } else if keyboard_input.get_pressed().next().is_some() {
        next_state.set(GameState::ReadyToReset);
    }
}

pub fn awaiting_reset(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut eat_writer: EventWriter<crate::snake::EatEvent>,
    segments_res: ResMut<crate::snake::SnakeSegments>,
    last_tail_position: ResMut<crate::snake::LastTailPosition>,
    mut tick_timer: ResMut<crate::TickTimer>,
    mut score: ResMut<crate::score::Score>,
    mut next_direction: ResMut<crate::snake::NextDirection>,
    mut tick_accum: ResMut<TickAccum>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    text: Query<Entity, With<crate::score::ScoreText>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        for ent in text.iter() {
            commands.entity(ent).despawn();
        }
        commands
            .spawn(
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
                .with_text_justify(JustifyText::Left)
                .with_style(Style {
                    align_self: AlignSelf::FlexStart,
                    position_type: PositionType::Absolute,
                    ..default()
                }),
            )
            .insert(crate::score::ScoreText);
        next_state.set(GameState::Playing);
        crate::snake::add_snake(commands, segments_res, last_tail_position);
        eat_writer.send(crate::snake::EatEvent);
        next_direction.0 = Some(crate::snake::Direction::Left);
        tick_accum.0 = TICK_RATE;
        tick_timer.0 =
            Timer::from_seconds(1. / TICK_RATE, TimerMode::Repeating);
        score.0 = 0;
    }
}
