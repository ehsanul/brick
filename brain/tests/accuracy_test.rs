extern crate brain;
extern crate state;
extern crate nalgebra as na;

use na::{ Unit, Vector3, UnitQuaternion };
use state::{ PlayerState, PlanResult };
use brain::plan::hybrid_a_star;
use std::f32::consts::PI;

fn run(name: &str, x: f32, y: f32, yaw: f32, step: f32) {
    let mut total_position_error_squared = 0.0;
    let mut min_position_error = std::f32::MAX;
    let mut max_position_error = 0.0;

    let mut total_rotation_error_squared = 0.0;
    let mut min_rotation_error = std::f32::MAX;
    let mut max_rotation_error = 0.0;

    let mut total_expansions = 0;
    let mut min_expansions = std::usize::MAX;
    let mut max_expansions = 0;

    let mut total_plan_len = 0;
    let mut min_plan_len = std::usize::MAX;
    let mut max_plan_len = 0;
    let num = 100;
    for i in 0..num {
        let mut player = PlayerState::default();

        let mut desired_player = PlayerState::default();
        desired_player.position.x = x * (17.0 * i as f32).sin();
        desired_player.position.y = y + (y / 4.0) * (11.0 * i as f32).sin();
        desired_player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, yaw + (13.0 * i as f32).sin() * PI/4.0);

        let PlanResult { plan, visualization_lines, visualization_points } = hybrid_a_star(&player, &desired_player, step);
        if plan.is_none() {
            println!("FAILED: {:?}", desired_player.position);
        }
        assert!(plan.is_some());

        if let Some(plan) = plan {
            let plan_len = plan.len();
            total_plan_len += plan_len;
            if plan_len > max_plan_len { max_plan_len = plan_len }
            if plan_len < min_plan_len { min_plan_len = plan_len }

            let expansions = visualization_points.len();
            total_expansions += expansions;
            if expansions > max_expansions { max_expansions = expansions }
            if expansions < min_expansions { min_expansions = expansions }

            let last = plan[plan_len - 1];
            let last2 = plan[plan_len - 2];
            //let position_error = (last.0.position - desired_player.position).norm();
            // https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Vector_formulation
            let a = last2.0.position;
            let n = Unit::new_normalize(last.0.position - last2.0.position).unwrap();
            let p = desired_player.position;
            let position_error = ((a - p) - na::dot(&(a - p), &n) * n).norm();
            total_position_error_squared += position_error * position_error;
            if position_error > max_position_error { max_position_error = position_error }
            if position_error < min_position_error { min_position_error = position_error }

            let desired_heading = desired_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            let heading = last2.0.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            let rotation_error = 1.0 - na::dot(&heading, &desired_heading);
            total_rotation_error_squared += rotation_error * rotation_error;
            if rotation_error > max_rotation_error { max_rotation_error = rotation_error }
            if rotation_error < min_rotation_error { min_rotation_error = rotation_error }
        }
    }
    println!("");
    println!("----------------------------");
    println!("avg expansions: {}", (total_expansions as f32) / (num as f32));
    //println!("min expansions: {}", min_expansions);
    println!("max expansions: {}", max_expansions);
    println!("----------------------------");
    println!("avg plan_len: {}", (total_plan_len as f32) / (num as f32));
    //println!("min plan_len: {}", min_plan_len);
    println!("max plan_len: {}", max_plan_len);
    println!("----------------------------");
    println!("rms position error: {}", (total_position_error_squared / (num as f32)).sqrt());
    //println!("min error: {}", min_position_error);
    println!("max position error: {}", max_position_error);
    println!("----------------------------");
    println!("rms rotation error: {}", (total_rotation_error_squared / (num as f32)).sqrt());
    //println!("min error: {}", min_rotation_error);
    println!("max rotation error: {}", max_rotation_error);
    println!("----------------------------");
}

#[test]
fn close_and_forward_4_step() {
    run("close and forward, 10-step", 10.0, 80.0, -PI/2.0, 4.0/120.0);
}

#[test]
fn close_and_forward_10_step() {
    run("close and forward, 10-step", 100.0, 400.0, -PI/2.0, 10.0/120.0);
}

#[test]
fn close_and_forward_20_step() {
    run("close and forward, 20-step", 100.0, 400.0, -PI/2.0, 20.0/120.0);
}

#[test]
fn medium_and_forward_20_step() {
    run("medium and forward, 20-step", 500.0, 800.0, -PI/2.0, 20.0/120.0);
}

#[test]
fn medium_180_and_forward_20_step() {
    run("medium and forward, 20-step", 500.0, 800.0, PI/2.0, 20.0/120.0);
}

#[test]
fn medium_90_and_forward_20_step() {
    run("medium and forward, 20-step", 500.0, 800.0, 0.0, 20.0/120.0);
}

/* LAST RESULT */
/*
----------------------------
avg expansions: 309.03
max expansions: 663
----------------------------
avg plan_len: 6.33
max plan_len: 7
----------------------------
rms error: 9.333805
max error: 27.308598
----------------------------
test close_and_forward_20_step ... ok

----------------------------
avg expansions: 551.64
max expansions: 1293
----------------------------
avg plan_len: 11.56
max plan_len: 13
----------------------------
rms error: 0.663478
max error: 1.1688014
----------------------------
test close_and_forward_4_step ... ok

----------------------------
avg expansions: 1101.15
max expansions: 4623
----------------------------
avg plan_len: 9.2
max plan_len: 11
----------------------------
rms error: 9.416299
max error: 23.394838
----------------------------
test medium_and_forward_20_step ... ok

----------------------------
avg expansions: 1234.5
max expansions: 2571
----------------------------
avg plan_len: 11.01
max plan_len: 12
----------------------------
rms error: 4.6374955
max error: 16.530273
----------------------------
test close_and_forward_10_step ... ok
*/
