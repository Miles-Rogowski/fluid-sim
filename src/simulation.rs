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

const TARGET_DENSITY: f32 = 2.0;
const GRAVITY_STRENGTH: f32 = 0.098;
const DENSITY_WEIGHT: f32 = 18400.0;
const PRESSURE_MULTIPLIER: f32 = 5.0;

fn calculate_velocities(
    mut particles: Query<(&mut Transform, &mut Velocity, &FluidParticle)>,
    //window: Query<&mut Window, With<PrimaryWindow>>,
){
    //position, smoothing radius
    let mut vec: Vec<(Vec2, f32)> = vec![];
    for (transform, _, fluid_particle) in particles.iter_mut(){
        

        vec.push((Vec2{ x: transform.translation.x, y: transform.translation.y }, fluid_particle.smoothing_radius));
    }

    particles.par_iter_mut().for_each(|(transform, mut velocity, _)|{
        velocity.y -= GRAVITY_STRENGTH;

        let density = get_smoothing_factor(vec.clone(), Vec2{ x: transform.translation.x, y: transform.translation.y });
        let density_gradient = calculate_density_gradient(vec.clone(), Vec2{ x: transform.translation.x, y: transform.translation.y }, density);

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
    let volume = 3.141 * radius.powf(8.0) / 4.0;
    let mut value = radius * radius - distance * distance;

    if value < 0.0{
        value = 0.0;
    }

    return value * value * value / volume;//* DENSITY_WEIGHT;
}

fn smoothing_function_derivative(dst: f32, radius: f32) -> f32{
    if dst >= radius{
        return 0.0;
    }
    let f = radius * radius - dst * dst;
    let scale = -24.0 / (3.141 * radius.powf(8.0));
    return scale * dst * f * f;
}

fn calculate_density_gradient(particles: Vec<(Vec2, f32)>, sample_location: Vec2, density: f32) -> Vec2{
    let mut gradient = Vec2::ZERO;

    for (position, smoothing_radius) in &particles{
        let dst = (position - sample_location).length();
        if dst < 0.0001{
            continue;
        }
        let dir = Vec2{ x: position.x - sample_location.x, y: position.y - sample_location.y } / dst;
        let slope = smoothing_function_derivative(dst, *smoothing_radius);
        //let density = get_smoothing_factor(particles.clone(), *position);
        gradient += dir * slope * PARTICLE_MASS / density;
        //println!("density: {}, slope: {}, gradient: {}, dst: {}, smoothing_rad: {}", density, slope, gradient, dst, smoothing_radius);
    }
    return gradient;
}

fn convert_density_to_pressure(density: f32) -> f32{
    let density_dif = density - TARGET_DENSITY;
    let pressure = density_dif * PRESSURE_MULTIPLIER;
    return pressure;
}