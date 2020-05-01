use crate::HeuristicModel; // TODO as _;
use na::{self, Rotation3, Unit, Vector3};
use plan;
use predict;
use rlbot;
use state::*;
use std;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::time::Instant;

// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
fn what_do(_game: &GameState) -> Action {
    Action::Shoot // TODO
}

pub fn opponent_goal_shoot_at(game: &GameState) -> Vector3<f32> {
    // TODO calculate which part of the goal is hardest for the opponent to reach
    match game.player.team {
        // FIXME check if we have this right...
        Team::Blue => Vector3::new(0.0, BACK_WALL_DISTANCE, GOAL_Z / 2.0),
        Team::Orange => Vector3::new(0.0, -BACK_WALL_DISTANCE, GOAL_Z / 2.0),
    }
}

/// guess best point on ball to hit, get the heading at that point
#[no_mangle]
pub extern "C" fn simple_desired_contact(
    ball: &BallState,
    desired_ball_position: &Vector3<f32>,
) -> DesiredContact {
    let desired_vector = Unit::new_normalize(desired_ball_position - ball.position);
    let desired_velocity = 3000.0 * desired_vector.into_inner();
    let velocity_delta = desired_velocity - ball.velocity;

    // this is pretty crude, doesn't even consider that the ball will undergo gravity after the
    // hit! but should be good enough for us here for now
    let impulse_direction = Unit::new_normalize(velocity_delta);
    let ball_normal = -1.0 * impulse_direction.into_inner();

    DesiredContact {
        position: ball.position + BALL_COLLISION_RADIUS * ball_normal,
        heading: -1.0 * ball_normal,
    }
}

fn reachable_desired_player_state<H: HeuristicModel>(
    model: &mut H,
    player: &PlayerState,
    ball_trajectory: &[BallState],
    desired_ball_position: &Vector3<f32>,
) -> DesiredContact {
    let start = Instant::now();

    let mut estimates = ball_trajectory
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            let desired_contact = simple_desired_contact(ball, &desired_ball_position);
            let shooting_time = non_admissable_estimated_time(model, &player, &desired_contact);
            (i, shooting_time)
        })
        .collect::<Vec<_>>();
    estimates.sort_by(|(i, shooting_time), (i2, shooting_time2)| {
        let ball_time = (*i as f32) * TICK;
        let ball_time2 = (*i2 as f32) * TICK;
        let diff1 = (ball_time - shooting_time).abs();
        let diff2 = (ball_time2 - shooting_time2).abs();
        if diff1 == diff2 {
            std::cmp::Ordering::Equal
        } else if diff1 < diff2 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL SHOOTABLE DURATION: {:?}", start.elapsed());
        println!("#############################");
    }

    // TODO let's return the ball time and/or shooting time too here?
    let shootable_ball_index = estimates[0].0;
    simple_desired_contact(&ball_trajectory[shootable_ball_index], &desired_ball_position)
}

fn shoot<H: HeuristicModel>(model: &mut H, game: &GameState, bot: &mut BotState) -> PlanResult {
    let desired_ball_position: Vector3<f32> = opponent_goal_shoot_at(&game);
    let last_plan = None;
    // FIXME check if last plan is still valid before using this
    // let last_plan
    //     if bot.last_action.is_some() && bot.last_action.as_ref().unwrap() == &Action::Shoot {
    //         bot.plan.as_ref()
    //     } else {
    //         None
    //     };
    let result = hit_ball(model, &game, &desired_ball_position, last_plan);
    bot.last_action = Some(Action::Shoot);
    result
}

// 1. binary search the ball trajectory
// 2. for each search pivot point, determine a set of car states (position/velocity) that collide
//    with the ball in such a way as to cause the ball to head towards the desired ball position
// 3. based on that desired player state (shooting_player_state), we now need to determine the
//    time by which we can arrive at that point. this is a guestimate, maybe we can use very
//    coarse a* for this if it's fast enough, or just a version of the heuristic_cost function
//    that isn't so admissible (ie more realistic/average timing).
// 4. compare that to the time in the ball trajectory
// 5. if we arrived earlier than the ball, we know we can hit it at an earlier point in it's
//    trajectory. if we arrived later, then we hope we can hit it on time at a later point in
//    the trajctory
// 6. boom we found the earliest point at which it's possible to hit the ball into its desired
//    position, and got the corresponding desired player state all at once.
// 7. plan motion to reach desired player state
fn hit_ball<H: HeuristicModel>(
    model: &mut H,
    game: &GameState,
    desired_ball_position: &Vector3<f32>,
    last_plan: Option<&Plan>,
) -> PlanResult {
    //let start = Instant::now();
    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 10.0);
    //println!("#############################");
    //println!("BALL DURATION: {:?}", start.elapsed());
    //println!("#############################");
    //let start = Instant::now();

    let desired_contact = reachable_desired_player_state(
        model,
        &game.player,
        &ball_trajectory,
        &desired_ball_position,
    );
    let start = Instant::now();
    let result = plan::plan(model, &game.player, &game.ball, &desired_contact, last_plan);
    println!("PLAN DURATION: {:?}", start.elapsed());
    result
}

