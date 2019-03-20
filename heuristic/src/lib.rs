extern crate state;
extern crate tensorflow;
extern crate kdtree;
extern crate csv;
extern crate nalgebra as na;
extern crate ord_subset;

use std::error::Error;
use std::fs::File;
use tensorflow::Graph;
use tensorflow::Operation;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::SessionRunArgs;
use tensorflow::Tensor;

use kdtree::KdTree;

use state::{ PlayerState, DesiredContact, BALL_RADIUS, CAR_DIMENSIONS };
use na::{ Unit, Vector3, Rotation3 };
use std::f32::consts::PI;

use ord_subset::{OrdSubsetIterExt, OrdSubsetSliceExt};

pub trait HeuristicModel {
    fn heuristic(&mut self, players: &[PlayerState], costs: &mut [f32]) -> Result<(), Box<dyn Error>>;

    // NOTE scale is a fudge factor to make the heuristic over-estimate, which gives up
    // accuracy/optimality in exchange for speed
    fn configure(&mut self, desired: &DesiredContact, scale: f32);
}

pub struct NeuralHeuristic {
    session: Session,
    op_input: Operation,
    op_predict: Operation,
    ball_position: Vector3<f32>,
    normalization_rotation: Rotation3<f32>,
    scale: f32,
}

impl NeuralHeuristic {
    // export_dir is a directory like "./nn/simple_throttle_cost_saved_model/1551586435";
    pub fn try_new(export_dir: &str) -> Result<Self, Box<dyn Error>>{
        let mut graph = Graph::new();
        let session = Session::from_saved_model(&SessionOptions::new(),
                                                &["serve"],
                                                &mut graph,
                                                export_dir)?;

        let op_input = graph.operation_by_name_required("dense_input").unwrap();

        // NOTE tried to determine the operation name by looking at the graph inside tensorboard,
        // but it's pretty confusing tbh
        let op_predict = graph.operation_by_name_required("dense_2/BiasAdd").unwrap();

        Ok(NeuralHeuristic {
            session,
            op_input,
            op_predict,
            // set these in configure step
            ball_position: Vector3::new(0.0, 0.0, 0.0),
            normalization_rotation: Rotation3::from_euler_angles(0.0, 0.0, 0.0),
            scale: 1.0,
        })
    }
}

// scale to [0, 1]
fn scale(val: f32, min: f32, max: f32) -> f32 {
    (val - min) / (max - min)
}

// NOTE min/max values must match those used when scaling training data!
fn scale_pos(val: f32) -> f32 {
    scale(val, -10_000.0, 10_000.0)
}
fn scale_vel(val: f32) -> f32 {
    scale(val, -2300.0, 2300.0)
}
fn scale_avel(val: f32) -> f32 {
    scale(val, -6.0, 6.0)
}
fn scale_rot(val: f32) -> f32 {
    scale(val, -3.2, 3.2)
}

impl HeuristicModel for NeuralHeuristic {
    fn heuristic(&mut self, players: &[PlayerState], costs: &mut [f32]) -> Result<(), Box<dyn Error>> {
        let mut players_tensor = Tensor::new(&[players.len() as u64,  12u64]);
        for (i, player) in players.iter().enumerate() {
            let offset = i * 12;
            // FIXME use normalization rotation
            let pos = player.position; //self.normalization_rotation * (player.position - self.ball_position);
            players_tensor[offset + 0] = scale_pos(pos.x);
            players_tensor[offset + 1] = scale_pos(pos.y);
            players_tensor[offset + 2] = scale_pos(pos.z);

            // FIXME use normalization rotation
            let vel = player.velocity; //self.normalization_rotation * player.velocity;
            players_tensor[offset + 3] = scale_vel(vel.x);
            players_tensor[offset + 4] = scale_vel(vel.y);
            players_tensor[offset + 5] = scale_vel(vel.z);

            let avel = player.angular_velocity;
            players_tensor[offset + 6] = scale_avel(avel.x);
            players_tensor[offset + 7] = scale_avel(avel.y);
            players_tensor[offset + 8] = scale_avel(avel.z);

            // FIXME use normalization rotation
            let (roll, pitch, yaw) = player.rotation.euler_angles(); //(self.normalization_rotation * player.rotation).euler_angles();
            players_tensor[offset + 9 ] = scale_rot(roll);
            players_tensor[offset + 10] = scale_rot(pitch);
            players_tensor[offset + 11] = scale_rot(yaw);

            //println!("{:?}", [
            //    players_tensor[offset + 0],
            //    players_tensor[offset + 1],
            //    players_tensor[offset + 2],
            //    players_tensor[offset + 3],
            //    players_tensor[offset + 4],
            //    players_tensor[offset + 5],
            //    players_tensor[offset + 6],
            //    players_tensor[offset + 7],
            //    players_tensor[offset + 8],
            //    players_tensor[offset + 9],
            //    players_tensor[offset + 10],
            //    players_tensor[offset + 11],
            //]);
        }

        //&[
        //    0.609323f32, 0.667159, 1.0, 0.270244, 0.065328, 0.521546, 0.081916, 0.496829, 0.511105, 0.694068, 0.423122, 0.676769,
        //    0.607017f32, 0.500871, 1.0, 0.010846, 0.499089, 0.438751, 0.014231, 0.513214, 0.498750, 0.081110, 0.516453, 0.500443,
        //]

        let mut output_step = SessionRunArgs::new();
        output_step.add_feed(&self.op_input, 0, &players_tensor);
        let prediction_token = output_step.request_fetch(&self.op_predict, 0);
        self.session.run(&mut output_step)?;

        let predictions = output_step.fetch(prediction_token)?;
        costs.copy_from_slice(&predictions);

        for c in costs.iter_mut() { *c *= self.scale }

        Ok(())
    }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.normalization_rotation = get_normalization_rotation(desired);
        self.ball_position = get_ball_position(desired);
        self.scale = scale;
    }
}

