extern crate brain;
extern crate state;
extern crate nalgebra as na;

use na::{ Unit };
use state::{ PlayerState, PlanResult };
use brain::plan::hybrid_a_star;

fn run(name: &str, x: f32, y: f32, step: f32) {
    let mut total_error_squared = 0.0;
    let mut min_error = std::f32::MAX;
    let mut max_error = 0.0;
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
            //let error = (last.0.position - desired_player.position).norm();
            // https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Vector_formulation
            let a = last2.0.position;
            let n = Unit::new_normalize(last.0.position - last2.0.position).unwrap();
            let p = desired_player.position;
            let error = ((a - p) - na::dot(&(a - p), &n) * n).norm();
            total_error_squared += error * error;
            if error > max_error { max_error = error }
            if error < min_error { min_error = error }
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
    println!("rms error: {}", (total_error_squared / (num as f32)).sqrt());
    //println!("min error: {}", min_error);
    println!("max error: {}", max_error);
    println!("----------------------------");
}

#[test]
fn close_and_forward_4_step() {
    run("close and forward, 10-step", 10.0, 80.0, 4.0/120.0);
}

#[test]
fn close_and_forward_10_step() {
    run("close and forward, 10-step", 100.0, 400.0, 10.0/120.0);
}

#[test]
fn close_and_forward_20_step() {
    run("close and forward, 20-step", 100.0, 400.0, 20.0/120.0);
}

#[test]
fn medium_and_forward_20_step() {
    run("medium and forward, 20-step", 500.0, 800.0, 20.0/120.0);
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
