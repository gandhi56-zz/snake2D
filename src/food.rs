//=== Food ----------------------------------------===//
use bevy::{
    ecs::{
        query::With, 
        entity::Entity,
        system::{Query, ResMut, Res, Commands},
    },
    sprite::{
        entity::SpriteBundle
    },
};
use crate::{Size, Position,SnakeSegments,Materials};
use std::time::{UNIX_EPOCH, SystemTime};

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
    let mut pos = Position::default();
    pos.randomize();

    // check if pos coincides with a snake segment
    // if so, return without spawning
    let segment_positions = segments.0
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>();

    for _cnt in 1..5{
        let mut ok = true;
        
        // check if pos coincides with snake segment
        for segpos in segment_positions.iter(){
            if segpos == &pos{
                ok = false;
            }
        }

        match ok{
            true => {break;}
            _ => {pos.randomize();}
        };

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