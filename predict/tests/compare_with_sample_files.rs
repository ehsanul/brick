extern crate predict;
extern crate state;

use state::*;
use std::f32::consts::PI;
extern crate nalgebra as na;
use na::{Point3, Rotation3, Translation3, Unit, UnitQuaternion, Vector3};


#[test]
fn test_throttle_straight() {
    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    for full_sample in predict::sample::THROTTLE_STRAIGHT_ALL.iter() {
        let mut i = 0;
        // offset by 32 frames to ensure minimum 32 frames of simulation ahead in the slice
        while i + 32 < full_sample.len() {
            let sample = &full_sample[i..];
            i += 1;

            let player_start = sample[0];
            let player_end = sample[16];

            // we can miss data at the edges, ignore for now
            if (player_start.angular_velocity.z.abs() / 0.2) >= 10.0 || player_start.local_velocity().y < -100.0 {
                continue;
            }

            let predicted_player_end = predict::player::next_player_state(&player_start, &controller, 16.0 * TICK);
            let position_error = (predicted_player_end.position - player_end.position).norm();
            println!("position error: {}", position_error);
            if position_error > 10.0 {
                println!("");
                println!("---------------------------------------------------------");
                println!("position error: {}, frame: {}, x: {}", position_error, player_start.angular_velocity.x, player_start.position.x);
                println!("player_start: {:?}", player_start);
                println!("expected player_end: {:?}", player_end);
                println!("predicted player_end: {:?}", predicted_player_end);
            }

            assert!(position_error < 250.0);
        }
    }
}
