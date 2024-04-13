#![cfg(debug_assertions)]

use std::time::Instant;

use bevy::prelude::*;

#[derive(Resource)]
pub struct LastFrameTime {
    pub time: Instant,
}

#[derive(Resource, Default)]
pub struct DebugStats {
    pub frame_time: f32,
    pub memory_usage: f32,
}

#[derive(Resource)]
pub struct FrameRate {
    buf: [f32; 40],
    i: usize,
}

impl FrameRate {
    pub fn new() -> Self {
        FrameRate {
            buf: [0.0; 40],
            i: 0,
        }
    }

    pub fn calc(&self) -> f32 {
        self.buf.iter().sum::<f32>() / 40.
    }

    fn push(&mut self, new_rate: f32) {
        self.buf[self.i] = new_rate;
        self.next_index();
    }

    fn next_index(&mut self) {
        if self.i == 39 {
            self.i = 0;
        } else {
            self.i += 1;
        }
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn set_stats(
    mut frame_time_counter: ResMut<LastFrameTime>,
    mut debug_stats: ResMut<DebugStats>,
    mut frame_rate: ResMut<FrameRate>,
) {
    debug_stats.frame_time =
        frame_time_counter.time.elapsed().as_millis() as f32;
    frame_time_counter.time = Instant::now();
    frame_rate.push(1000.0 / debug_stats.frame_time);

    debug_stats.memory_usage = crate::PEAK_ALLOC.current_usage_as_mb();
}
