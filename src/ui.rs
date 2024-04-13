use bevy::prelude::*;
use bevy_egui::egui::RichText;
use bevy_egui::egui::{self, FontId};
use bevy_egui::EguiContexts;
use crate::score::{LeaderboardEarned, Score, HIGHSCORES};
use crate::GameState;

#[cfg(debug_assertions)]
use crate::debug::{DebugStats, FrameRate};

pub fn setup_ui(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        panel_fill: egui::Color32::TRANSPARENT,
        ..default()
    });
}

// TODO break up this horrific function into multiple??
pub fn ui_system(
    mut contexts: EguiContexts,
    score: Res<Score>,
    game_state: Res<State<GameState>>,
    leaderboard_earned: Res<LeaderboardEarned>,
    name: Res<crate::Name>,
    #[cfg(debug_assertions)]
    debug_stats: Res<DebugStats>,
    #[cfg(debug_assertions)]
    frame_rate: Res<FrameRate>,
) {
    let ctx = contexts.ctx_mut();
    let state = game_state.get();
    egui::CentralPanel::default().show(ctx, |ui| {
        if state == &GameState::Playing {
            let score_text_raw = format!("Score: {}", score.0);
            let text =
                RichText::new(score_text_raw).font(FontId::proportional(40.0));
            ui.label(text);
        } else if state == &GameState::GameOver {
            ui.vertical_centered(|ui| {
                ui.add_space(201.0); // 301.0 - 40 * 2 - 20
                let game_over =
                    RichText::new("Game Over").font(FontId::proportional(40.0));
                ui.label(game_over);
                let score_header = RichText::new(format!("Score: {}", score.0))
                    .font(FontId::proportional(40.0));
                ui.label(score_header);
                let prompt = RichText::new("Press any key to continue...")
                    .font(FontId::proportional(20.0));
                ui.label(prompt);
            });
        } else if state == &GameState::EnterName {
            ui.vertical_centered(|ui| {
                let mut space = 221.0;
                if let LeaderboardEarned::Placed(_) = *leaderboard_earned {
                    let highscore_text = RichText::new("New Highscore!")
                        .font(FontId::proportional(40.0));
                    space -= 40.0;
                    ui.label(highscore_text);
                }
                ui.add_space(space);

                let enter_name_text = RichText::new("Enter Name:")
                    .font(FontId::proportional(40.0));
                ui.label(enter_name_text);
                let name =
                    RichText::new(&(name.0)).font(FontId::proportional(40.0));
                ui.label(name);
            });
        } else if state == &GameState::ViewingLeaderboard
            || state == &GameState::ReadyToReset
        {
            ui.vertical_centered(|ui| {
                ui.add_space(301.0 - 140.0);
                let highscore_header = RichText::new("Highscores: ")
                    .font(FontId::proportional(40.0));
                ui.label(highscore_header);

                let hs_arc = HIGHSCORES.get().unwrap();
                let highscores = hs_arc.lock().unwrap();
                for score in highscores.highscores.iter() {
                    let score_text = RichText::new(format!(
                        "{}: {}\n",
                        score.name, score.score
                    ))
                    .font(FontId::proportional(30.0))
                    .line_height(Some(15.0));
                    ui.label(score_text);
                }
                ui.add_space(15.0);

                let prompt = RichText::new("Press any key to continue...")
                    .font(FontId::proportional(20.0));
                ui.label(prompt);
            });
        }
    });

    #[cfg(debug_assertions)]
    {
        egui::SidePanel::right("debug")
            .show_separator_line(false)
            .exact_width(50.0)
            .show(ctx, |ui| {
                let fps = frame_rate.calc();
                let fps_text_raw = format!("{:.0} fps\n", fps);
                let fps_text = RichText::new(fps_text_raw)
                    .color(egui::Color32::GREEN)
                    .line_height(Some(7.0));
                let mem_text_raw =
                    format!("{:.0} MB", debug_stats.memory_usage);
                let mem_text =
                    RichText::new(mem_text_raw).color(egui::Color32::GREEN);

                ui.label(fps_text);
                ui.label(mem_text);
            });
    }
}
