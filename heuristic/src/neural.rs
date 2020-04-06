extern crate tensorflow;

use self::tensorflow::Graph;
use self::tensorflow::Operation;
use self::tensorflow::Session;
use self::tensorflow::SessionOptions;
use self::tensorflow::SessionRunArgs;
use self::tensorflow::Tensor;
use crate::{get_ball_position, get_normalization_rotation, HeuristicModel};
use na::{Rotation3, Unit, Vector3};
use state::{DesiredContact, PlayerState, BALL_RADIUS, CAR_DIMENSIONS};
use std::error::Error;
use std::fs::File;

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
    pub fn try_new(export_dir: &str) -> Result<Self, Box<dyn Error>> {
        let mut graph = Graph::new();
        let session =
            Session::from_saved_model(&SessionOptions::new(), &["serve"], &mut graph, export_dir)?;

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
    fn unscaled_heuristic(
        &mut self,
        players: &[PlayerState],
        costs: &mut [f32],
    ) -> Result<(), Box<dyn Error>> {
        let mut players_tensor = Tensor::new(&[players.len() as u64, 12u64]);
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
            players_tensor[offset + 9] = scale_rot(roll);
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

        Ok(())
    }

    fn scale(&self) -> f32 { self.scale }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.normalization_rotation = get_normalization_rotation(desired);
        self.ball_position = get_ball_position(desired);
        self.scale = scale;
    }
}