#[derive(Debug)]
pub struct BasicHeuristic {
    goal_center: Vector3<f32>,
    desired_heading: Vector3<f32>,
    scale: f32,
}


impl BasicHeuristic {
    fn single_heuristic(&self, player: &PlayerState) -> f32 {
        // basic heuristic cost is a lower-bound for how long it would take, given max boost, to reach
        // the desired position and velocity. and we need to do rotation too.
        //
        // NOTE for now we ignore the fact that we are not starting at the max boost velocity pointed
        // directly at the desired position. the heuristic just needs to be a lower bound, until we
        // want to get it more accurate and thus ignore irrelevant branches more efficiently.
        let towards_goal = self.goal_center - player.position;
        let distance = towards_goal.norm();

        // XXX more correct to use predict::player::MAX_BOOST_SPEED, but it checks way too many paths.
        // with a lower value, ie higher heuristic cost, we get a potentially less optimal path, but we
        // get it a lot faster. it's not so bad given that we aren't actually going in a straight line
        // boosting at max speed anyways
        let movement_time_cost = distance / 1150.0;

        // basic penalty for being on the wrong side of the ball which will require a big turn. this
        // allows us to forgo searching right near the ball on the wrong side when it'll never work
        // out.
        let current_heading = player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let car_to_desired = Unit::new_normalize(self.goal_center - player.position).into_inner();
        let mut penalty_time_cost = if distance < 800.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.70 {
            0.5
        } else if distance < 1500.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.88 {
            0.5
        } else if distance < 2000.0 && na::Matrix::dot(&self.desired_heading, &car_to_desired) < -0.95 {
            0.5
        } else {
            0.0
        };
        // we have a tighter radius when slow, the numbers above are tuned for going fast
        if player.velocity.norm() < 800.0 {
            penalty_time_cost *= 0.2;
        }
        // if passing sideways, the penalty should be way lower since we're moving out of the deadzone
        penalty_time_cost *= na::Matrix::dot(&current_heading, &car_to_desired).abs();

        movement_time_cost + penalty_time_cost
    }
}

