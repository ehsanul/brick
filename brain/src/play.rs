use heuristic::HeuristicModel; // TODO as _;
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
        position: ball.position + BALL_RADIUS * ball_normal,
        heading: -1.0 * ball_normal,
    }
}

fn reachable_desired_player_state<H: HeuristicModel>(
    model: &mut H,
    player: &PlayerState,
    ball_trajectory: &[BallState],
    desired_ball_position: &Vector3<f32>,
) -> Option<DesiredContact> {
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
        if diff1 == diff1 {
            std::cmp::Ordering::Equal
        } else if diff1 < diff2 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    let shootable_ball_state = estimates.iter().map(|(i, _)| *i).next(); // next == first

    //let closest_desired_contact = DesiredContact::default();
    //let mut closest_time_diff = std::f32::MAX;
    //let shootable_ball_state = .binary_search_by(|(i, ball)| {
    //    let ball_time = (*i as f32) * TICK;
    //    let start2 = Instant::now();
    //    let desired_contact = simple_desired_contact(ball, &desired_ball_position);
    //    let shooting_time = non_admissable_estimated_time(&player, &desired_contact);
    //    //println!("#############################");
    //    //println!("SINGLE SHOOTABLE DURATION: {:?}", start2.elapsed());
    //    println!("ball: {:?}", ball);
    //    println!("desired_ball_position: {:?}", desired_ball_position);
    //    println!("contact position: {:?}", desired_contact.position);
    //    println!("shooting time: {}", shooting_time);
    //    println!("ball time: {}", ball_time);
    //    println!("----------------------");

    //    let time_diff = (ball_time - shooting_time).abs();
    //    if time_diff < closest_time_diff {
    //        closest_time_diff = time_diff;
    //        closest_desired_contact = desired_contact;
    //    }
    //    if shooting_time == ball_time {
    //        std::cmp::Ordering::Equal
    //    } else if shooting_time < ball_time {
    //        std::cmp::Ordering::Greater
    //    } else {
    //        std::cmp::Ordering::Less
    //    }
    //});
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL SHOOTABLE DURATION: {:?}", start.elapsed());
        println!("#############################");
    }

    match shootable_ball_state {
        Some(i) => {
            // TODO don't re-calculate this, store temporarily in a variable instead
            Some(simple_desired_contact(
                &ball_trajectory[i],
                &desired_ball_position,
            ))
        }
        None => None,
    }
}

fn shoot<H: HeuristicModel>(model: &mut H, game: &GameState, bot: &mut BotState) -> PlanResult {
    let desired_ball_position: Vector3<f32> = opponent_goal_shoot_at(&game);
    let last_plan =
        if bot.last_action.is_some() && bot.last_action.as_ref().unwrap() == &Action::Shoot {
            bot.plan.as_ref()
        } else {
            None
        };
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

    // since we binary search the trajectory, it's useful to do that over two slices,
    // depending on whether we have to turn to reach the ball or not. this ensures we
    // don't think we need to turn to hit a ball that will be behind us in 5 seconds,
    // given it's coming towards us and right in front already.
    let current_heading = game.player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let mut last_dot = None;
    let transition_index = ball_trajectory.iter().position(|ball| {
        let towards_ball = Unit::new_normalize(ball.position - game.player.position);
        let dot = na::Matrix::dot(&current_heading, &towards_ball);
        if let Some(last_dot_value) = last_dot {
            last_dot_value * dot < 0.0 // if only one is negative, we've transitioned
        } else {
            last_dot = Some(dot);
            false
        }
    });
    let trajectory_segment1: &[BallState];
    let trajectory_segment2: &[BallState];
    if let Some(transition_index) = transition_index {
        //println!("len: {}, transition: {}", ball_trajectory.len(), transition_index);
        let (first, last) = ball_trajectory.split_at(transition_index);
        trajectory_segment1 = first;
        trajectory_segment2 = last;
    } else {
        //println!("no transition");
        trajectory_segment1 = &ball_trajectory;
        trajectory_segment2 = &[];
    }

    let desired_contact = match reachable_desired_player_state(
        model,
        &game.player,
        &trajectory_segment1,
        &desired_ball_position,
    ) {
        Some(dc) => dc,
        None => {
            match reachable_desired_player_state(
                model,
                &game.player,
                &trajectory_segment2,
                &desired_ball_position,
            ) {
                Some(dc) => dc,
                None => {
                    let fake_desired = DesiredContact::default();
                    return PlanResult {
                        plan: None,
                        desired: fake_desired,
                        visualization_lines: vec![],
                        visualization_points: vec![],
                    };
                }
            }
        }
    };

    // TODO if we move the above logic to `plan::plan`, we can maybe call it this way:
    //plan::plan(&game.player, &game.ball, &DesiredState {
    //    player: None,
    //    ball: shooting_player
    //})
    let start = Instant::now();
    let x = plan::plan(model, &game.player, &desired_contact, last_plan);
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("PLAN DURATION: {:?}", start.elapsed());
        println!("#############################");
    }
    x
}

fn non_admissable_estimated_time<H: HeuristicModel>(
    model: &mut H,
    current: &PlayerState,
    desired: &DesiredContact,
) -> f32 {
    // unreachable, we can't fly
    if desired.position.z > BALL_RADIUS + CAR_DIMENSIONS.z {
        return std::f32::MAX;
    }

    let mut single_heuristic_cost = [0.0];
    model
        .heuristic(&[current.clone()], &mut single_heuristic_cost[0..1])
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
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL DURATION: {:?}", duration);
        println!("#############################");
    }

    plan_result
}

#[no_mangle]
pub extern "C" fn closest_plan_index(current_player: &PlayerState, plan: &Plan) -> usize {
    let mut iter = plan.iter();
    let mut last_distance = std::f32::MAX;
    let mut index = 0;
    assert!(plan.len() != 0);
    while let Some((player, _controller, _step_duration)) = iter.next() {
        let delta = current_player.position - player.position;
        let distance = delta.norm();

        if distance > last_distance {
            // we iterate and choose the controller at the point distance increases. this
            // is because `controller` is the previous controller input to reach the given
            // player.position. NOTE this logic is only good if we provide a "exploded"
            // plan, ie we have a position for every tick, and also only if we cut parts of
            // the path off as we pass them
            break;
        }
        last_distance = distance;
        index += 1;
    }

    index - 1
}

#[no_mangle]
pub extern "C" fn next_input(
    current_player: &PlayerState,
    bot: &mut BotState,
) -> rlbot::ControllerState {
    // TODO DRY with closest_plan_index function
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
    println!(
        "last_error: {:?}, error_slope: {:?}",
        last_error, error_slope
    ); // TODO normalize slope to speed!
    let proportional_signal = PROPORTIONAL_DIST_GAIN * last_error;
    let differential_signal = DIFFERENTIAL_GAIN * error_slope;
    let signal = proportional_signal + differential_signal;
    println!(
        "signal: {}, p: {}, d: {}",
        signal, proportional_signal, differential_signal
    );
    input.steer += signal;

    if input.steer > 1.0 {
        if input.steer > 2.0 {
            println!("super right");
            //input.handbrake = true;
        }

        input.steer = 1.0;
    }

    if input.steer < -1.0 {
        if input.steer < -2.0 {
            println!("super left");
            //input.handbrake = true;
        }
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
        pitch: 0.0,       // brick is a brick
        yaw: 0.0,         // brick is a brick
        roll: 0.0,        // brick is a brick
        jump: false,      // brick is a brick
        boost: false,     // brick is a brick
        handbrake: false, // brick is a brick
    }
}
