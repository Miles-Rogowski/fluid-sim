use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{FluidParticle, Velocity};

pub struct MouseControlPlugin;

impl Plugin for MouseControlPlugin{
    fn build(&self, app: &mut App){
        app
        .insert_resource(AnchorPoint(Vec2{ x: 0.0, y: 0.0 }))
        .add_systems(Update, mouse_displacement)
        .add_systems(Update, create_obstacles);
    }
}

const MOUSE_EFFECT_RADIUS: f32 = 200.0;
const MOUSE_EFFECT_STRENGTH: f32 = 20.0;

const OBSTACLE_COLOR: LinearRgba = LinearRgba::rgb(0.3, 0.3, 0.3);

#[derive(Component)]
struct Active;

#[derive(Component)]
pub struct Obstacle{
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

#[derive(Resource)]
struct AnchorPoint(Vec2);


fn mouse_displacement(
    mut particles: Query<(&Transform, &mut Velocity), With<FluidParticle>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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

            if mouse_input.pressed(MouseButton::Left) && !keyboard_input.pressed(KeyCode::ControlLeft){
                mouse_input_strength -= 0.05;
            }
            if mouse_input.pressed(MouseButton::Right) && !keyboard_input.pressed(KeyCode::ControlLeft){
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


fn create_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut anchor_point: ResMut<AnchorPoint>,
    active_obstacles: Query<(Entity, &Obstacle, &Active)>,
    inactive_obstacles: Query<(Entity, &Obstacle), Without<Active>>,
){
    let window = window.single().unwrap();
    let Ok((camera, camera_transform)) = camera.single() else { panic!("no camera!") };

    if let Some(mouse_position) = window.cursor_position(){
        let world_mouse_position = camera.viewport_to_world_2d(camera_transform, mouse_position).unwrap();


        if keyboard_input.pressed(KeyCode::ControlLeft){
            if mouse_input.just_pressed(MouseButton::Left){
                anchor_point.0 = world_mouse_position;
            }
            else if mouse_input.pressed(MouseButton::Left) && anchor_point.0 - world_mouse_position != Vec2::ZERO{
                

                    let width = anchor_point.0.x - world_mouse_position.x;
                    let height = anchor_point.0.y - world_mouse_position.y;

                    let top_left = Vec2{ x: f32::min(anchor_point.0.x , anchor_point.0.x - width), y: f32::min(anchor_point.0.y, anchor_point.0.y - height)};
                    let bottom_right = Vec2{ x: f32::max(anchor_point.0.x, anchor_point.0.x - width), y: f32::max(anchor_point.0.y, anchor_point.0.y - height)};

                    if let Some(active_obstacle) = active_obstacles.iter().next(){
                        commands.entity(active_obstacle.0).despawn();
                    }

                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(width, height))),
                        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(OBSTACLE_COLOR)))),
                        Transform::from_xyz(anchor_point.0.x - width / 2.0, anchor_point.0.y - height / 2.0, 10.0),
                        Active,
                        Obstacle{ top_left: top_left, bottom_right: bottom_right},
                    ));
            }

            if mouse_input.just_pressed(MouseButton::Right){
                for (entity, obstacle) in inactive_obstacles.iter(){
                    if world_mouse_position.x > obstacle.top_left.x && world_mouse_position.y > obstacle.top_left.y && world_mouse_position.x < obstacle.bottom_right.x && world_mouse_position.y < obstacle.bottom_right.y{
                        commands.entity(entity).despawn();
                    }
                }
            }
        }

        if mouse_input.just_released(MouseButton::Left){
            if let Some(active_obstacle) = active_obstacles.iter().next(){
                commands.entity(active_obstacle.0).remove::<Active>();
            }
        }
    }
}