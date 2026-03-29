use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::sync::LazyLock;
use bevy::mesh::PrimitiveTopology;
use bevy::asset::RenderAssetUsages;
use crate::FluidParticle;

pub struct MarchingSquaresPlugin;

impl Plugin for MarchingSquaresPlugin{
    fn build(&self, app: &mut App){
        app
        .add_systems(Update, marching_squares);
    }
}


const GRID_SIZE: f32 = 40.0;

static LOOKUP_TABLE: LazyLock<[Vec<Vec2>; 16]> = LazyLock::new(|| [
    vec![Vec2::ZERO, Vec2::ZERO],                                                                                // 0:  0000
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:0.5,y:1.0}],                                         // 1:  0001 BL
    vec![Vec2{x:0.5,y:1.0}, Vec2{x:1.0,y:0.5}],                                         // 2:  0010 BR
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:1.0,y:0.5}],                                         // 3:  0011 BR+BL
    vec![Vec2{x:1.0,y:0.5}, Vec2{x:0.5,y:0.0}],                                        // 4:  0100 TR
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:0.5,y:0.0}, Vec2{x:1.0,y:0.5}, Vec2{x:0.5,y:1.0}], // 5:  0101 TR+BL
    vec![Vec2{x:0.5,y:0.0}, Vec2{x:0.5,y:1.0}],                                         // 6:  0110 TR+BR
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:0.5,y:0.0}],                                         // 7:  0111 TR+BR+BL
    vec![Vec2{x:0.5,y:0.0}, Vec2{x:0.0,y:0.5}],                                         // 8:  1000 TL
    vec![Vec2{x:0.5,y:0.0}, Vec2{x:0.5,y:1.0}],                                         // 9:  1001 TL+BL
    vec![Vec2{x:0.5,y:0.0}, Vec2{x:1.0,y:0.5}, Vec2{x:0.0,y:0.5}, Vec2{x:0.5,y:1.0}], // 10: 1010 TL+BR
    vec![Vec2{x:1.0,y:0.5}, Vec2{x:0.5,y:0.0}],                                         // 11: 1011 TL+BR+BL
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:1.0,y:0.5}],                                         // 12: 1100 TL+TR
    vec![Vec2{x:0.5,y:1.0}, Vec2{x:1.0,y:0.5}],                                         // 13: 1101 TL+TR+BL
    vec![Vec2{x:0.0,y:0.5}, Vec2{x:0.5,y:1.0}],                                         // 14: 1110 TL+TR+BR
    vec![Vec2::ZERO, Vec2::ZERO],                                                                                // 15: 1111
]);

#[derive(Component)]
struct MarchingSquareMesh;


fn marching_squares(
    particles: Query<&Transform, With<FluidParticle>>,
    window: Query<&mut Window, With<PrimaryWindow>>,
    past_meshes: Query<Entity, With<MarchingSquareMesh>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){

    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();

    let mut grid: Vec<bool> = vec![false; ((width / GRID_SIZE).ceil() as usize + 1) * ((height / GRID_SIZE).ceil() as usize + 1)];

    let mut vertices: Vec<Vec2> = vec![];

    for transform in particles.iter(){
        let x = transform.translation.x;
        let y = transform.translation.y;

        let grid_x = ((x + width / 2.0) / GRID_SIZE).floor() as usize;
        let grid_y = ((-y + height / 2.0) / GRID_SIZE).floor() as usize;

        let index = grid_y * ((width / GRID_SIZE).ceil() as usize + 1) + grid_x;
        let index1 = (grid_y + 1) * ((width / GRID_SIZE).ceil() as usize + 1) + grid_x;
        let index2 = grid_y * ((width / GRID_SIZE).ceil() as usize + 1) + grid_x + 1;
        let index3 = (grid_y + 1) * ((width / GRID_SIZE).ceil() as usize + 1) + grid_x + 1;

        if index < grid.len(){
            grid[index] = true;
        }
        if index1 < grid.len(){
            grid[index1] = true;
        }
        if index2 < grid.len(){
            grid[index2] = true;
        }
        if index3 < grid.len(){
            grid[index3] = true;
        }
    }

    for y in 0..((height / GRID_SIZE).ceil() as usize){
        for x in 0..((width / GRID_SIZE).ceil() as usize){
            let index = y * ((width / GRID_SIZE).ceil() as usize + 1) + x;

            let top_left = grid[index];
            let top_right = grid[index + 1];
            let bottom_left = grid[index + ((width / GRID_SIZE).ceil() as usize + 1)];
            let bottom_right = grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1];

            let lookup_index = (top_left as usize) * 8 + (top_right as usize) * 4 + (bottom_right as usize) * 2 + (bottom_left as usize);


            let edges = &LOOKUP_TABLE[lookup_index];

            for segment in edges.chunks(2){
                let v1 = segment[0];
                let v2 = segment[1];

                vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v1.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v1.y * GRID_SIZE - height / 2.0) });
                vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v2.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v2.y * GRID_SIZE - height / 2.0) });
            }
                
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    let positions: Vec<[f32; 3]> = vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(LinearRgba::rgb(1.0, 0.0, 0.0))))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MarchingSquareMesh,
    ));

    for entity in past_meshes.iter(){
        commands.entity(entity).despawn();
    }


}