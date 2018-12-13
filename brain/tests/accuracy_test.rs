extern crate brain;
extern crate state;
extern crate nalgebra as na;

use na::{ Unit, Vector3, UnitQuaternion, Rotation3 };
use state::{ PlayerState, PlanResult, DesiredContact };
use brain::plan::hybrid_a_star;
use std::f32::consts::PI;

fn run(name: &str, x: f32, y: f32, yaw: f32, step: f32) {
    // need to re-do the accuracy calculation since we now need to check how accurate our contact
    // with the ball is instead
    assert!(false);
    return;

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

        let mut desired = DesiredContact::default();
        desired.position.x = x * (17.0 * i as f32).sin();
        desired.position.y = y + (y / 4.0) * (11.0 * i as f32).sin();

        let desired_yaw = if desired.position.norm() > 200.0 {
            yaw + (13.0 * i as f32).sin() * PI/4.0
        } else {
            // when close, some angles are not really reachable
            yaw
        };
        desired.heading = Rotation3::from_euler_angles(0.0, 0.0, desired_yaw) * desired.heading;

        let PlanResult { plan, visualization_lines, visualization_points, .. } = hybrid_a_star(&player, &desired, step);
        if plan.is_none() {
            println!("FAILED | pos: {:?} | yaw: {}", desired.position, desired_yaw);
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

            if expansions > 100000 {
                println!("SLOW | pos: {:?} | yaw: {}", desired.position, desired_yaw);
            }
            assert!(expansions < 100000);

            let last = plan[plan_len - 1];
            let last2 = plan[plan_len - 2];

            // https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Vector_formulation
            //
            // FIXME THIS IS WRONG NOW! we have an array of goals now, so we'd have to find which
            // one we did best on. we don't provided desired player position anymore, instead it's
            // desired contact & heading!
            //
            //let a = last2.0.position;
            //let n = Unit::new_normalize(last.0.position - last2.0.position).unwrap();
            //let p = desired.position;
            //let position_error = ((a - p) - na::dot(&(a - p), &n) * n).norm();
            //total_position_error_squared += position_error * position_error;
            //if position_error > max_position_error { max_position_error = position_error }
            //if position_error < min_position_error { min_position_error = position_error }
            //
            //let desired_heading = desired_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            //let heading = last2.0.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            //let rotation_error = 1.0 - na::dot(&heading, &desired_heading);
            //total_rotation_error_squared += rotation_error * rotation_error;
            //if rotation_error > max_rotation_error { max_rotation_error = rotation_error }
            //if rotation_error < min_rotation_error { min_rotation_error = rotation_error }
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
