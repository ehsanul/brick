use state::*;
use plan;
use predict;
use rlbot;
use na::{ self, Unit, Vector3, Rotation3, UnitQuaternion };
use std::f32::consts::PI;
use std;
use predict::arena::{ BACK_WALL_DISTANCE, GOAL_Z };
use std::time::{Duration, Instant};
use std::collections::VecDeque;

enum Action {
    Shoot,
    //Shadow,
    //GoToMid, // XXX not a real action, just a test
}

// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
fn what_do(game: &GameState) -> Action {
    Action::Shoot // TODO
}


fn opponent_goal_shoot_at(game: &GameState) -> Vector3<f32> {
    // TODO calculate which part of the goal is hardest for the opponent to reach
    match game.player.team {
        // FIXME check if we have this right...
        Team::Blue => Vector3::new(0.0, BACK_WALL_DISTANCE, GOAL_Z / 2.0),
        Team::Orange => Vector3::new(0.0, -BACK_WALL_DISTANCE, GOAL_Z / 2.0),
    }
}

/// guess best point on ball to hit, get the heading at that point
#[no_mangle]
pub extern fn simple_desired_contact(ball: &BallState, desired_ball_position: &Vector3<f32>) -> DesiredContact  {
    let desired_vector = Unit::new_normalize(desired_ball_position - ball.position);
    let desired_velocity = 3000.0 * desired_vector.unwrap();
    let velocity_delta = desired_velocity - ball.velocity;

    // this is pretty crude, doesn't even consider that the ball will undergo gravity after the
    // hit! but should be good enough for us here for now
    let impulse_direction = Unit::new_normalize(velocity_delta);
    let ball_normal = -1.0 * impulse_direction.unwrap();

    DesiredContact {
        position: ball.position + BALL_RADIUS * ball_normal,
        heading: -1.0 * ball_normal,
    }
}

// FIXME this rough step is probably too rough when we're really close,
//       so probably needs to be dynamic based on distance or some such
static ROUGH_STEP: f32 = 60.0/120.0;

