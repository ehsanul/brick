use na::{Vector3, Translation3, UnitQuaternion, Rotation3};
use state::*;
use std::f32;
use std::f32::consts::{PI, E};


enum PredictionCategory {
    Soaring,
    //Rolling,
}

fn find_prediction_category(current: &BallState) -> PredictionCategory {
    // hard-coded the only thing we can handle right now
    PredictionCategory::Soaring
}


#[no_mangle]
pub extern fn next_ball_state(current: &BallState, time_step: f32) -> BallState {
    match find_prediction_category(&current) { // whoops, borrowed it.. new let binding?
        PredictionCategory::Soaring => next_ball_state_soaring(&current, time_step),
        //PredictionCategory::Rolling => next_player_state_rolling(&current, time_step),
    }
}

fn next_ball_state_soaring(current: &BallState, time_step: f32) -> BallState {
    let mut next = (*current).clone();
    next
}
