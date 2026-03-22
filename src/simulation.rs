use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{FluidParticle, Velocity, PARTICLE_SIZE};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin{
    fn build(&self, app: &mut App){
        app
        .add_systems(Update, calculate_velocities)
        .add_systems(Update, move_particles.after(calculate_velocities));
    }
}

const TARGET_DENSITY: f32 = 1.0;
const GRAVITY_STRENGTH: f32 = 0.098;

fn calculate_velocities(
    mut particles: Query<(&Transform, &mut Velocity), With<FluidParticle>>,
    //window: Query<&mut Window, With<PrimaryWindow>>,
){
    for (transform, mut velocity) in particles.iter_mut(){
        velocity.y -= GRAVITY_STRENGTH;
    }
}

fn move_particles(
    mut particles: Query<(&mut Transform, &mut Velocity), With<FluidParticle>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
){
    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    let h_cutoff = (height / 2.0) + (PARTICLE_SIZE / 2.0);
    let w_cutoff = (width / 2.0) + (PARTICLE_SIZE / 2.0);

    for (mut transform, mut velocity) in particles.iter_mut(){
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
        
        if transform.translation.y <= -h_cutoff && velocity.y < 0.0{
            velocity.y = -velocity.y;
            transform.translation.y = -h_cutoff;
        }
        else if transform.translation.y >= h_cutoff && velocity.y > 0.0{
            velocity.y = -velocity.y;
            transform.translation.y = h_cutoff;
        }

        if transform.translation.x <= -w_cutoff && velocity.x <= 0.0{
            velocity.x = -velocity.x;
            transform.translation.x = -w_cutoff;
        }
        else if transform.translation.x >= w_cutoff && velocity.x >= 0.0{
            velocity.x = -velocity.x;
            transform.translation.x = w_cutoff;
        }
    }

    
}