fn reachable_desired_player_state(player: &PlayerState, ball_trajectory: &[BallState], desired_ball_position: &Vector3<f32>) -> Option<DesiredContact> {
    let start = Instant::now();


    let mut closest_desired_contact = DesiredContact::new();
    let mut closest_time_diff = std::f32::MAX;

    let mut estimates = ball_trajectory.iter().enumerate().map(|(i, ball)| {
        let ball_time = (i as f32) * predict::TICK;
        let desired_contact = simple_desired_contact(ball, &desired_ball_position);
        let shooting_time = non_admissable_estimated_time(&player, &desired_contact);
        (i, shooting_time)
    }).collect::<Vec<_>>();
    estimates.sort_by(|(i, shooting_time), (i2, shooting_time2)| {
        let ball_time = (*i as f32) * predict::TICK;
        let ball_time2 = (*i2 as f32) * predict::TICK;
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

    //let shootable_ball_state = .binary_search_by(|(i, ball)| {
    //    let ball_time = (*i as f32) * predict::TICK;
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
    let start = Instant::now();

    match shootable_ball_state {
        Some(i) => {
            // TODO don't re-calculate this, store temporarily in a variable instead
            Some(simple_desired_contact(&ball_trajectory[i], &desired_ball_position))
        }
        None => None,
        //Err(i) => {
        //    // we may not find an exact match with time. so we want to allow for some wiggle room
        //    // here and use the closest desired state if it seems reachable
        //    if closest_time_diff < ROUGH_STEP*1.2 {
        //        Some(closest_desired_contact)
        //    } else {
        //        println!("no plan found! traj: {}, closest_time_diff: {}, ROUGH_STEP: {}", ball_trajectory.len(), closest_time_diff, ROUGH_STEP);
        //        None
        //    }
        //}
    }
}

// 1. get desired ball position to shoot at
// 2. binary search the ball trajectory
// 3. for each search pivot point, determine a set of car states (position/velocity) that collide
//    with the ball in such a way as to cause the ball to head towards the desired ball position
// 4. based on that desired player state (shooting_player_state), we now need to determine the
//    time by which we can arrive at that point. this is a guestimate, maybe we can use very
//    coarse a* for this if it's fast enough, or just a version of the heuristic_cost function
//    that isn't so admissible (ie more realistic/average timing).
// 5. compare that to the time in the ball trajectory
// 6. if we arrived earlier than the ball, we know we can hit it at an earlier point in it's
//    trajectory. if we arrived later, then we hope we can hit it on time at a later point in
//    the trajctory
// 7. boom we found the earliest point at which it's possible to hit the ball into its desired
//    position, and got the corresponding desired player state all at once.
// 8. plan motion to reach desired player state
//
// XXX maybe some of this logic should go in plan::plan, by taking a desired ball position. this
// way we can reuse that for passing, goal keeping, shadow defending, etc, etc
fn shoot(game: &GameState) -> PlanResult {
    let desired_ball_position: Vector3<f32> = opponent_goal_shoot_at(&game);
    let start = Instant::now();
    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 10.0);
    //println!("#############################");
    //println!("BALL DURATION: {:?}", start.elapsed());
    //println!("#############################");
    let start = Instant::now();

    // since we binary search the trajectory, it's useful to do that over two slices,
    // depending on whether we have to turn to reach the ball or not. this ensures we
    //  don't think we need to turn to hit a ball that will be behind us in 5 seconds,
    // given it's coming towards us and right in front already.
    let current_heading = game.player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let transition_index = ball_trajectory.iter().position(|ball| {
        let towards_ball = Unit::new_normalize(ball.position - game.player.position);
        na::dot(&current_heading, &towards_ball) < 0.65 // FIXME tune
    });
    let trajectory_in_front: &[BallState];
    let trajectory_behind: &[BallState];
    if let Some(transition_index) = transition_index {
        //println!("len: {}, transition: {}", ball_trajectory.len(), transition_index);
        let (first, last) = ball_trajectory.split_at(transition_index);
        trajectory_in_front = first;
        trajectory_behind = last;
    } else {
        //println!("no transition");
        trajectory_in_front = &ball_trajectory;
        trajectory_behind = &[];
    }

    let desired_contact = match reachable_desired_player_state(&game.player, &trajectory_in_front, &desired_ball_position) {
        Some(dc) => dc,
        None => {
            match reachable_desired_player_state(&game.player, &trajectory_behind, &desired_ball_position) {
                Some(dc) => dc,
                None => {
                    let fake_desired = DesiredContact::new();
                    return PlanResult { plan: None, desired: fake_desired, visualization_lines: vec![], visualization_points: vec![] };
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
    let x = plan::plan(&game.player, &game.ball, &desired_contact);
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("PLAN DURATION: {:?}", start.elapsed());
        println!("#############################");
    }
    let start = Instant::now();
    x
}

fn go_near_ball(game: &GameState) -> PlanResult {
    let mut desired = DesiredContact::new();
    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 1.0);
    let ball_in_one_sec = ball_trajectory[ball_trajectory.len() - 1];
    //let current_heading = game.player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    desired.position.x = ball_in_one_sec.position.x;
    desired.position.y = ball_in_one_sec.position.y;
    desired.heading = Unit::new_normalize(ball_in_one_sec.position - game.player.position).unwrap();
    plan::plan(&game.player, &game.ball, &desired)
}

fn non_admissable_estimated_time(current: &PlayerState, desired: &DesiredContact) -> f32 {
    // unreachable, we can't fly
    if desired.position.z > BALL_RADIUS + CAR_DIMENSIONS.z {
        return std::f32::MAX;
    }

    let towards_contact = desired.position - current.position;
    let distance = towards_contact.norm();
    let mut estimated_movement_time = distance / 800.0; // TODO TUNE

    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let towards_contact_heading = Unit::new_normalize(towards_contact).unwrap();

    let pointed_right_way_value = na::dot(&current_heading, &towards_contact_heading);
    let needs_additional_turning_value = na::dot(&towards_contact_heading, &Unit::new_normalize(desired.heading.clone()).unwrap());

    if pointed_right_way_value < 0.0 {
        estimated_movement_time += 1.5; // TODO TUNE
    }
    if needs_additional_turning_value  < 0.0 {
        estimated_movement_time += 1.5; // TODO TUNE
    }
    estimated_movement_time

    // let PlanResult { plan, .. } = plan::hybrid_a_star(&current, &desired, ROUGH_STEP);
    // if let Some(plan) = plan {
    //     ROUGH_STEP * ((plan.len() - 1) as f32) // first item is current position, doesn't count
    // } else {
    //     std::f32::MAX
    // }
}

// TODO
//fn shadow(game: &GameState) -> PlayerState {
//}

fn go_to_mid(game: &GameState) -> PlanResult {
    plan::plan(&game.player, &game.ball, &DesiredContact::new())
}

/// main entrypoint for bot to figure out what to do given the current state
// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
#[no_mangle]
pub extern fn play(game: &GameState, bot: &BotState) -> PlanResult {
    let start = Instant::now();
    let mut x = match what_do(&game) {
        //Action::GoToMid => go_to_mid(&game),
        Action::Shoot => shoot(&game),
    };
    let duration = start.elapsed();
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL DURATION: {:?}", duration);
        println!("#############################");
    }

    // fallback when we don't know how to shoot it
    if x.plan.is_none() {
        //println!("FALLBACK");
        //x = go_near_ball(&game);
    }

    x

    // TODO fallback value when we don't know what to do
    //plan_result.plan.unwrap_or_else(|| {
    //    let mut fallback = BrickControllerState::new();
    //    fallback.throttle = Throttle::Forward;
    //    fallback
    //})
}

#[no_mangle]
pub extern fn closest_plan_index(current_player: &PlayerState, plan: &Plan) -> usize {
    let mut iter = plan.iter();
    let mut last_distance = std::f32::MAX;
    let mut index = 0;
    while let Some((player, controller)) = iter.next() {
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
pub extern fn next_input(current_player: &PlayerState, bot: &mut BotState) -> rlbot::PlayerInput {
    // TODO DRY with closest_plan_index function
    if let Some(ref plan) = bot.plan {
        let index = closest_plan_index(&current_player, &plan);

        // we need to look one past closest index to see the controller to reach next position
        if index < plan.len() - 1 {
            let current_heading = current_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
            let (closest_player, _) = plan[index];
            let (next_player, controller) = plan[index + 1];

            // FIXME we should account for differences in the tick and interpolate between the two
            // closest indices to get the real closet delta/distance
            let closest_delta = current_player.position - closest_player.position;
            let closest_distance = closest_delta.norm();
            let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, PI/2.0);
            let relative_right = clockwise_90_rotation * current_heading;
            let direction = na::dot(&closest_delta, &relative_right); // positive for right, negative for left
            let error = direction * closest_distance;

            bot.turn_errors.push_back(error);
            if bot.turn_errors.len() > 1000 {
                // keep last 100
                bot.turn_errors = bot.turn_errors.split_off(900);
            }

            //println!("controller: {:?}", controller);
            let mut input = convert_controller_to_rlbot_input(&controller);
            //println!("input before: {:?}", input);
            //pd_adjust(&mut input, &bot.turn_errors);
            //println!("input after: {:?}", input);


            return input;
        }
    }

    // fallback
    let mut input = rlbot::PlayerInput::default();
    input.Throttle = 0.5;
    if current_player.position.z > 150.0 {
        if current_player.velocity.z > 200.0 {
            input.Throttle = -1.0;
        } else if (current_player.position.z as i32 % 2) == 0 {
            input.Jump = true;
        }
    }
    input.Throttle = 0.5;
    input
}

const PROPORTIONAL_GAIN: f32 = 0.005;
const DIFFERENTIAL_GAIN: f32 = 0.002;
const DIFFERENTIAL_STEPS: usize = 4;
fn pd_adjust(input: &mut rlbot::PlayerInput, errors: &VecDeque<f32>) {
    // build up some errors before we do anything
    if errors.len() <= DIFFERENTIAL_STEPS { return; }
    let last_error = errors[errors.len() - 1];
    let error_slope = (last_error - errors[errors.len() - 1 - DIFFERENTIAL_STEPS]) / DIFFERENTIAL_STEPS as f32;
    println!("last_error: {:?}, error_slope: {:?}", last_error, error_slope);
    let proportional_signal = PROPORTIONAL_GAIN * last_error;
    let differential_signal = DIFFERENTIAL_GAIN * error_slope;
    let signal = proportional_signal + differential_signal;
    println!("signal: {}, p: {}, d: {}", signal, proportional_signal, differential_signal);
    input.Steer += signal;

    if input.Steer > 1.0 {
        if input.Steer > 2.0 {
            println!("super right");
            //input.Handbrake = true;
        }

        input.Steer = 1.0;
    }

    if input.Steer < -1.0 {
        if input.Steer < -2.0 {
            println!("super left");
            //input.Handbrake = true;
        }
        input.Steer = -1.0;
    }
}

fn convert_controller_to_rlbot_input(controller: &BrickControllerState) -> rlbot::PlayerInput {
    rlbot::PlayerInput {
        Throttle: match controller.throttle {
            Throttle::Idle => 0.0,
            Throttle::Forward => 1.0,
            Throttle::Reverse => -1.0,
        },
        Steer: match controller.steer {
            Steer::Straight => 0.0,
            Steer::Left => -1.0,
            Steer::Right => 1.0,
        },
        Pitch: 0.0, // brick is a brick
        Yaw: 0.0, // brick is a brick
        Roll: 0.0, // brick is a brick
        Jump: false, // brick is a brick
        Boost: false, // brick is a brick
        Handbrake: false, // brick is a brick
    }
}

