use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::sync::LazyLock;
use std::collections::HashMap;
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

#[derive(PartialEq, Eq, Hash)]
enum Edge{
    Top,
    Left,
    Bottom,
    Right
}


const GRID_SIZE: f32 = 40.0; //40.0
const THREASHOLD: f32 = 1.0;

static LOOKUP_TABLE: LazyLock<[Vec<Edge>; 16]> = LazyLock::new(|| [
    vec![],
    vec![Edge::Left, Edge::Bottom],
    vec![Edge::Bottom, Edge::Right],
    vec![Edge::Left, Edge::Right],
    vec![Edge::Right, Edge::Top],
    vec![Edge::Left, Edge::Top, Edge::Right, Edge::Bottom],
    vec![Edge::Top, Edge::Bottom],
    vec![Edge::Left, Edge::Top],
    vec![Edge::Top, Edge::Left],
    vec![Edge::Top, Edge::Bottom],
    vec![Edge::Top, Edge::Right, Edge::Left, Edge::Bottom],
    vec![Edge::Right, Edge::Top],
    vec![Edge::Left, Edge::Right],
    vec![Edge::Bottom, Edge::Right],
    vec![Edge::Left, Edge::Bottom],
    vec![],
]);

const EDGE_POINTS: LazyLock<HashMap<Edge, (Vec2, Vec2)>> = LazyLock::new(|| HashMap::from([
    (Edge::Top, (Vec2{ x: 0.0, y: 0.0 }, Vec2{ x: 1.0, y: 0.0 })),
    (Edge::Right, (Vec2{ x: 1.0, y: 0.0}, Vec2{ x: 1.0, y: 1.0 })),
    (Edge::Bottom, (Vec2{ x: 0.0, y: 1.0}, Vec2{ x: 1.0, y: 1.0 })),
    (Edge::Left, (Vec2{ x: 0.0, y: 0.0 }, Vec2{ x: 0.0, y: 1.0 }))
]));

#[derive(Component)]
struct MarchingSquareMesh;

#[derive(Component)]
struct GridVisualizer;


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

    let mut grid: Vec<u32> = vec![0; ((width / GRID_SIZE).ceil() as usize + 1) * ((height / GRID_SIZE).ceil() as usize + 1)];

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
            grid[index] += 1;
        }
        if index1 < grid.len(){
            grid[index1] += 1;
        }
        if index2 < grid.len(){
            grid[index2] += 1;
        }
        if index3 < grid.len(){
            grid[index3] += 1;
        }
    }

    for y in 0..((height / GRID_SIZE).ceil() as usize){
        for x in 0..((width / GRID_SIZE).ceil() as usize){
            let index = y * ((width / GRID_SIZE).ceil() as usize + 1) + x;

            let top_left = grid[index] >= THREASHOLD as u32;
            let top_right = grid[index + 1] >= THREASHOLD as u32;
            let bottom_left = grid[index + ((width / GRID_SIZE).ceil() as usize + 1)] >= THREASHOLD as u32;
            let bottom_right = grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1] >= THREASHOLD as u32;

            let lookup_index = (top_left as usize) * 8 + (top_right as usize) * 4 + (bottom_right as usize) * 2 + (bottom_left as usize);


            let edges = &LOOKUP_TABLE[lookup_index];

            for segment in edges.chunks(2){
                let edge1 = &segment[0];
                let edge2 = &segment[1];

                let (val1_a, val1_b) = match edge1{
                    Edge::Top => {
                        (grid[index], grid[index + 1])
                    },
                    Edge::Left => {
                        (grid[index], grid[index + ((width / GRID_SIZE).ceil() as usize + 1)])
                    },
                    Edge::Bottom => {
                        (grid[index + ((width / GRID_SIZE).ceil() as usize + 1)], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                        
                    },
                    Edge::Right => {
                        (grid[index + 1], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                    }
                };

                let (val2_a, val2_b) = match edge2{
                    Edge::Top => {
                        (grid[index], grid[index + 1])
                    },
                    Edge::Left => {
                        (grid[index], grid[index + ((width / GRID_SIZE).ceil() as usize + 1)])
                    },
                    Edge::Bottom => {
                        (grid[index + ((width / GRID_SIZE).ceil() as usize + 1)], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                        
                    },
                    Edge::Right => {
                        (grid[index + 1], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                    }
                };

                let v1 = edge_to_point(edge1, val1_a, val1_b);
                let v2 = edge_to_point(edge2, val2_a, val2_b);


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

fn edge_to_point(edge: &Edge, value_a: u32, value_b: u32) -> Vec2{
    let mut t = 0.5;
    if value_a != value_b{
        t = (THREASHOLD - value_a as f32) / (value_b as f32 - value_a as f32);
    }
    let x = EDGE_POINTS[edge].0.x + t * (EDGE_POINTS[edge].1.x - EDGE_POINTS[edge].0.x);
    let y = EDGE_POINTS[edge].0.y + t * (EDGE_POINTS[edge].1.y - EDGE_POINTS[edge].0.y);

    return Vec2{ x: x, y: y };
}