fn non_admissable_estimated_time<H: HeuristicModel>(
    model: &mut H,
    current: &PlayerState,
    desired: &DesiredContact,
) -> f32 {
    // unreachable, we can't fly
    if desired.position.z > BALL_COLLISION_RADIUS + CAR_DIMENSIONS.z {
        return std::f32::MAX;
    }

    let mut single_heuristic_cost = [0.0];
    model
        .unscaled_heuristic(&[current.clone()], &mut single_heuristic_cost[0..1])
        .expect("Heuristic failed initial!");
    unsafe { *single_heuristic_cost.get_unchecked(0) }
}

// TODO
//fn shadow(game: &GameState) -> PlayerState {
//}

/// main entrypoint for bot to figure out what to do given the current state
// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
pub fn play<H: HeuristicModel>(model: &mut H, game: &GameState, bot: &mut BotState) -> PlanResult {
    let start = Instant::now();
    let plan_result = match what_do(game) {
        Action::Shoot => shoot(model, game, bot),
    };
    let duration = start.elapsed();
    plan_result
}

#[no_mangle]
pub extern "C" fn closest_plan_index(current_player: &PlayerState, plan: &Plan) -> usize {
    assert!(plan.len() != 0);

    let mut index = 0;
    let mut min_distance = std::f32::MAX;

    for (i, (player, _, _)) in plan.iter().enumerate() {
        let distance = (current_player.position - player.position).norm();
        if distance < min_distance {
            min_distance = distance;
            index = i
        }
    }

    index
}

#[no_mangle]
pub extern "C" fn next_input(
    current_player: &PlayerState,
    bot: &mut BotState,
) -> rlbot::ControllerState {
    if let Some(ref plan) = bot.plan {
        let index = closest_plan_index(&current_player, &plan);

        // we need to look one past closest index to see the controller to reach next position
        if index < plan.len() - 1 {
            let current_heading =
                current_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            let (closest_player, _, _) = plan[index];
            let (_next_player, controller, _) = plan[index + 1];

            // FIXME we should account for differences in the tick and interpolate between the two
            // closest indices to get the real closet delta/distance
            let closest_delta = current_player.position - closest_player.position;
            let closest_distance = closest_delta.norm();
            let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0);
            let relative_right = clockwise_90_rotation * current_heading;

            if closest_distance == 0.0 {
                bot.turn_errors.push_back(0.0);
            } else {
                let direction = na::Matrix::dot(
                    &Unit::new_normalize(closest_delta.clone()).into_inner(),
                    &relative_right,
                ); // positive for right, negative for left
                println!("direction: {}, distance: {}", direction, closest_distance);
                let error = direction * closest_distance;
                bot.turn_errors.push_back(error);
            }

            if bot.turn_errors.len() > 1000 {
                // keep last 100
                bot.turn_errors = bot.turn_errors.split_off(900);
            }

            //println!("controller: {:?}", controller);
            let mut input = convert_controller_to_rlbot_input(&controller);
            //println!("input before: {:?}", input);
            pd_adjust(&mut input, &bot.turn_errors);
            //println!("input after: {:?}", input);

            return input;
        }
    }

    // fallback
    let mut input = rlbot::ControllerState::default();
    input.throttle = 0.5;
    if current_player.position.z > 150.0 {
        if (current_player.position.z as i32 % 2) == 0 {
            input.jump = true;
        }
    }
    input
}

const PROPORTIONAL_DIST_GAIN: f32 = 0.004;
const DIFFERENTIAL_GAIN: f32 = 0.35;
const DIFFERENTIAL_STEPS: usize = 2;
fn pd_adjust(input: &mut rlbot::ControllerState, errors: &VecDeque<f32>) {
    // build up some errors before we do anything
    if errors.len() <= DIFFERENTIAL_STEPS {
        return;
    }
    let last_error = errors[errors.len() - 1];
    let error_slope =
        (last_error - errors[errors.len() - 1 - DIFFERENTIAL_STEPS]) / DIFFERENTIAL_STEPS as f32;
    //println!(
    //    "last_error: {:?}, error_slope: {:?}",
    //    last_error, error_slope
    //); // TODO normalize slope to speed!
    let proportional_signal = PROPORTIONAL_DIST_GAIN * last_error;
    let differential_signal = DIFFERENTIAL_GAIN * error_slope;
    let signal = proportional_signal + differential_signal;
    //println!(
    //    "signal: {}, p: {}, d: {}",
    //    signal, proportional_signal, differential_signal
    //);
    input.steer += signal;

    if input.steer > 1.0 {
        input.steer = 1.0;
    } else if input.steer < -1.0 {
        input.steer = -1.0;
    }
}

fn convert_controller_to_rlbot_input(controller: &BrickControllerState) -> rlbot::ControllerState {
    rlbot::ControllerState {
        throttle: match controller.throttle {
            Throttle::Idle => 0.0,
            Throttle::Forward => 1.0,
            Throttle::Reverse => -1.0,
        },
        steer: match controller.steer {
            Steer::Straight => 0.0,
            Steer::Left => -1.0,
            Steer::Right => 1.0,
        },
        pitch: controller.pitch,
        yaw: controller.yaw,
        roll: controller.roll,
        jump: controller.jump,
        boost: controller.boost,
        handbrake: controller.handbrake,
    }
}
