use bevy::prelude::*;
use bevy::ecs::schedule::ShouldRun;

use modulo::Mod;

use crate::grid::{GridPlugin, Position, Size};

mod grid;

pub const GRID_HEIGHT: usize = 50;
pub const GRID_WIDTH: usize = 50;

const NEIGHBORS: [[i32;2];8] = [
    [-1,-1],[0,-1],[1,-1],
    [-1,0],[1,0],
    [-1,1],[0,1],[1,1],
];

fn main() {
    App::new()
        .init_resource::<StateGrid>()
        .insert_resource(Speed(0.1))
        .insert_resource(EntityGrid(vec![]))
        .insert_resource(Paused(true))
        .insert_resource(LastMouseCell(-1,-1))
        .insert_resource(WindowDescriptor {
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_grid)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(should_update_run)
                .with_system(update_cells),
        )
        .add_system(spawn_cells_with_mouse)
        .add_system(handle_keyboard_input)
        .add_system_to_stage(CoreStage::PostUpdate, update_cell_sprites)
        .add_plugin(GridPlugin)
        .add_plugins(DefaultPlugins)
        .run()
}


struct StateGrid([[bool; GRID_HEIGHT]; GRID_WIDTH]);

impl Default for StateGrid {
    fn default() -> Self{
        Self([[false; GRID_HEIGHT]; GRID_WIDTH])
    }
}

struct Speed(f32);

struct EntityGrid(Vec<Vec<Entity>>);

struct Paused(bool);

struct LastMouseCell(i32, i32);

#[derive(Component)]
struct Cell;

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_grid(
    mut commands: Commands,
    mut grid: ResMut<EntityGrid>,
    ) {
    for x in 0..GRID_WIDTH {
        (*grid).0.push(vec![]);
        for y in 0..GRID_HEIGHT {
            let id = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        ..default()
                    },
                    ..default()
                })
            .insert(Cell)
                .insert(Position { x: x as i32, y: y as i32})
                .insert(Size::square(0.8))
                .id();
            (*grid).0[x].push(id);
        }
    }
}

fn update_cells(
    mut state_grid: ResMut<StateGrid>,
    ) {
    let initial_state_grid = (*state_grid).0.clone();
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT { 
            let mut num_alive_nb = 0;
            for [dx,dy] in NEIGHBORS {
                let nx = (dx + x as i32).modulo(GRID_WIDTH as i32) as usize;
                let ny = (dy + y as i32).modulo(GRID_HEIGHT as i32) as usize;
                if initial_state_grid[nx][ny] {
                    num_alive_nb += 1;
                }
            }
            let mut alive = initial_state_grid[x][y];
            if alive && num_alive_nb < 2 {
                alive = false;
            } else if alive && num_alive_nb > 3 {
                alive = false;
            } else if !alive && num_alive_nb == 3 {
                alive = true;
            }
            (*state_grid).0[x][y] = alive;
        }
    }
}

fn update_cell_sprites(
    state_grid: Res<StateGrid>,
    entity_grid: Res<EntityGrid>,
    mut sprites: Query<&mut Sprite, With<Cell>>,
    ) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let mut sprite = sprites.get_mut((*entity_grid).0[x][y]).unwrap();
            sprite.color = if (*state_grid).0[x][y] { Color::WHITE } else { Color::BLACK };
        }
    }
}


fn should_update_run(
    paused: Res<Paused>,
    time: Res<Time>,
    mut next_run_time: Local<u128>,
    speed: Res<Speed>,
    ) -> ShouldRun {
        let milis = time.time_since_startup().as_millis();
        if !paused.0 && milis > *next_run_time {
            *next_run_time = milis + (speed.0 * 1000.0) as u128;
            return ShouldRun::Yes;
        }
        ShouldRun::No
}

fn spawn_cells_with_mouse(
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    mut state_grid: ResMut<StateGrid>,
    mut last_cell: ResMut<LastMouseCell>,
    ) {
    if buttons.pressed(MouseButton::Left) {
        let window = windows.get_primary().unwrap();
        if let Some(position) = window.cursor_position() {
            let x = position.x as f32 / window.width() as f32 * GRID_WIDTH as f32;
            let y = position.y as f32 / window.height() as f32 * GRID_HEIGHT as f32;
            if last_cell.0 != x as i32 || last_cell.1 != y as i32 {
                (*state_grid).0[x as usize][y as usize] = !state_grid.0[x as usize][y as usize];
                (*last_cell).0 = x as i32;
                (*last_cell).1 = y as i32;
            }
        }
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut speed: ResMut<Speed>,
    mut paused: ResMut<Paused>,
    mut commands: Commands,
    ) {
        if keyboard_input.just_released(KeyCode::Space) {
            (*paused).0 = !paused.0;
        }
        if keyboard_input.just_released(KeyCode::Left) {
            (*speed).0 = speed.0 * 1.25;
        }
        if keyboard_input.just_released(KeyCode::Right) {
            (*speed).0 = speed.0 * 0.75;
        }
        if keyboard_input.just_released(KeyCode::C) {
            commands.insert_resource(StateGrid::default());
            (*paused).0 = true;
        }
}
