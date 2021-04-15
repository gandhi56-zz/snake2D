use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use rand::prelude::random;
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy::asset::Handle;
use bevy::render::pass::ClearColor;

const ARENA_WIDTH: u32 = 20;
const ARENA_HEIGHT: u32 = 20;

#[derive(Default, Copy, Clone, Eq, Hash)]
struct Position{
    x: i32,
    y: i32,
}

impl PartialEq for Position{
    fn eq(&self, pos: &Position) -> bool {
        self.x == pos.x && self.y == pos.y
    }
}

struct Size{
    width: f32,
    height: f32,
}

impl Size{
    pub fn square(x: f32) -> Self{
        Self{
            width: x,
            height: x,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Direction{
    Left, 
    Right, 
    Up, 
    Down,
}

impl Direction{
    fn opposite(&self) -> Self{
        match self{
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }
}

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
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner.system()),
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


struct SnakeHead{
    direction: Direction,
}
struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SnakeMovement{
    Input,
    Movement,
    Eating,
    Growth,
}

struct Materials{
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

fn spawn_snake(
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

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
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


fn snake_movement(
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

fn spawn_segment(
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




//=== Food ----------------------------------------===//
struct Food{
    timestamp: u128,
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>,
){
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    // get random position
    let mut pos = Position{
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
        .insert(Food{ timestamp })
        .insert(pos)
        .insert(Size::square(0.8));
    
    println!("Spawning food with timestamp: {}", timestamp);
}

//=== eat food and grow ----------------------------===//
struct GrowthEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

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

fn snake_growth(
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

//=== Game over event --------------------------------===//

struct GameOverEvent;

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
