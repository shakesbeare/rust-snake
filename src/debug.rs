#![cfg(debug_assertions)]

use std::time::Instant;

use bevy::prelude::*;

#[derive(Resource)]
pub struct LastFrameTime {
    pub time: Instant,
}

#[derive(Resource, Default)]
pub struct DebugStats {
    frame_time: f32,
    memory_usage: f32,
}

#[derive(Resource, Default)]
pub struct FrameRate {
    buf: [f32; 30],
    i: usize,
}

 impl FrameRate {
    pub fn new() -> Self {
        FrameRate {
            buf: [0.0; 30],
            i: 0,
        }
    }
    
    fn calc(&self) -> f32 {
        self.buf.iter().sum::<f32>() / 30.
    }

    fn push(&mut self, new_rate: f32) {
        self.buf[self.i] = new_rate;
        self.next_index();
    }

    fn next_index(&mut self) {
        if self.i == 29 {
            self.i = 0;
        } else {
            self.i += 1;
        }
    }
}

#[derive(Component)]
pub struct DebugText;

pub fn set_stats(
    mut frame_time_counter: ResMut<LastFrameTime>,
    mut debug_stats: ResMut<DebugStats>,
    mut frame_rate: ResMut<FrameRate>,
) {
    debug_stats.frame_time = frame_time_counter.time.elapsed().as_millis() as f32;
    frame_time_counter.time = Instant::now();
    frame_rate.push(1000.0 / debug_stats.frame_time);

    let sys = sysinfo::System::new_all();

    debug_stats.memory_usage = crate::PEAK_ALLOC.current_usage_as_mb();
}

pub fn setup_stats_display(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/roboto-thin.ttf"),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: asset_server.load("fonts/roboto-thin.ttf"),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
            TextSection::new(
                "Memory Usage: ",
                TextStyle {
                    font: asset_server.load("fonts/roboto-thin.ttf"),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: asset_server.load("fonts/roboto-thin.ttf"),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
        ])
        .with_text_justify(JustifyText::Right)
        .with_style(Style {
            align_self: AlignSelf::FlexStart,
            position_type: PositionType::Absolute,
            left: Val::Px(400.),
            ..default()
        }),
        DebugText,
    ));
}

pub fn update_stats_display(
    mut fps_text: Query<&mut Text, With<DebugText>>,
    frame_rate: Res<FrameRate>,
    debug_stats: Res<DebugStats>,
) {
    let fps = frame_rate.calc();

    for mut text in fps_text.iter_mut() {
        text.sections[1].value = format!("{:.2}\n", fps);
        text.sections[3].value = format!("{}\n", debug_stats.memory_usage);
    }
}
