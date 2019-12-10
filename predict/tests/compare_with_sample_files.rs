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

fn compare<'a>(controller: BrickControllerState, all_samples: impl Iterator<Item = &'a Vec<PlayerState>>) {
    let mut position_errors = vec![];
    let mut velocity_errors = vec![];
    let mut avz_errors = vec![];
    let mut max_position_error = 0.0;
    let mut max_velocity_error = 0.0;
    let mut max_avz_error = 0.0;

    for full_sample in all_samples {
        let mut i = 0;
        // offset by 32 frames to ensure minimum 32 frames of simulation ahead in the slice
        while full_sample[i..].len() >= predict::sample::MIN_SAMPLE_LENGTH {
            let sample = &full_sample[i..];
            i += 1;

            let player_start = sample[0];
            let player_end = sample[16 / 2];

            // we can miss data at the edges, or not be able to extrapolate at the dges. ignore for now
            // TODO remove all these checks
            if (player_start.angular_velocity.z.abs() / 0.2) >= 20.0 || player_start.local_velocity().y < -100.0 || player_start.local_velocity().x.abs() > 1000.0 {
                continue;
            }

            let predicted_player_end = predict::player::next_player_state(&player_start, &controller, 16.0 * TICK);
            let position_error = (predicted_player_end.position - player_end.position).norm();
            let velocity_error = (predicted_player_end.velocity - player_end.velocity).norm();
            let avz_error = (predicted_player_end.angular_velocity.z - player_end.angular_velocity.z).abs();

            position_errors.push(position_error);
            velocity_errors.push(velocity_error);
            avz_errors.push(avz_error);

            if position_error > max_position_error {
                max_position_error = position_error;
            }
            if velocity_error > max_velocity_error {
                max_velocity_error = velocity_error;
            }
            if avz_error > max_avz_error {
                max_avz_error = avz_error;
            }

            //println!("position error: {}", position_error);
            //if position_error > 20.0 {
            if velocity_error > 250.0 {
                println!("");
                println!("---------------------------------------------------------");
                println!("position error: {}, velocity_error: {}, avz_error: {}, frame: {}, x: {}", position_error, velocity_error, avz_error, player_start.angular_velocity.x, player_start.position.x);
                println!("player_start: {:?}", player_start);
                println!("expected player_end: {:?}", player_end);
                println!("predicted player_end: {:?}", predicted_player_end);
            }
        }
    }

    println!("");
    println!("");
    println!("---------------------------------------------------------");
    println!("{:?}", controller);
    println!("max position error: {}", max_position_error);
    println!("rms position error: {}", rms(&position_errors));
    println!("50th percentile position error: {}", percentile_value(&mut position_errors, 50.0));
    println!("95th percentile position error: {}", percentile_value(&mut position_errors, 95.0));
    println!("99th percentile position error: {}", percentile_value(&mut position_errors, 99.0));
    println!("99.9th percentile position error: {}", percentile_value(&mut position_errors, 99.9));

    println!("max velocity error: {}", max_velocity_error);
    println!("rms velocity error: {}", rms(&velocity_errors));
    println!("50th percentile velocity error: {}", percentile_value(&mut velocity_errors, 50.0));
    println!("95th percentile velocity error: {}", percentile_value(&mut velocity_errors, 95.0));
    println!("99th percentile velocity error: {}", percentile_value(&mut velocity_errors, 99.0));
    println!("99.9th percentile velocity error: {}", percentile_value(&mut velocity_errors, 99.9));

    println!("max avz error: {}", max_avz_error);
    println!("rms avz error: {}", rms(&avz_errors));
    println!("50th percentile avz error: {}", percentile_value(&mut avz_errors, 50.0));
    println!("95th percentile avz error: {}", percentile_value(&mut avz_errors, 95.0));
    println!("99th percentile avz error: {}", percentile_value(&mut avz_errors, 99.0));
    println!("99.9th percentile avz error: {}", percentile_value(&mut avz_errors, 99.9));
    println!("");
    println!("");

    // TODO investigate the worst offenders and reduce these values
    assert!(max_position_error < 20.0);
    assert!(rms(&position_errors) < 0.6);
    assert!(percentile_value(&mut position_errors, 50.0) < 0.5);
    assert!(percentile_value(&mut position_errors, 95.0) < 1.5);
    assert!(percentile_value(&mut position_errors, 99.0) < 2.5);
    assert!(percentile_value(&mut position_errors, 99.9) < 7.5);

    assert!(rms(&velocity_errors) < 9.5);
    assert!(rms(&avz_errors) < 0.15);
}

#[test]
fn test_throttle_straight() {
    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;
    let all_samples = predict::sample::THROTTLE_STRAIGHT_ALL.iter();
    compare(controller, all_samples);
}

#[test]
fn test_throttle_left() {
    let mut controller = BrickControllerState::new();
    controller.steer = Steer::Left;
    controller.throttle = Throttle::Forward;
    let all_samples = predict::sample::THROTTLE_LEFT_ALL.iter();
    compare(controller, all_samples);
}

#[test]
fn test_throttle_right() {
    let mut controller = BrickControllerState::new();
    controller.steer = Steer::Right;
    controller.throttle = Throttle::Forward;
    let all_samples = predict::sample::THROTTLE_RIGHT_ALL.iter();
    compare(controller, all_samples);
}

#[test]
fn test_boost_straight() {
    let mut controller = BrickControllerState::new();
    controller.boost = true;
    let all_samples = predict::sample::BOOST_STRAIGHT_ALL.iter();
    compare(controller, all_samples);
}

#[test]
fn test_boost_left() {
    let mut controller = BrickControllerState::new();
    controller.steer = Steer::Left;
    controller.boost = true;
    let all_samples = predict::sample::BOOST_LEFT_ALL.iter();
    compare(controller, all_samples);
}

#[test]
fn test_boost_right() {
    let mut controller = BrickControllerState::new();
    controller.steer = Steer::Right;
    controller.boost = true;
    let all_samples = predict::sample::BOOST_RIGHT_ALL.iter();
    compare(controller, all_samples);
}