impl Default for BasicHeuristic {
    fn default() -> BasicHeuristic {
        BasicHeuristic {
            goal_center: Vector3::new(0.0, 0.0, 0.0),
            desired_heading: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl HeuristicModel for BasicHeuristic {
    fn heuristic(&mut self, players: &[PlayerState], costs: &mut [f32]) -> Result<(), Box<dyn Error>> {
        assert!(players.len() == costs.len());
        for (i, cost) in costs.iter_mut().enumerate() {
            let player = unsafe { players.get_unchecked(i) };
            *cost = self.single_heuristic(player) * self.scale;
        }

        Ok(())
    }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.desired_heading = Unit::new_normalize(desired.heading.clone()).into_inner();
        self.goal_center = desired.position - (CAR_DIMENSIONS.x / 2.0) * self.desired_heading;
        self.scale = scale;
    }
}

const KNN_DIMENSIONS: usize = 3; // x, y. yaw

#[derive(Debug)]
pub struct KnnHeuristic {
    tree: KdTree<f32, f32, [f32; KNN_DIMENSIONS]>,
    ball_position: Vector3<f32>,
    normalization_rotation: Rotation3<f32>,
    scale: f32,
}

// so that yaw distance is in the same ballpark as positional distance
const SCALE_CIRCULAR_DISTANCE: f32 = 500.0;

// +PI and -PI are the same angle, so the distance needs to take that into account!
fn circular_distance(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs().min(2.0 * PI + a - b).min(2.0 * PI + b - a);
    SCALE_CIRCULAR_DISTANCE * distance
}

fn squared_distance(a: f32, b: f32) -> f32 {
    (b - a).powf(2.0)
}

fn knn_distance(a: &[f32], b: &[f32]) -> f32 {
    squared_distance(a[0], b[0]) + squared_distance(a[1], b[1]) + circular_distance(a[2], b[2]).powf(2.0)
}

impl KnnHeuristic {
    pub fn try_new(path: &str) -> Result<Self, Box<dyn Error>>{
        let mut tree = KdTree::new(KNN_DIMENSIONS);

        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(File::open(path)?);
        //tree.add(&x, 99.0).unwrap();
        for record in rdr.records() {
          let record = record?;
          let cost = record.get(0).expect("Invalid row?").parse()?;
          let x = record.get(1).expect("Invalid row?").parse()?;
          let y = record.get(2).expect("Invalid row?").parse()?;
          let yaw = record.get(12).expect("Invalid row?").parse()?;
            tree.add([x, y, yaw], cost)?;
        }

        Ok(KnnHeuristic {
            tree,
            // set the rest in configure step
            ..Default::default()
        })
    }

    fn single_heuristic(&self, player: &PlayerState) -> f32 {
        let pos = self.normalization_rotation * (player.position - self.ball_position);
        let (_roll, _pitch, yaw) = (self.normalization_rotation * player.rotation).euler_angles();
        let point = [pos.x, pos.y, yaw];
        let nearest = self.tree.nearest(&point, 3, &knn_distance).unwrap();

        let max_distance: f32 = *nearest.iter().map(|(d, _)| d).ord_subset_max().unwrap();
        let total_weights: f32 = nearest.iter().map(|(d, _)| max_distance / d).sum();
        let weighted_average_cost: f32 = nearest.iter().map(|(distance, &cost)| {
            let weight = max_distance / distance;
            weight * cost
        }).sum::<f32>() / total_weights;

        weighted_average_cost
    }
}

impl Default for KnnHeuristic {
    fn default() -> KnnHeuristic {
        KnnHeuristic {
            tree: KdTree::new(KNN_DIMENSIONS),
            ball_position: Vector3::new(0.0, 0.0, 0.0),
            normalization_rotation: Rotation3::from_euler_angles(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl HeuristicModel for KnnHeuristic {
    fn heuristic(&mut self, players: &[PlayerState], costs: &mut [f32]) -> Result<(), Box<dyn Error>> {
        assert!(players.len() == costs.len());
        for (i, cost) in costs.iter_mut().enumerate() {
            let player = unsafe { players.get_unchecked(i) };
            *cost = self.single_heuristic(player) * self.scale;
        }

        Ok(())
    }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.normalization_rotation = get_normalization_rotation(desired);
        self.ball_position = get_ball_position(desired);
        self.scale = scale;
    }
}

fn get_normalization_rotation(desired: &DesiredContact) -> Rotation3<f32> {
    // the training data is based on the ball positioned at 0, 0, and the desired heading being
    // directly in the positive y axis. given the current heading, we want to find
    // a transformation matrix that would tranform it into the standard heading, which we can
    // apply to the car in order to align with how we trained.
    let standard_heading = Vector3::new(0.0, 1.0, 0.0);
    let heading = desired.heading / desired.heading.norm();

    let mut angle = na::Matrix::dot(&standard_heading, &heading).acos();

    // if standard is to the right, we need to rotate clockwise
    // https://math.stackexchange.com/a/555243
    let delta = heading.x * standard_heading.y - heading.y * standard_heading.x;
    if delta < 0.0 {
        angle *= -1.0;
    }

    Rotation3::from_euler_angles(0.0, 0.0, angle)
}


fn get_ball_position(desired: &DesiredContact) -> Vector3<f32> {
    desired.position + BALL_RADIUS * desired.heading
}
