#![allow(clippy::too_many_arguments)]

pub mod cheats;
pub mod debug;
pub mod food;
pub mod game_mode;
pub mod score;
pub mod snake;
pub mod ui;

use bevy::prelude::*;
use cheats::ScoreBlocker;
use futures::Future;
use rand::random;
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
    MainMenu,
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
pub struct CalcHighscoresEvent;

#[derive(Event)]
pub struct RestartEvent;

#[derive(Resource)]
pub struct TickTimer(pub Timer);

#[derive(Resource)]
pub struct Name(pub String);

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
    mut acquire_highscores: EventWriter<crate::score::AcquireHighscores>,
) {
    commands.spawn(Camera2dBundle::default());
    acquire_highscores.send(crate::score::AcquireHighscores);

    // Preload assets before the game begins
    let border_color = Color::BEIGE;
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(1.0, 602.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-301.0, 0.0, 0.0)),
        ..default()
    });
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(1.0, 602.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(301.0, 0.0, 0.0)),
        ..default()
    });
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(602.0, 1.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 301.0, 0.0)),
        ..default()
    });
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(602.0, 1.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, -301.0, 0.0)),
        ..default()
    });
}

#[allow(clippy::too_many_arguments)]
pub fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    food: Query<Entity, With<crate::food::Food>>,
    segments: Query<Entity, With<crate::snake::Segment>>,
    walls: Query<Entity, With<crate::game_mode::Wall>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut enter_name_event: EventWriter<CalcHighscoresEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // despawn all text, snake segments, and food
    if reader.read().next().is_some() {
        next_state.set(GameState::GameOver);

        for ent in food.iter().chain(segments.iter()).chain(walls.iter()) {
            commands.entity(ent).despawn();
        }
    }

    if state.get() == &GameState::GameOver
        && keyboard_input.get_pressed().next().is_some()
    {
        next_state.set(GameState::EnterName);
        enter_name_event.send(CalcHighscoresEvent);
    }
}

pub fn calc_highscores(
    score: Res<crate::score::Score>,
    mut leaderboard_place_earned: ResMut<crate::score::LeaderboardEarned>,
    mut calc_highscores_event: EventReader<CalcHighscoresEvent>,
) {
    if calc_highscores_event.read().next().is_some() {
        let hs_arc = HIGHSCORES.get().unwrap();
        let highscores = hs_arc.lock().unwrap();
        let mut i = 0;
        while i < 5 && score.0 > highscores.highscores[i].score {
            i += 1;
        }

        if i > 0 {
            *leaderboard_place_earned = LeaderboardEarned::Placed(i as u8);
        } else {
            *leaderboard_place_earned = LeaderboardEarned::NotPlaced;
        }
    }
}

pub fn enter_name(
    mut next_state: ResMut<NextState<GameState>>,
    mut ev_char: EventReader<ReceivedCharacter>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut name: ResMut<Name>,
    mut leaderboard_event: EventWriter<ViewLeaderboardEvent>,
    mut acquire_highscores: EventWriter<crate::score::AcquireHighscores>,
    mut send_highscores: EventWriter<crate::score::SendHighscores>,
    score: Res<crate::score::Score>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        name.0 = name.0.chars().filter(|c| c.is_alphanumeric()).collect();
        if name.0.is_empty() {
            name.0 = "Anonymous".to_string();
        }
        let highscore = crate::score::Highscore {
            name: name.0.clone(),
            score: score.0,
        };

        // send highscore to server
        send_highscores.send(crate::score::SendHighscores(highscore));
        acquire_highscores.send(crate::score::AcquireHighscores);

        name.0.clear();

        // show leaderboard
        next_state.set(GameState::ViewingLeaderboard);
        leaderboard_event.send(ViewLeaderboardEvent);
    }

    // wasm does not send ReceivedCharacter events for backspace
    #[cfg(target_arch = "wasm32")]
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        name.0.pop();
    }

    for ev in ev_char.read() {
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            name.0.pop();
            continue;
        }
        // if keyboard_input.just_pressed(KeyCode::Backspace) {
        // }
        if name.0.len() <= 10 && ev.char.is_ascii() {
            if ev.char == "\n" || ev.char == "\r" {
                continue;
            }
            name.0.push(ev.char.chars().next().unwrap());
        }
    }
}

pub fn leaderboard(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.get_pressed().next().is_some() {
        next_state.set(GameState::ReadyToReset);
    }
}

#[derive(Event)]
pub struct ResetEvent;

pub fn awaiting_reset(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        reset_writer.send(ResetEvent);
    }
}

/// Reset game when reset event is sent
pub fn reset_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut eat_writer: EventWriter<crate::snake::EatEvent>,
    segments_res: ResMut<crate::snake::SnakeSegments>,
    mut next_direction: ResMut<crate::snake::NextDirection>,
    mut tick_accum: ResMut<TickAccum>,
    mut tick_timer: ResMut<crate::TickTimer>,
    mut score: ResMut<crate::score::Score>,
    mut score_blocker: ResMut<ScoreBlocker>,
    last_tail_position: ResMut<crate::snake::LastTailPosition>,
    mut reset_reader: EventReader<ResetEvent>,
    food: Query<Entity, With<crate::food::Food>>,
    segments: Query<Entity, With<crate::snake::Segment>>,
) {
    if reset_reader.read().next().is_none() {
        return;
    }

    for ent in food.iter().chain(segments.iter()) {
        commands.entity(ent).despawn();
    }
    let rand: u8 = random();
    let dir: snake::Direction = rand.into();

    next_state.set(GameState::Playing);
    crate::snake::add_snake(commands, segments_res, last_tail_position, dir);
    eat_writer.send(crate::snake::EatEvent(true));
    next_direction.0 = Some(dir);
    tick_accum.0 = TICK_RATE;
    tick_timer.0 = Timer::from_seconds(1. / TICK_RATE, TimerMode::Repeating);
    score.0 = 0;
    score_blocker.0 = 0;
}
