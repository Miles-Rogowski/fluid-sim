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

enum Corner{
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}

enum TriPoint{
    Edge(Edge),
    Corner(Corner)
}



const GRID_SIZE: f32 = 15.0; //15.0
const THREASHOLD: f32 = 1.0;

const DARK_COLOR: LinearRgba = LinearRgba::rgb(0.0, 0.125, 0.5);
const MEDIUM_COLOR: LinearRgba = LinearRgba::rgb(0.0, 0.25, 1.0);
const LIGHT_COLOR: LinearRgba = LinearRgba::rgb(1.0, 1.0, 1.0);

//this lookup table was made by Claude, I cant be bothered to manually write all these out :
static TRIANGLE_LOOKUP_TABLE: LazyLock<[Vec<TriPoint>; 16]> = LazyLock::new(|| [
    // 0: no corners inside
    vec![],
    // 1: bottom-left inside
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::BottomLeft)],
    // 2: bottom-right inside
    vec![TriPoint::Edge(Edge::Bottom), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::BottomRight)],
    // 3: bottom-left + bottom-right inside
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::BottomRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::BottomLeft)],
    // 4: top-right inside
    vec![TriPoint::Edge(Edge::Right), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::TopRight)],
    // 5: bottom-left + top-right inside (ambiguous)
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::BottomLeft), TriPoint::Edge(Edge::Right), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::TopRight)],
    // 6: bottom-right + top-right inside
    vec![TriPoint::Edge(Edge::Bottom), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::TopRight), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::BottomRight)],
    // 7: bottom-left + bottom-right + top-right inside
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::TopRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::BottomRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::BottomLeft)],
    // 8: top-left inside
    vec![TriPoint::Edge(Edge::Top), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::TopLeft)],
    // 9: bottom-left + top-left inside
    vec![TriPoint::Edge(Edge::Top), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::BottomLeft), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::BottomLeft), TriPoint::Corner(Corner::TopLeft)],
    // 10: bottom-right + top-left inside (ambiguous)
    vec![TriPoint::Edge(Edge::Bottom), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::BottomRight), TriPoint::Edge(Edge::Top), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::TopLeft)],
    // 11: bottom-left + bottom-right + top-left inside
    vec![TriPoint::Edge(Edge::Top), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::BottomRight), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::BottomLeft), TriPoint::Edge(Edge::Top), TriPoint::Corner(Corner::BottomLeft), TriPoint::Corner(Corner::TopLeft)],
    // 12: top-left + top-right inside
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::TopRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::TopLeft)],
    // 13: bottom-left + top-left + top-right inside
    vec![TriPoint::Edge(Edge::Bottom), TriPoint::Edge(Edge::Right), TriPoint::Corner(Corner::TopRight), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::TopLeft), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::TopLeft), TriPoint::Corner(Corner::BottomLeft)],
    // 14: bottom-right + top-left + top-right inside
    vec![TriPoint::Edge(Edge::Left), TriPoint::Edge(Edge::Bottom), TriPoint::Corner(Corner::BottomRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::TopRight), TriPoint::Edge(Edge::Left), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::TopLeft)],
    // 15: all corners inside
    vec![TriPoint::Corner(Corner::TopLeft), TriPoint::Corner(Corner::TopRight), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::TopLeft), TriPoint::Corner(Corner::BottomRight), TriPoint::Corner(Corner::BottomLeft)],
]);

