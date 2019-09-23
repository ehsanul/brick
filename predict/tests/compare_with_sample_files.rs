extern crate predict;
extern crate state;

use state::*;
use std::cmp::Ordering;

fn percentile_value(numbers: &mut Vec<f32>, percentile: f32) -> f32 {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));
    let i = (percentile * numbers.len() as f32) as usize / 100;
    numbers[i]
}

fn rms(numbers: &Vec<f32>) -> f32 {
    let total_error_squared: f32 = numbers.iter().map(|x| x * x).sum();
    (total_error_squared / (numbers.len() as f32)).sqrt()
}

#[test]
fn test_throttle_straight() {
    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    let mut position_errors = vec![];
    let mut max_position_error = 0.0;

    for full_sample in predict::sample::THROTTLE_STRAIGHT_ALL.iter() {
        let mut i = 0;
        // offset by 32 frames to ensure minimum 32 frames of simulation ahead in the slice
        while full_sample[i..].len() >= predict::sample::MIN_SAMPLE_LENGTH {
            let sample = &full_sample[i..];
            i += 1;

            let player_start = sample[0];
            let player_end = sample[16];

            // we can miss data at the edges, or not be able to extrapolate at the dges. ignore for now
            // TODO remove all these checks
            if (player_start.angular_velocity.z.abs() / 0.2) >= 10.0 || player_start.local_velocity().y < -100.0 || player_start.local_velocity().x.abs() > 1000.0 || player_start.velocity.norm() > 2100.0 {
                continue;
            }

            let predicted_player_end = predict::player::next_player_state(&player_start, &controller, 16.0 * TICK);
            let position_error = (predicted_player_end.position - player_end.position).norm();

            position_errors.push(position_error);
            if position_error > max_position_error {
                max_position_error = position_error;
            }

            println!("position error: {}", position_error);
            if position_error > 20.0 {
                println!("");
                println!("---------------------------------------------------------");
                println!("position error: {}, frame: {}, x: {}", position_error, player_start.angular_velocity.x, player_start.position.x);
                println!("player_start: {:?}", player_start);
                println!("expected player_end: {:?}", player_end);
                println!("predicted player_end: {:?}", predicted_player_end);
            }
        }
    }

    println!("max position error: {}", max_position_error);
    println!("rms position error: {}", rms(&position_errors));
    println!("50th percetile position error: {}", percentile_value(&mut position_errors, 50.0));
    println!("95th percetile position error: {}", percentile_value(&mut position_errors, 95.0));
    println!("99th percetile position error: {}", percentile_value(&mut position_errors, 99.0));
    println!("99.9th percetile position error: {}", percentile_value(&mut position_errors, 99.9));

    // TODO investigate the worst offenders and reduce these values
    assert!(max_position_error < 23.0);
    assert!(percentile_value(&mut position_errors, 50.0) < 0.12);
    assert!(percentile_value(&mut position_errors, 95.0) < 2.0);
    assert!(percentile_value(&mut position_errors, 99.0) < 2.1);
    assert!(percentile_value(&mut position_errors, 99.9) < 11.0);
}
