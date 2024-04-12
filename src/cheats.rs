use bevy::prelude::*;

use crate::{snake::TickAccum, ResetEvent, BIG_TICK_INCREASE, TICK_INCREASE, TICK_RATE};

#[derive(Resource)]
pub struct ScoreBlocker(pub u32);

pub fn quick_speed(
    mut tick_timer: ResMut<crate::TickTimer>,
    score: ResMut<crate::score::Score>,
    mut tick_accum: ResMut<TickAccum>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut score_blocker: ResMut<ScoreBlocker>,
) {
    if keyboard_input.pressed(KeyCode::KeyP) && score.0 == 0 && score_blocker.0 == 0 {
        tick_accum.0 =
            TICK_RATE + 27.0 * TICK_INCREASE + 3.0 * BIG_TICK_INCREASE;
        tick_timer.0 =
            Timer::from_seconds(1. / tick_accum.0, TimerMode::Repeating);
        score_blocker.0 = 30;
    }
}

pub fn quick_reset(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    if keyboard_input.pressed(KeyCode::Backslash) {
        reset_writer.send(ResetEvent);
    }
}
