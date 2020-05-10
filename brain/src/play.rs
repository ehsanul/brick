use crate::HeuristicModel; // TODO as _;
use na::{self, Rotation3, Unit, Vector3};
use plan;
use predict::{self, player::PredictPlayer};
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

// 1. for each point in the ball trajectory estimate the
//    time by which we can hit the ball into the goal
// 2. compare that to the time in the ball trajectory
// 3. choose the point in trajectory with smallest diff
fn reachable_contact_ball_and_time<H: HeuristicModel>(
    model: &mut H,
    player: &PlayerState,
    ball_trajectory: &[BallState],
    desired_ball_position: &Vector3<f32>,
) -> (DesiredContact, BallState, f32) {
    let mut estimates = ball_trajectory
        .iter()
        .enumerate()
        .map(|(i, ball)| {
            let desired_contact = simple_desired_contact(ball, &desired_ball_position);
            model.configure(&desired_contact, 1.0);
            let shooting_time = non_admissable_estimated_time(model, &player, &ball);
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

    let reachable_ball_index = estimates[0].0;
    let reachable_time = estimates[0].1;
    let desired_contact = simple_desired_contact(&ball_trajectory[reachable_ball_index], &desired_ball_position);

    (desired_contact, ball_trajectory[reachable_ball_index].clone(), reachable_time)
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
    let result = hit_ball(model, game, bot, &desired_ball_position, last_plan);
    bot.last_action = Some(Action::Shoot);
    result
}

fn hit_ball<H: HeuristicModel>(
    model: &mut H,
    game: &GameState,
    bot: &BotState,
    desired_ball_position: &Vector3<f32>,
    last_plan: Option<&Plan>,
) -> PlanResult {
    //let start = Instant::now();
    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 10.0);
    //println!("#############################");
    //println!("BALL DURATION: {:?}", start.elapsed());
    //println!("#############################");
    //let start = Instant::now();

    // FIXME additional lag should be added for brick's planning calculation lag
    let (desired_contact, ball, time) = reachable_contact_ball_and_time(
        model,
        &game.player.lag_compensated_player(&bot.controller_history, LAG_FRAMES),
        &ball_trajectory,
        &desired_ball_position,
    );
    let start = Instant::now();
    // FIXME additional lag should be added for brick's planning calculation lag
    let result = plan::plan(model, &game.player.lag_compensated_player(&bot.controller_history, LAG_FRAMES), &ball, &desired_contact, time, last_plan);
    // println!("PLAN DURATION: {:?}", start.elapsed());
    result
}

fn non_admissable_estimated_time<H: HeuristicModel>(
    model: &mut H,
    current: &PlayerState,
    ball: &BallState,
) -> f32 {
    // unreachable, we can't fly
    if ball.position.z - BALL_COLLISION_RADIUS > CAR_DIMENSIONS.z + CAR_OFFSET.z {
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
pub extern "C" fn closest_plan_index(given_player: &PlayerState, plan: &Plan) -> usize {
    assert!(plan.len() != 0);

    let mut index = 0;
    let mut min_distance = std::f32::MAX;

    for (i, (player, _, _)) in plan.iter().enumerate() {
        let distance = (given_player.position - player.position).norm();
        if distance < min_distance {
            min_distance = distance;
            index = i
        }
    }

    index
}

#[no_mangle]
pub extern "C" fn next_input(
    player: &PlayerState,
    bot: &mut BotState,
) -> rlbot::ControllerState {
    if let Some(ref plan) = bot.plan {
        // we need to take into account the inputs previously sent that will be processed
        // prior to finding where we are. instead of passing the current player, apply
        // LAG_FRAMES inputs that are not yet applied
        let player = player.lag_compensated_player(&bot.controller_history, LAG_FRAMES);
        let index = closest_plan_index(&player, &plan);

        // we need to look one past closest index to see the controller to reach next position
        if index < plan.len() - 1 {
            let current_heading =
                player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            let (closest_player, _, _) = plan[index];
            let (_next_player, controller, _) = plan[index + 1];
            //println!("index: {}, controller.steer: {:?}", index, controller.steer);

            // FIXME we should account for differences in the tick and interpolate between the two
            // closest indices to get the real closet delta/distance
            let closest_delta = player.position - closest_player.position;
            let closest_distance = closest_delta.norm();
            let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0);
            let relative_right = clockwise_90_rotation * current_heading;

            if closest_distance == 0.0 {
                bot.turn_errors.push_back(0.0);
            } else {
                let projection = na::Matrix::dot(
                    &Unit::new_normalize(closest_delta.clone()).into_inner(),
                    &relative_right,
                ); // positive for right, negative for left
                //println!("projection: {}, distance: {}", projection, closest_distance);
                let error = projection * closest_distance;
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
    input.throttle = 1.0;
    if player.position.z > 150.0 {
        if (player.position.z as i32 % 2) == 0 {
            input.jump = true;
        }
    }
    input
}

const THROTTLE_FACTOR: f32 = 0.2;
const PROPORTIONAL_DIST_GAIN: f32 = 0.004;
const DIFFERENTIAL_GAIN: f32 = 0.35;
const DIFFERENTIAL_STEPS: usize = 4; // NOTE 2 also works ok it seems
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
        let diff = (input.steer - 1.0).min(1.0 / THROTTLE_FACTOR);
        input.steer = 1.0;
        // slow down if we want to go in a tighter circle than we already are going
        input.throttle -= THROTTLE_FACTOR * diff;
    } else if input.steer < -1.0 {
        let diff = (-(input.steer + 1.0)).min(1.0 / THROTTLE_FACTOR);
        input.steer = -1.0;
        // slow down if we want to go in a tighter circle than we already are going
        input.throttle -= THROTTLE_FACTOR * diff;
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
