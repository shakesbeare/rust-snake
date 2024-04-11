use bevy::prelude::*;
use rand::prelude::random;

#[derive(Component)]
pub struct Food;

pub fn generate_food_coords() -> crate::Position {
    let x = (random::<f32>() * crate::WALL) as i32;
    let y = (random::<f32>() * crate::WALL) as i32;

    // reduce probability that food spawns on the wall
    // 10% chance to regenerate if food is on the wall
    if (x == 0
        || x == crate::WALL as i32 - 1
        || y == 0
        || y == crate::WALL as i32 - 1)
        && random::<f32>() > 0.9
    {
        return generate_food_coords();
    }

    crate::Position { x, y }
}

pub fn food_spawner(
    mut commands: Commands,
    mut eat_reader: EventReader<crate::snake::EatEvent>,
    segments: Res<crate::snake::SnakeSegments>,
    mut positions: Query<&mut crate::Position, With<crate::snake::Segment>>,
) {
    // check if eat_reader event chain has value
    if eat_reader.read().next().is_some() {
        // grab positions of snake segments
        let segment_positions = segments
            .iter()
            .map(|e| match positions.get_mut(*e) {
                Ok(p) => *p,
                Err(_) => crate::Position {
                    x: i32::MAX,
                    y: i32::MAX,
                },
            })
            .collect::<Vec<crate::Position>>();

        // generate food pos until it doesn't overlap with snake
        let mut food_pos = generate_food_coords();
        while segment_positions.contains(&food_pos) {
            food_pos = generate_food_coords();
        }
        // spawn food
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(175., 0., 0.),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    food_pos.x as f32,
                    food_pos.y as f32,
                    0.,
                )),
                ..default()
            })
            .insert(Food)
            .insert(food_pos)
            .insert(crate::Size::square(crate::BLOCK_SIZE));
    }
}
