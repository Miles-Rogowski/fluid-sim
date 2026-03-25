use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{FluidParticle, Velocity, PARTICLE_SIZE, PARTICLE_MASS, NUMBER_OF_PARTICLES};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin{
    fn build(&self, app: &mut App){
        app
        .add_systems(Update, calculate_velocities)
        .add_systems(Update, move_particles.after(calculate_velocities));
    }
}

const TARGET_DENSITY: f32 = 3.0;
const GRAVITY_STRENGTH: f32 = 0.098;
const PRESSURE_MULTIPLIER: f32 = 3.0;

fn calculate_velocities(
    mut particles: Query<(&mut Transform, &mut Velocity, &FluidParticle)>,
    window: Query<&mut Window, With<PrimaryWindow>>,
){

    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    //position, smoothing radius
    let mut vec: Vec<(Vec2, f32)> = vec![];
    for (transform, _, fluid_particle) in particles.iter_mut(){
        

        vec.push((Vec2{ x: transform.translation.x, y: transform.translation.y }, fluid_particle.smoothing_radius));
    }

    particles.par_iter_mut().for_each(|(transform, mut velocity, _)|{
        velocity.y -= GRAVITY_STRENGTH;

        let density = get_smoothing_factor(vec.clone(), Vec2{ x: transform.translation.x, y: transform.translation.y });
        let density_gradient = calculate_density_gradient(vec.clone(), Vec2{ x: transform.translation.x, y: transform.translation.y }, density, window.height() / 2.0, window.width() / 2.0);

        let pressure = convert_density_to_pressure(density);

        if density > 0.0{
            velocity.x += -pressure * density_gradient.x;
            velocity.y += -pressure * density_gradient.y;
        }
        
    });

}

fn move_particles(
    mut particles: Query<(&mut Transform, &mut Velocity, &FluidParticle)>,
    window: Query<&mut Window, With<PrimaryWindow>>,
){
    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    let h_cutoff = (height / 2.0) + (PARTICLE_SIZE / 2.0);
    let w_cutoff = (width / 2.0) + (PARTICLE_SIZE / 2.0);

    for (mut transform, mut velocity, _) in particles.iter_mut(){
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
        
        if transform.translation.y <= -h_cutoff && velocity.y < 0.0{
            velocity.y = -velocity.y * 0.7;
            transform.translation.y = -h_cutoff;
        }
        else if transform.translation.y >= h_cutoff && velocity.y > 0.0{
            velocity.y = -velocity.y * 0.7;
            transform.translation.y = h_cutoff;
        }

        if transform.translation.x <= -w_cutoff && velocity.x <= 0.0{
            velocity.x = -velocity.x * 0.7;
            transform.translation.x = -w_cutoff;
        }
        else if transform.translation.x >= w_cutoff && velocity.x >= 0.0{
            velocity.x = -velocity.x * 0.7;
            transform.translation.x = w_cutoff;
        }
    }

}

fn get_smoothing_factor(particles: Vec<(Vec2, f32)>, sample_location: Vec2) -> f32{
    let mut total_value: f32 = 0.0;
    for (transform, smoothing_radius) in particles{
        //find distance
        let dist_x = (sample_location.x - transform.x).abs();
        let dist_y = (sample_location.y - transform.y).abs();

        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        let sample_value: f32 = smoothing_function(dist, smoothing_radius);

        total_value += sample_value * PARTICLE_MASS;
    }
    return total_value;
}

fn smoothing_function(distance:f32, radius:f32) -> f32{

    if distance >= radius{
        return 0.0;
    }

    let volume = 3.141 * radius.powf(4.0) / 6.0;

    return (radius - distance) * (radius - distance) / volume;
}

fn smoothing_function_derivative(dst: f32, radius: f32) -> f32{
    if dst >= radius{
        return 0.0;
    }
    let scale = 12.0 / (3.141 * radius.powf(4.0));
    return (dst - radius) * scale;
}

fn calculate_density_gradient(particles: Vec<(Vec2, f32)>, sample_location: Vec2, density: f32, wall_height: f32, wall_width: f32) -> Vec2{
    let mut gradient = Vec2::ZERO;
    let wall_effect_offset = 0.1;

    for (position, smoothing_radius) in &particles{
        let dst = (position - sample_location).length();
        if dst < 0.0001{
            continue;
        }
        let dir = Vec2{ x: position.x - sample_location.x, y: position.y - sample_location.y } / dst;
        let slope = smoothing_function_derivative(dst, *smoothing_radius);
        gradient += dir * slope * PARTICLE_MASS / density;
    }

        if sample_location.y <= -wall_height + 5.0{
            gradient.y += wall_effect_offset;
        }
        else if sample_location.y >= wall_height - 5.0{
            gradient.y -= wall_effect_offset;
        }
        if sample_location.x <= -wall_width + 5.0{
            gradient.x += wall_effect_offset;
        }
        else if sample_location.x >= wall_width - 5.0{
            gradient.x -= wall_effect_offset;
        }

    return gradient;
}

fn convert_density_to_pressure(density: f32) -> f32{
    let density_dif = density - TARGET_DENSITY;
    let pressure = density_dif * PRESSURE_MULTIPLIER;
    return pressure;
}