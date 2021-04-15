#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use bevy::app::EventReader;
use bevy::app::EventWriter;
use bevy::ecs::query::With;
use bevy::ecs::entity::Entity;
use bevy::sprite::entity::SpriteBundle;
use bevy::math::Vec3;
use bevy::transform::components::Transform;
use bevy::window::Windows;
use bevy::ecs::system::Res;
use bevy::math::Vec2;
use bevy::ecs::system::Query;
use bevy::sprite::Sprite;
use bevy::render::entity::OrthographicCameraBundle;
use bevy::asset::Assets;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Commands;
use bevy::ecs::schedule::SystemSet;
use bevy::ecs::schedule::SystemStage;
use bevy::DefaultPlugins;
use bevy::render::color::Color;
use bevy::window::WindowDescriptor;
use bevy::app::App;
use crate::snake::{Position, snake_growth, snake_movement, snake_movement_input, spawn_snake, SnakeHead, SnakeMovement, SnakeSegment, SnakeSegments, Size};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::random;
use bevy::core::FixedTimestep;
use bevy::sprite::ColorMaterial;
use bevy::asset::Handle;
use bevy::render::pass::ClearColor;
use bevy::prelude::*;

mod snake;

const ARENA_WIDTH: u32 = 20;
const ARENA_HEIGHT: u32 = 20;



fn main() {
    App::build()
        .insert_resource(WindowDescriptor{
            title: "Snake 2D".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage("game setup", SystemStage::single(spawn_snake.system()))
        .add_system(snake_movement_input
            .system()
            .label(SnakeMovement::Input)
            .before(SnakeMovement::Movement),)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement.system().label(SnakeMovement::Movement))
                .with_system(
                    snake_eating
                        .system()
                        .label(SnakeMovement::Eating)
                        .after(SnakeMovement::Movement),
                )
                .with_system(
                    snake_growth
                    .system()
                    .label(SnakeMovement::Growth)
                    .after(SnakeMovement::Eating),
                )
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(size_scaling.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(4.0))
                .with_system(food_despawner.system())
                .with_system(food_spawner.system())
        )
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_system(game_over
            .system()
            .after(SnakeMovement::Movement)
        )
        .run();
}

fn setup(
    mut commands: Commands, 
    mut materials: ResMut<Assets<ColorMaterial>>){
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // set materials for each component
    commands.insert_resource(
        Materials{
            head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
            segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        }
    );
}

pub struct Materials{
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}


fn size_scaling(
    windows: Res<Windows>,
    mut q: Query<(&Size, &mut Sprite)>){
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut(){
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(
    windows: Res<Windows>,
    mut q: Query<(&Position, &mut Transform)>){
    
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32{
        let tile_size = bound_window / bound_game;
        pos/bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut(){
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }

}




//=== Food ----------------------------------------===//
struct Food;

#[derive(Debug)]
struct Timestamp(u128);

fn get_timestamp() -> u128{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn food_spawner(
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

fn food_despawner(
    mut commands: Commands,
    food_items: Query<(Entity, &Timestamp), With<Food>>,
){
    for (ent, ts) in food_items.iter(){
        if get_timestamp() - ts.0 >= 5{
            commands.entity(ent).despawn();
        }
    }
}

//=== eat food and grow ----------------------------===//
pub struct GrowthEvent;

#[derive(Default)]
pub struct LastTailPosition(Option<Position>);

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
){
    for head_pos in head_positions.iter(){
        for (ent, food_pos) in food_positions.iter(){
            if food_pos == head_pos{
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

//=== Game over event --------------------------------===//

pub struct GameOverEvent;

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    materials: Res<Materials>,
    segment_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>
){
    if reader.iter().next().is_some(){
        for ent in food.iter().chain(segments.iter()){
            commands.entity(ent).despawn();
        }
        spawn_snake(commands, materials, segment_res);
    }
}
