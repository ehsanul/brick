use state::*;
use plan;
use predict;
use na::{ self, Unit, Vector3, Rotation3, UnitQuaternion };
use std::f32::consts::PI;
use std;
use predict::arena::{ BACK_WALL_DISTANCE, GOAL_Z };


enum Action {
    //Shoot,
    //Shadow,
    GoToMid, // XXX not a real action, just a test
}

// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
fn what_do(game: &GameState) -> Action {
    Action::GoToMid // TODO
}


fn opponent_goal_shoot_at(game: &GameState) -> Vector3<f32> {
    // TODO calculate which part of the goal is hardest for the opponent to reach
    match game.player.team {
        // FIXME check if we have this right...
        Team::Blue => Vector3::new(0.0, BACK_WALL_DISTANCE, GOAL_Z / 2.0),
        Team::Orange => Vector3::new(0.0, -BACK_WALL_DISTANCE, GOAL_Z / 2.0),
    }
}


/// simpler version of shooting_player_state when we just need a basic guess
fn simple_shooting_player_state(ball: &BallState, desired_ball_position: &Vector3<f32>) -> PlayerState {
    let mut shooting_player = PlayerState::default();

    let desired_vector = Unit::new_normalize(desired_ball_position - ball.position);
    let desired_velocity = 1000.0 * desired_vector.unwrap();
    let velocity_delta = desired_velocity - ball.velocity;

    // this is pretty crude, doesn't even consider that the ball will undergo gravity after the
    // hit! but should be good enough for us here for now
    let impulse_direction = Unit::new_normalize(velocity_delta);
    let ball_normal = -1.0 * impulse_direction.unwrap();

    shooting_player.position = ball.position + (BALL_RADIUS + CAR_DIMENSIONS.x/2.0) * ball_normal;

    shooting_player.position = ball.position + (BALL_RADIUS + CAR_DIMENSIONS.x/2.0) * ball_normal;

    //let rotation = impulse_direction * na::inverse(&Unit::new_unchecked(Vector3::new(-1.0, 0.0, 0.0)));
    //shooting_player.rotation = UnitQuaternion::from_rotation_matrix(&rotation);
    // https://math.stackexchange.com/a/476311
    let initial = Vector3::new(-1.0, 0.0, 0.0);
    let v: Vector3<f32> = initial.cross(&impulse_direction);
    let vx = v.cross_matrix();
    let c = na::dot(&initial, &impulse_direction);
    let identity = Rotation3::identity();
    let rotation = Rotation3::from_matrix_unchecked(identity.matrix() + vx + (vx * vx) * (1.0 / (1.0 / c)));
    shooting_player.rotation = UnitQuaternion::from_rotation_matrix(&rotation);

    shooting_player
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
    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 5.0);

    let mut shooting_player = PlayerState::default();
    let mut shooting_time = std::f32::MAX;
    let shootable_ball_state = ball_trajectory.iter().enumerate().collect::<Vec<_>>().binary_search_by(|(i, ball)| {
        let ball_time = (*i as f32) / predict::TICK;

        shooting_player = simple_shooting_player_state(ball, &desired_ball_position);
        shooting_time = non_admissable_estimated_time(&game.player, &shooting_player);
        if shooting_time == ball_time {
            std::cmp::Ordering::Equal
        } else if shooting_time < ball_time {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let desired_state = match shootable_ball_state {
        Ok(i) => {
            DesiredState {
                player: Some(shooting_player),
                ball: None,
            }
        },
        Err(i) => {
            // we may not find an exact match with time. so we want to allow for some wiggle room
            // here and use the closet desired state
            let ball_time = (i as f32) / predict::TICK;
            if (ball_time - shooting_time).abs() < 0.05 {
                DesiredState {
                    player: Some(shooting_player),
                    ball: None,
                }
            } else {
                return PlanResult { plan: None, visualization_lines: vec![] };
            }
        },
    };

    // TODO if we move the above logic to `plan::plan`, we can maybe call it this way:
    //plan::plan(&game.player, &game.ball, &DesiredState {
    //    player: None,
    //    ball: shooting_player
    //})
    plan::plan(&game.player, &game.ball, &desired_state)
}

fn non_admissable_estimated_time(current: &PlayerState, desired: &PlayerState) -> f32 {
    unimplemented!();
}

// TODO
//fn shadow(game: &GameState) -> PlayerState {
//}

fn go_to_mid(game: &GameState) -> PlanResult {
    plan::plan(&game.player, &game.ball, &DesiredState {
        player: Some(PlayerState {
            position: Vector3::new(-3000.0, 2000.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0),
            team: Team::Blue, // TODO let's separate stuff like this out, Player vs PlayerState maybe
        }),
        ball: None,
    })
}

/// main entrypoint for bot to figure out what to do given the current state
// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
#[no_mangle]
pub extern fn play(game: &GameState) -> PlanResult {
    match what_do(&game) {
        Action::GoToMid => go_to_mid(&game),
        //Action::Shoot => shoot(),
    }

    // TODO fallback value when we don't know what to do
    //plan_result.plan.unwrap_or_else(|| {
    //    let mut fallback = BrickControllerState::new();
    //    fallback.throttle = Throttle::Forward;
    //    fallback
    //})
}
