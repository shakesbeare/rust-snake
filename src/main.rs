use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use rust_snake::cheats::*;
use rust_snake::food::*;
use rust_snake::game_mode::*;
use rust_snake::score::*;
use rust_snake::snake::*;
use rust_snake::ui::*;
use rust_snake::*;

#[cfg(debug_assertions)]
use rust_snake::debug::*;

fn main() {
    #[cfg(target_arch = "wasm32-unknown-unknown")]
    {
        wasm_logger::init(wasm_logger::Config::default());
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    let mut app = App::new();

    // Insert resources
    app.insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(TickTimer(Timer::from_seconds(
            1. / TICK_RATE,
            TimerMode::Repeating,
        )))
        .insert_resource(ClearColor(Color::hex("1d2021").unwrap()))
        .insert_resource(NextDirection::default())
        .insert_resource(Score::default())
        .insert_resource(LeaderboardEarned::NotPlaced)
        .insert_resource(LastPressed::default())
        .insert_resource(InputQueue::default())
        .insert_resource(InputQueueTimer(Timer::from_seconds(
            1. / 2. * TICK_RATE,
            TimerMode::Once,
        )))
        .insert_resource(TickAccum(TICK_RATE))
        .insert_resource(ScoreBlocker(0))
        .insert_resource(rust_snake::Name("".to_string()))
        .insert_resource(MenuState::default())
        .insert_resource(GameRules::default())
        .insert_resource(WallQueue::default());

    // States and Resources
    app.init_state::<GameState>()
        .init_state::<ScoresDownloaded>()
        .init_state::<WindowState>()
        .add_event::<EatEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<CalcHighscoresEvent>()
        .add_event::<ViewLeaderboardEvent>()
        .add_event::<AcquireHighscores>()
        .add_event::<TriggerDownload>()
        .add_event::<SendHighscores>()
        .add_event::<ResetEvent>()
        .add_event::<GameRuleChange>();

    // Systems ----------------
    // Startup
    app.add_systems(Startup, (init_scores, setup, setup_ui).chain());

    // Update
    // -- Core
    app.add_systems(
        Update,
        (
            food_spawner,
            control_snake,
            update_snake,
            snake_eating,
            snake_growth,
            quick_speed,
            quick_reset,
            game_over,
            position_translation,
        )
            .chain()
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(Update, game_over.run_if(in_state(GameState::GameOver)))
    .add_systems(Update, enter_name.run_if(in_state(GameState::EnterName)))
    .add_systems(Update, reset_game)
    .add_systems(Update, enqueue_walls)
    .add_systems(Update, try_spawn_walls.run_if(in_state(GameState::Playing)))
    .add_systems(Update, game_rule_changer);

    // -- UI
    app.add_systems(Update, menu_ui.run_if(in_state(GameState::MainMenu)))
        .add_systems(Update, playing_ui.run_if(in_state(GameState::Playing)))
        .add_systems(Update, game_over_ui.run_if(in_state(GameState::GameOver)))
        .add_systems(
            Update,
            enter_name_ui.run_if(in_state(GameState::EnterName)),
        )
        .add_systems(
            Update,
            viewing_leaderboard_ui
                .run_if(in_state(GameState::ViewingLeaderboard)),
        )
        .add_systems(
            Update,
            viewing_leaderboard_ui.run_if(in_state(GameState::ReadyToReset)),
        );

    #[cfg(debug_assertions)]
    {
        app.add_systems(Update, debug_ui);
    }

    // -- Graphics
    app.add_systems(Update, size_scaling)
        .add_systems(
            Update,
            calc_highscores.run_if(in_state(GameState::EnterName)),
        )
        .add_systems(
            Update,
            leaderboard.run_if(in_state(GameState::ViewingLeaderboard)),
        )
        .add_systems(
            Update,
            awaiting_reset.run_if(in_state(GameState::ReadyToReset)),
        );

    // Network
    app.add_systems(
        Update,
        (
            download_manager,
            download_scores
                .run_if(in_state(ScoresDownloaded::NotDownloaded))
                .chain(),
        ),
    )
    .add_systems(Update, upload_scores);

    // plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#game".into()),
                    resolution: bevy::window::WindowResolution::new(602., 602.),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                #[cfg(debug_assertions)]
                level: Level::DEBUG,
                #[cfg(debug_assertions)]
                filter: "info,wgpu_core=warn,wgpu_hal=warn,rust_snake=debug"
                    .into(),
                #[cfg(not(debug_assertions))]
                level: Level::ERROR,
                #[cfg(not(debug_assertions))]
                filter: "".to_string(),
                update_subscriber: None,
            }),
    )
    .add_plugins(EguiPlugin);

    // Debug only!!
    #[cfg(debug_assertions)]
    {
        app.add_systems(Update, (set_stats /*update_stats_display*/,));
        // app.add_systems(Startup, setup_stats_display);
        app.insert_resource::<DebugStats>(DebugStats::default())
            .insert_resource::<FrameRate>(FrameRate::new())
            .insert_resource::<LastFrameTime>(LastFrameTime {
                time: std::time::Instant::now(),
            });
    }

    app.run();
}
