extern crate bincode;
extern crate brain;
extern crate flate2;
extern crate nalgebra as na;
extern crate state;
extern crate rayon;

use bincode::serialize_into;
use brain::plan;
use brain::HeuristicModel; // TODO as _;
use flate2::write::GzEncoder;
use flate2::Compression;
use na::{UnitQuaternion, Vector3};
use rayon::prelude::*;
use state::*;

use std::error::Error;
use std::f32::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

const SPEED_FACTOR: f32 = 1000.0;
const POS_FACTOR: f32 = 500.0;
const YAW_FACTOR: f32 = 8.0;
const MAX_X: i32 = 8000;
const MAX_Y: i32 = 10000;

fn write_data(path: &str, plan: Plan) -> Result<(), Box<Error>> {
    let serializable_plan = SerializablePlan(plan);
    create_dir_all(&path)?;
    let file_path = Path::new(path).join("plan.bincode");
    let f = BufWriter::new(File::create(file_path)?);
    let mut e = GzEncoder::new(f, Compression::default());
    Ok(serialize_into(&mut e, &serializable_plan)?)
}

fn main() -> Result<(), Box<Error>> {
    // fewer than normal  threads, so some cores are left free for the computer user...
    rayon::ThreadPoolBuilder::new().num_threads(10).build_global().unwrap();

    let desired_ball_position: Vector3<f32> = brain::play::opponent_goal_shoot_at(&GameState::default());
    let ball = BallState::default();
    let desired_contact = brain::play::simple_desired_contact(&ball, &desired_ball_position);

    let config = SearchConfig {
        step_duration: 16.0 * TICK,
        slop: 40.0,
        max_cost: 10.0,
        max_iterations: 10_000_000, // allow more iterations before giving up
        scale_heuristic: 1.0,
        custom_filter: Some(|_| { true }), // ignore bounds
    };

    let max_speed_r = (MAX_BOOST_SPEED / SPEED_FACTOR).round() as i32;
    // TODO negative vy
    //(-max_speed_r..=max_speed_r).into_par_iter().for_each(|speed_r| {
    (0..=max_speed_r).into_par_iter().for_each(|speed_r| {
        ((-MAX_X / POS_FACTOR as i32)..=(MAX_X / POS_FACTOR as i32)).into_par_iter().for_each(|x_r| {
            ((-MAX_Y / POS_FACTOR as i32)..=(MAX_Y / POS_FACTOR as i32)).into_par_iter().for_each(|y_r| {
                (0..YAW_FACTOR as i32).into_par_iter().for_each(|yaw_r| {
                    let mut player = PlayerState::default();
                    let mut model = brain::BasicHeuristic::default();

                    player.position.x = x_r as f32 * POS_FACTOR;
                    player.position.y = y_r as f32 * POS_FACTOR;
                    let yaw = -PI + yaw_r as f32 * (2.0 * PI / YAW_FACTOR);
                    player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, yaw);
                    player.velocity = player.rotation
                        * Vector3::new(-1.0 * speed_r as f32 * SPEED_FACTOR, 0.0, 0.0);

                    let path = format!(
                        "./data/generated/vy_{}/x_{}/y_{}/yaw_{}/",
                        speed_r * SPEED_FACTOR as i32,
                        x_r * POS_FACTOR as i32,
                        y_r * POS_FACTOR as i32,
                        yaw_r as f32 / YAW_FACTOR,
                    );

                    if PathBuf::from(&path).exists() {
                        //println!("Path already exists: {}", path);
                        return;
                    }

                    if let Some(plan) = plan::hybrid_a_star(
                        &mut model,
                        &mut player.clone(),
                        &ball,
                        &desired_contact,
                        &config,
                    ).plan {
                        write_data(&path, plan).expect("writing failed");
                        println!("Done: x: {}, y: {}, local_vy: {}, yaw: {}", player.position.x, player.position.y, speed_r as f32 * SPEED_FACTOR, yaw);
                    } else {
                        println!("Failed: x: {}, y: {}, local_vy: {}, yaw: {}", player.position.x, player.position.y, speed_r as f32 * SPEED_FACTOR, yaw);
                    }
                })
            })
        })
    });

    Ok(())
}
