#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::food::{
    Food, food_despawner, food_spawner};
use crate::snake::{
    Position, snake_growth, snake_movement, snake_movement_input, spawn_snake, SnakeMovement, SnakeSegment, SnakeSegments, Size, LastTailPosition, snake_eating, GrowthEvent};

use bevy::{
    core::FixedTimestep,
    DefaultPlugins,
    ecs::{
        system::{Query, ResMut, Commands, Res},
        schedule::{SystemSet, SystemStage},
        query::{With},
        entity::{Entity},
    },
    app::{
        App,
        EventReader},
    math::{Vec2, Vec3},
    transform::components::Transform,
    window::{
        Windows, 
        WindowDescriptor},
    sprite::{Sprite, ColorMaterial},
    render::{
        entity::OrthographicCameraBundle,
        pass::ClearColor,
        color::Color},
    asset::{Assets, Handle},
    prelude::*,
    };

mod snake;
mod food;

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
