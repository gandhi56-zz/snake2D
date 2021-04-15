//=== Food ----------------------------------------===//
use bevy::ecs::query::With;
use bevy::ecs::entity::Entity;
use crate::Size;
use bevy::sprite::entity::SpriteBundle;
use crate::ARENA_HEIGHT;
use crate::ARENA_WIDTH;
use crate::random;
use crate::Position;
use bevy::ecs::system::Query;
use crate::SnakeSegments;
use bevy::ecs::system::ResMut;
use crate::Materials;
use bevy::ecs::system::Res;
use bevy::ecs::system::Commands;
use std::time::UNIX_EPOCH;
use std::time::SystemTime;

pub struct Food;

#[derive(Debug)]
pub struct Timestamp(u128);

fn get_timestamp() -> u128{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

pub fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>,
){


    // get random position
    let pos = Position{
        x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
        y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
    };

    // check if pos coincides with a snake segment
    // if so, return without spawning
    let segment_positions = segments.0
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>();

    for segpos in segment_positions.iter(){
        if segpos == &pos{
            return;
        }
    }

    commands
        .spawn_bundle(SpriteBundle{
            material: materials.food_material.clone(),
            ..Default::default()
        })
        .insert(Food)
        .insert(Timestamp(get_timestamp()))
        .insert(pos)
        .insert(Size::square(0.8));
}

pub fn food_despawner(
    mut commands: Commands,
    food_items: Query<(Entity, &Timestamp), With<Food>>,
){
    for (ent, ts) in food_items.iter(){
        if get_timestamp() - ts.0 >= 5{
            commands.entity(ent).despawn();
        }
    }
}