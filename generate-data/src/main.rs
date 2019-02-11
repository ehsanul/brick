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

// the hybrid a* is not perfect. since it has some discretization/slop, and uses larger time steps
// for speed, and also since the heuristic used is not actully admissible (another perf thing). so
// in order to get a more accurate path for our data set, we need to do more searches along the
// path of the initial plan. this can result in a much better plan being found.
fn best_plan(player: &mut PlayerState, desired_contact: &DesiredContact, config: &SearchConfig) -> Option<Plan> {
    // the formula is based on our heuristic function, and the fact that we use 32-tick steps when
    // far away
    let iterations = if player.position.norm() / 1150.0 > 2.0 { 32 / 2 } else { (config.step_duration / (2.0 * TICK)).round() as i32 };
    let mut last_plan: Option<Plan> = None;
    let mut last_exploded_plan: Option<Plan> = None;
    let mut reset_at = 0;
    for i in 0..iterations {
        let plan_result = plan::hybrid_a_star(&player, &desired_contact, &config);

        let mut exploded_plan_result = plan_result.clone();
        plan::explode_plan(&mut exploded_plan_result);

        if let Some(exploded_plan) = exploded_plan_result.plan {
            if last_exploded_plan.is_none() || exploded_plan.len() < last_exploded_plan.as_ref().unwrap().len() {
                last_exploded_plan = Some(exploded_plan);
                last_plan = plan_result.plan;
                reset_at = i;
            }
        } else if last_exploded_plan.is_some() {
            // no plan was ever found this time, but we do have last_exploded_plan
            // we will advance using last_exploded_plan
        } else {

            // plan wasn't found in a previous iteration either.
            // advance straight by two ticks and retry
            player.position += 2.0 * TICK * player.velocity;
            continue
        }

        // advance two ticks along best plan so far
        // NOTE zeroth index is original player start
        let index = 2 + (i - reset_at) as usize * 2;
        if let Some((next_player, _, _)) = last_exploded_plan.as_ref().unwrap().get(index) {
            player.position = next_player.position;
            player.velocity = next_player.velocity;
            player.angular_velocity = next_player.angular_velocity;
            player.rotation = next_player.rotation;
        }
    }

    last_plan
}

fn main() -> Result<(), Box<Error>> {
    let mut player = PlayerState::default();
    let desired_contact = DesiredContact::default();

    let config = SearchConfig {
        step_duration: 16.0 * TICK,
        slop: 10.0,
        max_cost: 10.0,
        max_iterations: 10_000,
    };

    let slow_config = SearchConfig {
        step_duration: 16.0 * TICK,
        slop: 10.0,
        max_cost: 10.0,
        max_iterations: 100_000,
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

                    let path = format!(
                        "./data/generated/{}/{}/{}/{}/",
                        speed_r * SPEED_FACTOR as i32,
                        x_r * POS_FACTOR as i32,
                        y_r * POS_FACTOR as i32,
                        yaw_r,
                    );

                    if let Some(plan) = best_plan(&mut player.clone(), &desired_contact, &config) {
                        write_data(&path, plan)?;
                        println!("Done: {:?}", player);
                    } else if let Some(plan) = best_plan(&mut player.clone(), &desired_contact, &slow_config) {
                        write_data(&path, plan)?;
                        println!("SLOW Done: {:?}", player);
                    } else {
                        println!("Failed: {:?}", player);
                    }
                }
            }
        }
    }

    Ok(())
}
