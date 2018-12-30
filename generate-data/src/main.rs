// "610 6172458 mimi"
extern crate state;
extern crate brain;
extern crate nalgebra as na;
extern crate bincode;
extern crate flate2;

use state::*;
use brain::plan;
use bincode::serialize_into;
use na::{ Vector3, UnitQuaternion };
use flate2::write::GzEncoder;
use flate2::Compression;

use std::f32::consts::PI;
use std::fs::{ File, create_dir_all };
use std::path::Path;
use std::io::BufWriter;
use std::error::Error;

const SPEED_FACTOR: f32 = 100.0;
const POS_FACTOR: f32 = 200.0;
const YAW_FACTOR: f32 = 4.0;
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
    let mut player = PlayerState::default();
    let desired_contact = DesiredContact::default();

    let config = SearchConfig {
        step_duration: 16.0 * TICK,
        slop: 10.0,
        max_cost: 10.0,
        max_iterations: 500_000,
    };

    // XXX do we need avz?
    //for avz in min_avz..max_avz {

    let max_speed_r = (MAX_BOOST_SPEED / SPEED_FACTOR).round() as i32;
    for speed_r in -max_speed_r..=max_speed_r {
        for x_r in (-MAX_X / POS_FACTOR as i32)..=(MAX_X / POS_FACTOR as i32) {
            for y_r in (-MAX_Y / POS_FACTOR as i32)..=(MAX_Y / POS_FACTOR as i32) {
                for yaw_r in 0..YAW_FACTOR as i32 {
                    player.position.x = x_r as f32 * POS_FACTOR;
                    player.position.y = y_r as f32 * POS_FACTOR;
                    player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI + yaw_r as f32 * (2.0 * PI / YAW_FACTOR));
                    player.velocity = player.rotation * Vector3::new(-1.0 * speed_r as f32 * SPEED_FACTOR, 0.0, 0.0);

                    let plan_result = plan::hybrid_a_star(&player, &desired_contact, &config);

                    if let Some(plan) = plan_result.plan {
                        let path = format!(
                            "./data/generated/{}/{}/{}/{}/",
                            speed_r * SPEED_FACTOR as i32,
                            x_r * POS_FACTOR as i32,
                            y_r * POS_FACTOR as i32,
                            yaw_r,
                        );
                        write_data(&path, plan)?;
                        println!("Done: {:?}", player);
                    } else {
                        println!("Failed: {:?}", player);
                    }
                }
            }
        }
    }

    Ok(())
}
