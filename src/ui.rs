use crate::game_mode::{GameRuleChange, GameRules};
use crate::score::{LeaderboardEarned, Score, HIGHSCORES};
use crate::{GameState, ResetEvent};
use bevy::prelude::*;
use bevy_egui::egui::RichText;
use bevy_egui::egui::{self, FontId};
use bevy_egui::EguiContexts;

#[cfg(debug_assertions)]
use crate::debug::{DebugStats, FrameRate};

#[derive(Resource, Default)]
pub struct MenuState {
    leaderboard_confirmation_shown: bool,
}

pub fn setup_ui(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        panel_fill: egui::Color32::TRANSPARENT,
        ..default()
    });
}

pub fn menu_ui(
    mut next_state: ResMut<NextState<GameState>>,
    mut reset_event: EventWriter<ResetEvent>,
    mut game_rule_event: EventWriter<GameRuleChange>,
    mut menu_state: ResMut<MenuState>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(301.0 - 100.0);
            ui.heading(
                RichText::new("Rust Snake").font(FontId::proportional(40.0)),
            );
            let play_button = ui.button(
                RichText::new("Play Classic").font(FontId::proportional(30.0)),
            );
            let walls_button = ui.button(
                RichText::new("Play Walls").font(FontId::proportional(30.0)),
            );
            let leaderboard_button = ui.button(
                RichText::new("View Leaderboard")
                    .font(FontId::proportional(30.0)),
            );
            if play_button.clicked() {
                next_state.set(GameState::Playing);
                reset_event.send(ResetEvent);
            }

            if walls_button.clicked() {
                game_rule_event.send(GameRuleChange(GameRules {
                    do_collide_walls: true,
                    do_spawn_walls: true,
                }));
                next_state.set(GameState::Playing);
                reset_event.send(ResetEvent);
            }

            if leaderboard_button.clicked() {
                menu_state.leaderboard_confirmation_shown = true;
            }
        });
    });

    if menu_state.leaderboard_confirmation_shown {
        egui::Window::new("Confirm?")
            .auto_sized()
            .max_size(egui::Vec2::new(200.0, 40.0))
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(301.0 - 100.0, 301.0 - 20.0))
            .collapsible(false)
            .open(&mut menu_state.leaderboard_confirmation_shown)
            .show(ctx, |ui| {
                ui.label(
                    "Leaderboard entries may not be suitable for all users.",
                );
                ui.vertical_centered(|ui| {
                    if ui.button(
                        RichText::new("Continue")
                            .text_style(egui::TextStyle::Heading),
                    ).clicked() {
                        if let Err(e) = webbrowser::open("https://berintmoffett.com/snake-leaderboard") {
                            error!("An error occurred while trying to open the webpage: {}", e);
                        }
                    }
                });
            });
    }
}

pub fn playing_ui(score: Res<Score>, mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        let score_text_raw = format!("Score: {}", score.0);
        let text =
            RichText::new(score_text_raw).font(FontId::proportional(40.0));
        ui.label(text);
    });
}

pub fn game_over_ui(score: Res<Score>, mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
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
    });
}

pub fn enter_name_ui(
    name: Res<crate::Name>,
    mut contexts: EguiContexts,
    leaderboard_earned: Res<LeaderboardEarned>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            let mut space = 221.0;
            if let LeaderboardEarned::Placed(_) = *leaderboard_earned {
                let highscore_text = RichText::new("New Highscore!")
                    .font(FontId::proportional(40.0));
                space -= 40.0;
                ui.label(highscore_text);
            }
            ui.add_space(space);

            let enter_name_text =
                RichText::new("Enter Name:").font(FontId::proportional(40.0));
            ui.label(enter_name_text);
            let name =
                RichText::new(&(name.0)).font(FontId::proportional(40.0));
            ui.label(name);
        });
    });
}

pub fn viewing_leaderboard_ui(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(301.0 - 140.0);
            let highscore_header =
                RichText::new("Highscores: ").font(FontId::proportional(40.0));
            ui.label(highscore_header);

            let hs_arc = HIGHSCORES.get().unwrap();
            let highscores = hs_arc.lock().unwrap();
            for score in highscores.highscores.iter() {
                let score_text =
                    RichText::new(format!("{}: {}\n", score.name, score.score))
                        .font(FontId::proportional(30.0))
                        .line_height(Some(15.0));
                ui.label(score_text);
            }
            ui.add_space(15.0);

            let prompt = RichText::new("Press any key to continue...")
                .font(FontId::proportional(20.0));
            ui.label(prompt);
        });
    });
}

#[cfg(debug_assertions)]
pub fn debug_ui(
    debug_stats: Res<DebugStats>,
    frame_rate: Res<FrameRate>,
    mut contexts: EguiContexts,
) {
    egui::SidePanel::right("debug")
        .show_separator_line(false)
        .exact_width(50.0)
        .show(contexts.ctx_mut(), |ui| {
            let fps = frame_rate.calc();
            let fps_text_raw = format!("{:.0} fps\n", fps);
            let fps_text = RichText::new(fps_text_raw)
                .color(egui::Color32::GREEN)
                .line_height(Some(7.0));
            let mem_text_raw = format!("{:.0} MB", debug_stats.memory_usage);
            let mem_text =
                RichText::new(mem_text_raw).color(egui::Color32::GREEN);

            ui.label(fps_text);
            ui.label(mem_text);
        });
}
