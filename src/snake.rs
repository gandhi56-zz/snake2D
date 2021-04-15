
use crate::GrowthEvent;
use bevy::app::EventReader;
use bevy::input::keyboard::KeyCode;
use bevy::input::Input;
use bevy::sprite::ColorMaterial;
use bevy::asset::Handle;
use crate::Size;
use bevy::math::Vec2;
use bevy::sprite::Sprite;
use bevy::sprite::entity::SpriteBundle;
use bevy::ecs::system::Res;
use crate::Materials;
use bevy::ecs::system::Commands;
use crate::ARENA_HEIGHT;
use crate::ARENA_WIDTH;
use bevy::app::EventWriter;
use crate::GameOverEvent;
use crate::LastTailPosition;
use bevy::ecs::system::Query;
use crate::Position;
use bevy::ecs::system::ResMut;
use bevy::ecs::entity::Entity;
use bevy::prelude::SystemLabel;

#[derive(PartialEq, Copy, Clone)]
pub enum Direction{
    Left, 
    Right, 
    Up, 
    Down,
}

impl Direction{
    pub fn opposite(&self) -> Self{
        match self{
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
}

pub struct SnakeHead{
    pub direction: Direction,
}
pub struct SnakeSegment;

#[derive(Default)]
pub struct SnakeSegments(pub Vec<Entity>);

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SnakeMovement{
    Input,
    Movement,
    Eating,
    Growth,
}


pub fn spawn_snake(
    mut commands: Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    segments.0 = vec![
        commands
            // spawn an entity with components: SpriteBundle, SnakeHead, SnakeSegment, Position and Size
            .spawn_bundle(SpriteBundle {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),

        spawn_segment(
            commands,
            &materials.segment_material,
            Position { x: 3, y: 2 },
        ),
    ];
}


pub fn spawn_segment(
    mut commands: Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

pub fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}


pub fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };

        // check for hitting the walls
        if head_pos.x < 0 || head_pos.y < 0 || 
            head_pos.x as u32 >= ARENA_WIDTH ||
            head_pos.y as u32 >= ARENA_HEIGHT{
            game_over_writer.send(GameOverEvent);
        }

        // check if the snake is eating itself
        if segment_positions.contains(&head_pos){
            game_over_writer.send(GameOverEvent)
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

pub fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    materials: Res<Materials>,
){
    if growth_reader.iter().next().is_some(){
        segments.0.push(spawn_segment(
            commands, 
            &materials.segment_material,
            last_tail_position.0.unwrap()
        ));
    }
}