use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{FluidParticle, Velocity};

pub struct MouseControlPlugin;

impl Plugin for MouseControlPlugin{
    fn build(&self, app: &mut App){
        app
        .add_systems(Update, mouse_displacement);
    }
}

const MOUSE_EFFECT_RADIUS: f32 = 200.0;
const MOUSE_EFFECT_STRENGTH: f32 = 20.0;

fn mouse_displacement(
    mut particles: Query<(&Transform, &mut Velocity), With<FluidParticle>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
){
    let window = window.single().unwrap();
    let Ok((camera, camera_transform)) = camera.single() else { panic!("no camera!") };

    if let Some(mouse_position) = window.cursor_position(){
        let world_mouse_position = camera.viewport_to_world_2d(camera_transform, mouse_position).unwrap();

        //apply force

        particles.par_iter_mut().for_each(|(transform, mut velocity)|{
            let pos_dif =  Vec2{ x: world_mouse_position.x, y: world_mouse_position.y } - Vec2{ x: transform.translation.x, y: transform.translation.y};
            let mut interaction_force = Vec2::ZERO;

            let sqrt_dst = Vec2::dot(pos_dif, pos_dif);

            let mut mouse_input_strength = 0.0;

            if mouse_input.pressed(MouseButton::Left){
                mouse_input_strength -= 0.05;
            }
            if mouse_input.pressed(MouseButton::Right){
                mouse_input_strength += 0.05;
            }


            if sqrt_dst < MOUSE_EFFECT_RADIUS * MOUSE_EFFECT_RADIUS{
                let dst = sqrt_dst.sqrt();
                let mut dir_to_inp_point = Vec2::ZERO;
                if dst > 0.001{
                    dir_to_inp_point = pos_dif / dst;
                }

                let center_t = 1.0 - (dst / MOUSE_EFFECT_RADIUS);

                if mouse_input_strength > 0.0{
                    interaction_force += ((dir_to_inp_point * MOUSE_EFFECT_STRENGTH - Vec2{ x: velocity.x, y: velocity.y }) * center_t) * mouse_input_strength;
                }
                else{
                    interaction_force += ((dir_to_inp_point * MOUSE_EFFECT_STRENGTH + Vec2{ x: velocity.x, y: velocity.y }) * center_t) * mouse_input_strength;
                }
                
            }

            velocity.x += interaction_force.x;
            velocity.y += interaction_force.y;
        })
    }
}