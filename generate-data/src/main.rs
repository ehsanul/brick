// "610 6172458 mimi"
extern crate state;
extern crate brain;
extern crate nalgebra as na;

use state::*;
use brain::plan;
use std::f32::consts::PI;
use na::{ Vector3, UnitQuaternion };

const SPEED_FACTOR: f32 = 100.0;
const POS_FACTOR: f32 = 200.0;
const YAW_FACTOR: f32 = 4.0;

fn write_data(plan: &Plan) {
}

fn main() {
    let mut player = PlayerState::default();
    let mut desired_contact = DesiredContact::default();

    let config = SearchConfig {
        step_duration: 16.0 * TICK,
        slop: 10.0,
        max_cost: 10.0,
        max_iterations: 200_000,
    };


    // XXX do we need avz?
    //for avz in min_avz..max_avz {

    let max_speed_r = (MAX_BOOST_SPEED / SPEED_FACTOR).round() as i32;
    for speed_r in -max_speed_r..=max_speed_r {
        for x_r in (-8000 / POS_FACTOR as i32)..=(8000 / POS_FACTOR as i32) {
            for y_r in (-10000 / POS_FACTOR as i32)..=(10000 / POS_FACTOR as i32) {
                for yaw_r in 0..YAW_FACTOR as i32 {
                    let mut pos = player.position;
                    pos.x = x_r as f32 * POS_FACTOR;
                    pos.y = y_r as f32 * POS_FACTOR;
                    player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI + yaw_r as f32 * (2.9 * PI / YAW_FACTOR));
                    player.velocity = player.rotation * Vector3::new(-1.0 * speed_r as f32 * SPEED_FACTOR, 0.0, 0.0);

                    let plan_result = plan::hybrid_a_star(&player, &desired_contact, &config);

                    if let Some(plan) = plan_result.plan {
                        write_data(&plan);
                    } else {
                        println!("Failed: {:?}", player);
                    }
                }
            }
        }
    }
}