const EDGE_POINTS: LazyLock<HashMap<Edge, (Vec2, Vec2)>> = LazyLock::new(|| HashMap::from([
    (Edge::Top, (Vec2{ x: 0.0, y: 0.0 }, Vec2{ x: 1.0, y: 0.0 })),
    (Edge::Right, (Vec2{ x: 1.0, y: 0.0}, Vec2{ x: 1.0, y: 1.0 })),
    (Edge::Bottom, (Vec2{ x: 0.0, y: 1.0}, Vec2{ x: 1.0, y: 1.0 })),
    (Edge::Left, (Vec2{ x: 0.0, y: 0.0 }, Vec2{ x: 0.0, y: 1.0 }))
]));

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

    let mut grid: Vec<u32> = vec![0; ((width / GRID_SIZE).ceil() as usize + 1) * ((height / GRID_SIZE).ceil() as usize + 1)];

    let mut dark_vertices: Vec<Vec2> = vec![];
    let mut medium_vertices: Vec<Vec2> = vec![];
    let mut light_vertices: Vec<Vec2> = vec![];

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
        if index2 < grid.len() && grid_x != ((width / GRID_SIZE).ceil() as usize){
            grid[index2] += 1;
        }
        if index3 < grid.len() && grid_x != ((width / GRID_SIZE).ceil() as usize){
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


            let edges = &TRIANGLE_LOOKUP_TABLE[lookup_index];

            for segment in edges.chunks(3){
                let point1 = &segment[0];
                let point2 = &segment[1];
                let point3 = &segment[2];

                let v1 = match point1{
                    TriPoint::Edge(Edge::Top) => {
                        edge_to_point(&Edge::Top, grid[index], grid[index + 1])
                    },
                    TriPoint::Edge(Edge::Left) => {
                        edge_to_point(&Edge::Left, grid[index], grid[index + ((width / GRID_SIZE).ceil() as usize + 1)])
                    },
                    TriPoint::Edge(Edge::Bottom) => {
                        edge_to_point(&Edge::Bottom, grid[index + ((width / GRID_SIZE).ceil() as usize + 1)], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                        
                    },
                    TriPoint::Edge(Edge::Right) => {
                        edge_to_point(&Edge::Right, grid[index + 1], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                    },
                    TriPoint::Corner(_) => {
                        corner_to_point(point1)
                    }
                };

                let v2 = match point2{
                    TriPoint::Edge(Edge::Top) => {
                        edge_to_point(&Edge::Top, grid[index], grid[index + 1])
                    },
                    TriPoint::Edge(Edge::Left) => {
                        edge_to_point(&Edge::Left, grid[index], grid[index + ((width / GRID_SIZE).ceil() as usize + 1)])
                    },
                    TriPoint::Edge(Edge::Bottom) => {
                        edge_to_point(&Edge::Bottom, grid[index + ((width / GRID_SIZE).ceil() as usize + 1)], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                        
                    },
                    TriPoint::Edge(Edge::Right) => {
                        edge_to_point(&Edge::Right, grid[index + 1], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                    },
                    TriPoint::Corner(_) => {
                        corner_to_point(point2)
                    }
                };

                let v3 = match point3{
                    TriPoint::Edge(Edge::Top) => {
                        edge_to_point(&Edge::Top, grid[index], grid[index + 1])
                    },
                    TriPoint::Edge(Edge::Left) => {
                        edge_to_point(&Edge::Left, grid[index], grid[index + ((width / GRID_SIZE).ceil() as usize + 1)])
                    },
                    TriPoint::Edge(Edge::Bottom) => {
                        edge_to_point(&Edge::Bottom, grid[index + ((width / GRID_SIZE).ceil() as usize + 1)], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                        
                    },
                    TriPoint::Edge(Edge::Right) => {
                        edge_to_point(&Edge::Right, grid[index + 1], grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1])
                    },
                    TriPoint::Corner(_) => {
                        corner_to_point(point3)
                    }
                };

                //let v1 = edge_to_point(point1, val1_a, val1_b);
                //let v2 = edge_to_point(point2, val2_a, val2_b);

                let total_value = grid[index] + grid[index + 1] + grid[index + ((width / GRID_SIZE).ceil() as usize + 1)] + grid[index + ((width / GRID_SIZE).ceil() as usize + 1) + 1];

                if total_value >= (THREASHOLD * 16.0) as u32{
                    dark_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v1.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v1.y * GRID_SIZE - height / 2.0) });
                    dark_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v2.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v2.y * GRID_SIZE - height / 2.0) });
                    dark_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v3.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v3.y * GRID_SIZE - height / 2.0) });
                }
                else if total_value >= (THREASHOLD * 8.0) as u32{
                    medium_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v1.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v1.y * GRID_SIZE - height / 2.0) });
                    medium_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v2.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v2.y * GRID_SIZE - height / 2.0) });
                    medium_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v3.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v3.y * GRID_SIZE - height / 2.0) });
                }
                else{
                    light_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v1.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v1.y * GRID_SIZE - height / 2.0) });
                    light_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v2.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v2.y * GRID_SIZE - height / 2.0) });
                    light_vertices.push(Vec2{ x: x as f32 * GRID_SIZE + v3.x * GRID_SIZE - width / 2.0, y: -(y as f32 * GRID_SIZE + v3.y * GRID_SIZE - height / 2.0) });
                }
            }
                
        }
    }

    let mut dark_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let dark_positions: Vec<[f32; 3]> = dark_vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();
    dark_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, dark_positions);  
    
    let mut medium_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let medium_positions: Vec<[f32; 3]> = medium_vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();
    medium_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, medium_positions);

    let mut light_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let light_positions: Vec<[f32; 3]> = light_vertices.iter().map(|v| [v.x, v.y, 0.0]).collect();
    light_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, light_positions);


    commands.spawn((
        Mesh2d(meshes.add(dark_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(DARK_COLOR)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MarchingSquareMesh,
    ));

    commands.spawn((
        Mesh2d(meshes.add(medium_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(MEDIUM_COLOR)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MarchingSquareMesh,
    ));

    commands.spawn((
        Mesh2d(meshes.add(light_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::from(LIGHT_COLOR)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MarchingSquareMesh,
    ));

    for entity in past_meshes.iter(){
        commands.entity(entity).despawn();
    }


}

fn edge_to_point(edge: &Edge, value_a: u32, value_b: u32) -> Vec2{
    let t = if value_a != value_b{
        (THREASHOLD - value_a as f32) / (value_b as f32 - value_a as f32)
    }
    else{
        0.5
    };

    let x = EDGE_POINTS[edge].0.x + t * (EDGE_POINTS[edge].1.x - EDGE_POINTS[edge].0.x);
    let y = EDGE_POINTS[edge].0.y + t * (EDGE_POINTS[edge].1.y - EDGE_POINTS[edge].0.y);

        return Vec2{ x: x, y: y };
}

fn corner_to_point(corner: &TriPoint) -> Vec2{
    let vector = match corner{
        &TriPoint::Corner(Corner::TopLeft) => {
            Vec2{ x: 0.0, y: 0.0 }
        },
        &TriPoint::Corner(Corner::TopRight) => {
            Vec2{ x: 1.0, y: 0.0 }
        },
        &TriPoint::Corner(Corner::BottomLeft) => {
            Vec2{ x: 0.0, y: 1.0 }
        },
        &TriPoint::Corner(Corner::BottomRight) => {
            Vec2{ x: 1.0, y: 1.0 }
        },
        &TriPoint::Edge(_) => {
            Vec2::ZERO
        }
    };

    return vector;
}