use bevy::prelude::*;

use crate::{
    snake::{EatEvent, SnakeHead, SnakeSegments},
    Position,
};

#[derive(Resource, Default, Clone, Copy)]
pub struct GameRules {
    pub do_collide_walls: bool,
    pub do_spawn_walls: bool,
}

#[derive(Event)]
pub struct GameRuleChange(pub GameRules);

#[derive(Resource, Default)]
pub struct WallQueue {
    positions: Vec<Position>,
}

#[derive(Component)]
pub struct Wall;

pub fn game_rule_changer(
    mut game_rule_change_event: EventReader<GameRuleChange>,
    mut game_rules: ResMut<GameRules>,
) {
    for game_rule_change in game_rule_change_event.read() {
        *game_rules = game_rule_change.0;
    }
}

pub fn enqueue_walls(
    mut eat_event: EventReader<EatEvent>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut wall_queue: ResMut<WallQueue>,
    game_rules: Res<GameRules>,
) {
    let eat_event = eat_event.read().next();
    if eat_event.is_none() || !game_rules.do_spawn_walls || eat_event.unwrap().0 {
        return;
    } 

    if let Some((head_entity, _)) = heads.iter_mut().next() {
        let head_pos = positions.get_mut(head_entity).unwrap();
        wall_queue.positions.push(*head_pos);
    }
}

pub fn try_spawn_walls(
    mut commands: Commands,
    segments: ResMut<SnakeSegments>,
    mut wall_queue: ResMut<WallQueue>,
    mut positions: Query<&mut Position>,
) {
    let segment_positions = segments
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>();
    wall_queue.positions = wall_queue
        .positions
        .iter()
        .filter_map(|pos| {
            if segment_positions.contains(pos) {
                Some(*pos)
            } else {
                debug!("Spawning wall at {:?}", pos);
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GRAY,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(*pos)
                    .insert(crate::Size::square(crate::BLOCK_SIZE))
                    .insert(Wall);
                None
            }
        })
        .collect();
}
