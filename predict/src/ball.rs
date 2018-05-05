use na::{Vector3, Translation3, UnitQuaternion, Rotation3};
use state::*;
use std::f32;
use std::f32::consts::{PI, E};

static SIDE_WALL_DISTANCE: f32 = 4096.0;
static BACK_WALL_DISTANCE: f32 = 5140.0;
static CEILING_DISTANCE: f32 = 2044.0;
static GOAL_X: f32 = 892.75;
static GOAL_Z: f32 = 640.0;

// static BALL_RADIUS: f32 = 93.143 is imported from state, was: R = 91.25
static RESTITUION: f32 = 0.6; // was: C_R = 0.6

// TODO
// was: Y = 2.0
// was: mu = 0.285
// was: A = 0.0003

static GRAVITY: f32 = 650.0; // uu/s2
static AIR_RESISTANCE: f32 = 0.0305; // % loss per second
static BALL_MAX_SPEED: f32 = 6000.0;
static BALL_MAX_ROTATION_SPEED: f32 = 6.0;

static TICK: f32 = 1.0 / 120.0; // matches RL's internal fixed physics tick rate

enum PredictionCategory {
    Soaring,
    //Rolling,
}

fn find_prediction_category(current: &BallState) -> PredictionCategory {
    // hard-coded the only thing we can handle right now
    PredictionCategory::Soaring
}


#[no_mangle]
pub extern fn ball_trajectory(current: &BallState, duration: f32) -> Vec<BallState> {
    let mut t = 0.0;
    let mut trajectory = Vec::with_capacity((duration / TICK).ceil() as usize);
    while t < duration {
        trajectory.push(next_ball_state_dt(&current, TICK));
        t += TICK;
    }
    trajectory
}


fn next_ball_state_dt(current: &BallState, time_step: f32) -> BallState {
    match find_prediction_category(&current) {
        PredictionCategory::Soaring => next_ball_state_soaring_dt(&current, time_step),
        //PredictionCategory::Rolling => next_ball_state_rolling_dt(&current, time_step),
    }
}

fn next_ball_state_soaring_dt(current: &BallState, time_step: f32) -> BallState {
    let mut next = (*current).clone();

    if arena_will_collide(current) {
        let bounced = calculate_bounce(&current);
        next.position = bounced.position;
        next.velocity = bounced.velocity;
    }

    // TODO gravity, air resistance and all that

    next
}

/// true if the arena is going to be in collision with the ball in the next tick, and isn't already
/// bouncing away having collided
fn arena_will_collide(current: &BallState) -> bool {
    false // FIXME
}

fn calculate_bounce(current: &BallState) -> BallState {
    // FIXME
    let mut bounced = (*current).clone();
    bounced
}
