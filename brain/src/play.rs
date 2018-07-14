use state::*;
use plan;
use predict;
use na::{ Vector3, UnitQuaternion };
use std::f32::consts::PI;
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

// TODO
//fn shoot(game: &GameState) -> Option<PlayerState> {
//    let desired_ball_position: Vector3<f32> = opponent_goal_shoot_at(&game);
//    let ball_trajectory = predict::ball::ball_trajectory(&game.ball, 5.0);
//    let mut shooting_player_state = PlayerState::default();
//    let shootable_ball_state = ball_trajectory.binary_search_by(|ball| {
//        shooting_player_state = shooting_player_states(ball, desired_ball_position).find(|shooting_player| {
//        });
//        shooting_player_state.time // hmm, let's use enumerate and use the index here for time? would be nice to just store the time maybe in samples instead...
//    });
//
//    match shootable_ball_state  {
//        Some(_ball_state) => Ok(shooting_player_state ),
//        None => Err(()),
//    }
//
//    plan::plan(&game.player, &game.ball, DesiredState {
//        player: None(),
//        ball: shootable_ball_state
//    })
//}

// TODO
//fn shadow(game: &GameState) -> PlayerState {
//}

fn go_to_mid(game: &GameState) -> BrickControllerState {
    let controller = plan::plan(&game.player, &game.ball, &DesiredState {
        player: Some(PlayerState {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0),
            team: Team::Blue, // TODO let's separate stuff like this out, Player vs PlayerState maybe
        }),
        ball: None,
    });

    controller.unwrap_or_else(|| {
        let mut fallback = BrickControllerState::new();
        fallback.throttle = Throttle::Forward;
        fallback
    })
}

/// main entrypoint for bot to figure out what to do given the current state
// TODO we need to also include our current (ie previously used) strategy state as an input here,
// and logic for expiring it if it's no longer applicable.
#[no_mangle]
pub extern fn play(game: &GameState) -> BrickControllerState {
    match what_do(&game) {
        Action::GoToMid => go_to_mid(&game),
        //Action::Shoot => shoot(),
    }
}
