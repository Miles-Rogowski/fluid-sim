use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{FluidParticle, Velocity, PARTICLE_SIZE, PARTICLE_MASS, NUMBER_OF_PARTICLES};
use crate::mouse_control::Obstacle;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin{
    fn build(&self, app: &mut App){
        app
        .insert_resource(Densities{ density_array: [0.0; NUMBER_OF_PARTICLES as usize], particle_array: vec![] })
        .add_systems(Update, precalculate_densities)
        .add_systems(Update, calculate_velocities.after(precalculate_densities))
        .add_systems(Update, move_particles.after(calculate_velocities));
    }
}

const TARGET_DENSITY: f32 = 2.75; //2.75
const GRAVITY_STRENGTH: f32 = 0.098; // 0.098
const PRESSURE_MULTIPLIER: f32 = 1.95; //1.95
const SMOOTHING_RADIUS: f32 = 50.0; //50

#[derive(Resource)]
struct Densities{
    density_array: [f32; NUMBER_OF_PARTICLES as usize],
    particle_array: Vec<(Vec2, usize)>
}


fn precalculate_densities(
    particles: Query<(&mut Transform, &mut Velocity), With<FluidParticle>>,
    mut densities: ResMut<Densities>,
){

    densities.particle_array = vec![];

    for (i, (transform, _))  in particles.iter().enumerate(){
        densities.particle_array.push((Vec2{ x: transform.translation.x, y: transform.translation.y }, i));
    }

    for (i, (transform, _)) in particles.iter().enumerate(){
        densities.density_array[i] = get_smoothing_factor(densities.particle_array.clone(), Vec2{ x: transform.translation.x, y: transform.translation.y});
    }
}


fn calculate_velocities(
    mut particles: Query<(&mut Transform, &mut Velocity), With<FluidParticle>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    densities: ResMut<Densities>,
){

    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    for (i, (transform, mut velocity)) in particles.iter_mut().enumerate(){
        velocity.y -= GRAVITY_STRENGTH;

        let vec2_pos= Vec2{ x: transform.translation.x, y: transform.translation.y };

        let density = densities.density_array[i];

        let relevant_particles = get_releavant_particles(vec2_pos, densities.particle_array.clone());

        let density_gradient = calculate_density_gradient(relevant_particles.clone(), vec2_pos, density, height / 2.0, width / 2.0, densities.density_array);

        let pressure = convert_density_to_pressure(density);

        if density > 0.0{
            velocity.x += -pressure * density_gradient.x;
            velocity.y += -pressure * density_gradient.y;
        }
        
    };

}

fn move_particles(
    mut particles: Query<(&mut Transform, &mut Velocity), (With<FluidParticle>, Without<Obstacle>)>,
    obstacles: Query<(&Obstacle, &Transform)>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    time: Res<Time>,
){
    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    let h_cutoff = (height / 2.0) + (PARTICLE_SIZE / 2.0);
    let w_cutoff = (width / 2.0) + (PARTICLE_SIZE / 2.0);

    let dt = time.delta_secs();

    particles.par_iter_mut().for_each(|(mut transform, mut velocity)|{
        transform.translation.x += velocity.x * dt * 60.0;
        transform.translation.y += velocity.y * dt * 60.0;
        
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


        for (obstacle, obstacle_transform) in obstacles.iter(){
            if transform.translation.x > obstacle.top_left.x && transform.translation.x < obstacle.bottom_right.x && transform.translation.y > obstacle.top_left.y && transform.translation.y < obstacle.bottom_right.y{
                let mut overlap = Vec2::ZERO;

                overlap.x = f32::min(transform.translation.x - obstacle.top_left.x, obstacle.bottom_right.x - transform.translation.x);
                overlap.y = f32::min(transform.translation.y - obstacle.top_left.y, obstacle.bottom_right.y - transform.translation.y);

                if overlap.x < overlap.y{
                    velocity.x = -velocity.x;

                    if overlap.x == transform.translation.x - obstacle.top_left.x{
                        transform.translation.x = obstacle.top_left.x;
                    }
                    else{
                        transform.translation.x = obstacle.bottom_right.x;
                    }
                }
                else{
                    velocity.y = -velocity.y;

                    if overlap.y == transform.translation.y - obstacle.top_left.y{
                        transform.translation.y = obstacle.top_left.y;
                    }
                    else{
                        transform.translation.y = obstacle.bottom_right.y;
                    }
                }

            }
        }

    });

}

fn get_smoothing_factor(particles: Vec<(Vec2, usize)>, sample_location: Vec2) -> f32{
    let mut total_value: f32 = 0.0;
    for (transform, _) in particles{
        //find distance
        let dist_x = (sample_location.x - transform.x).abs();
        let dist_y = (sample_location.y - transform.y).abs();

        let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();

        let sample_value: f32 = smoothing_function(dist, SMOOTHING_RADIUS);

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

fn calculate_density_gradient(particles: Vec<(Vec2, usize)>, sample_location: Vec2, density: f32, wall_height: f32, wall_width: f32, densities: [f32; NUMBER_OF_PARTICLES as usize]) -> Vec2{
    let mut gradient = Vec2::ZERO;
    let wall_effect_offset = 0.1;

    for (position, i) in particles.iter(){
        let dst = (position - sample_location).length();
        if dst < 0.0001{
            continue;
        }
        let dir = Vec2{ x: position.x - sample_location.x, y: position.y - sample_location.y } / dst;
        let slope = smoothing_function_derivative(dst, SMOOTHING_RADIUS);
        let other_density = densities[*i];
        let shared_pressure = calculate_shared_pressure(other_density, density);
        gradient += -shared_pressure * dir * slope * PARTICLE_MASS / density;
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

fn calculate_shared_pressure(density_a: f32, density_b: f32) -> f32{
    let pressure_a = convert_density_to_pressure(density_a);
    let pressure_b = convert_density_to_pressure(density_b);
    return (pressure_a + pressure_b) / 2.0;
}

fn get_releavant_particles(sample_location: Vec2, particles: Vec<(Vec2, usize)>) -> Vec<(Vec2, usize)>{
    let mut relevant_particles = vec![];
    for position in particles.iter(){
        if (sample_location - position.0).length().abs() <= SMOOTHING_RADIUS{
            relevant_particles.push(*position);
        }
    }

    return relevant_particles;

}