use bevy::prelude::*;
use crate::simulation::SimulationPlugin;
use crate::mouse_control::MouseControlPlugin;

mod simulation;
mod mouse_control;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(MouseControlPlugin)
    .add_plugins(SimulationPlugin)
    .add_systems(Startup, setup)
    .run();
}

const NUMBER_OF_PARTICLES: u32 = 500;
const ROW_SIZE: u32 = 25;
const PARTICLE_COLOR: LinearRgba = LinearRgba::rgb(0.0, 0.25, 1.0);
pub const PARTICLE_SIZE: f32 = 5.0; 
pub const PARTICLE_MASS: f32 = 5.0;



#[derive(Component)]
pub struct FluidParticle;

#[derive(Component)]
pub struct Velocity{
    pub x: f32,
    pub y: f32
}




fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    commands.spawn(Camera2d);

    for i in 0..NUMBER_OF_PARTICLES{

        let position = find_particle_position_in_grid(i);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(PARTICLE_COLOR)))),
            Transform::from_xyz(position.x, position.y, position.z),
            FluidParticle,
            Velocity{ x: 0.0, y: 0.0 },
        ));
    }
}

fn find_particle_position_in_grid(particle_number: u32) -> Vec3{
    let mut transform = Vec3::ZERO;
    transform.x = ((particle_number % ROW_SIZE) as f32 * PARTICLE_SIZE * 2.0) - (ROW_SIZE as f32 * PARTICLE_SIZE);
    transform.y = ((particle_number / ROW_SIZE) as f32 * PARTICLE_SIZE * 2.0) - (NUMBER_OF_PARTICLES / ROW_SIZE) as f32 * PARTICLE_SIZE;
    transform.z = 1.0;
    return transform;
}