use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use obj::*;
use ncollide::shape::TriMesh3;
use na::Point3;

pub static SIDE_WALL_DISTANCE: f32 = 4096.0;
pub static BACK_WALL_DISTANCE: f32 = 5140.0;
pub static CEILING_DISTANCE: f32 = 2044.0;
pub static GOAL_X: f32 = 892.75;
pub static GOAL_Z: f32 = 640.0;

lazy_static! {
    pub static ref ARENA: TriMesh3<f32> = {
        let file = File::open("./assets/arena.obj").expect("Couldn't open arena.obj file");
        let input = BufReader::new(file);
        let arena: Obj = load_obj(input).expect("failed to parse arena.obj file in predict");

        let vertices: Vec<Point3<f32>> = arena.vertices.iter().map(|vert|
            Point3::new(vert.position[0], vert.position[1], vert.position[2])
        ).collect();

        let indices: Vec<Point3<usize>> = arena.indices.chunks(3).map(|indices|
            Point3::new(indices[0] as usize, indices[1] as usize, indices[2] as usize)
        ).collect();

        TriMesh3::new(
            Arc::new(vertices), // Arc<Vec<P>>,
            Arc::new(indices), // Arc<Vec<Point<usize, U3>>>,
            None, // uvs: Option<Arc<Vec<Point<<P as EuclideanSpace>::Real, U2>>>>,
            None, // normals: Option<Arc<Vec<<P as Point>::Vector>>>
        )
    };
}
