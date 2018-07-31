use state::*;
use plan;
use predict;
use na::{ self, Unit, Vector3, Rotation3, UnitQuaternion };
use std::f32::consts::PI;
use std;
use predict::arena::{ BACK_WALL_DISTANCE, GOAL_Z };
use std::time::{Duration, Instant};

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
    let shootable_ball_state = ball_trajectory.iter().enumerate().collect::<Vec<_>>().binary_search_by(|(i, ball)| {
        let ball_time = (*i as f32) * predict::TICK;
        let start2 = Instant::now();
        let desired_contact = simple_desired_contact(ball, &desired_ball_position);
        let shooting_time = non_admissable_estimated_time(&player, &desired_contact);
        //println!("#############################");
        //println!("SINGLE SHOOTABLE DURATION: {:?}", start2.elapsed());
        println!("ball: {:?}", ball);
        println!("desired_ball_position: {:?}", desired_ball_position);
        println!("contact position: {:?}", desired_contact.position);
        println!("shooting time: {}", shooting_time);
        println!("ball time: {}", ball_time);
        println!("----------------------");

        let time_diff = (ball_time - shooting_time).abs();
        if time_diff < closest_time_diff {
            closest_time_diff = time_diff;
            closest_desired_contact = desired_contact;
        }
        if shooting_time == ball_time {
            std::cmp::Ordering::Equal
        } else if shooting_time < ball_time {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL SHOOTABLE DURATION: {:?}", start.elapsed());
        println!("#############################");
    }
    let start = Instant::now();

    match shootable_ball_state {
        Ok(i) => {
            // TODO don't re-calculate this, store temporarily in a variable instead
            Some(simple_desired_contact(&ball_trajectory[i], &desired_ball_position))
        }
        Err(i) => {
            // we may not find an exact match with time. so we want to allow for some wiggle room
            // here and use the closest desired state if it seems reachable
            if closest_time_diff < ROUGH_STEP*1.2 {
                Some(closest_desired_contact)
            } else {
                println!("no plan found! closest_time_diff: {}, ROUGH_STEP: {}", closest_time_diff, ROUGH_STEP);
                None
            }
        }
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
        println!("len: {}, transition: {}", ball_trajectory.len(), transition_index);
        let (first, last) = ball_trajectory.split_at(transition_index);
        trajectory_in_front = first;
        trajectory_behind = last;
    } else {
        println!("no transition");
        trajectory_in_front = &ball_trajectory;
        trajectory_behind = &[];
    }

    let desired_contact = match reachable_desired_player_state(&game.player, &trajectory_in_front, &desired_ball_position) {
        Some(dc) => dc,
        None => {
            // FIXME
            let fake_desired = DesiredContact::new();
            return PlanResult { plan: None, desired: fake_desired, visualization_lines: vec![], visualization_points: vec![] };

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

fn non_admissable_estimated_time(current: &PlayerState, desired: &DesiredContact) -> f32 {
    let PlanResult { plan, .. } = plan::hybrid_a_star(&current, &desired, ROUGH_STEP);
    if let Some(plan) = plan {
        ROUGH_STEP * ((plan.len() - 1) as f32) // first item is current position, doesn't count
    } else {
        std::f32::MAX
    }
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
pub extern fn play(game: &GameState) -> PlanResult {
    let start = Instant::now();
    let x = match what_do(&game) {
        //Action::GoToMid => go_to_mid(&game),
        Action::Shoot => shoot(&game),
    };
    let duration = start.elapsed();
    if start.elapsed().as_secs() >= 1 || start.elapsed().subsec_millis() > 200 {
        println!("#############################");
        println!("TOTAL DURATION: {:?}", duration);
        println!("#############################");
    }

    x

    // TODO fallback value when we don't know what to do
    //plan_result.plan.unwrap_or_else(|| {
    //    let mut fallback = BrickControllerState::new();
    //    fallback.throttle = Throttle::Forward;
    //    fallback
    //})
}
