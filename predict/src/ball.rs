use std::f32;

use na::{self, Isometry3, Vector3, Unit};
use ncollide;

use state::*;
use arena::ARENA;


// static BALL_RADIUS: f32 = 93.143 is imported from state, was: R = 91.25
static RESTITUION: f32 = 0.6; // was: C_R = 0.6

static Y: f32 = 2.0; // parallel bounce friction constant
static MU: f32 = 0.285; // parallel bounce friction constant
static A: f32 = 0.0003; // friction -> angular velocity factor

static GRAVITY: f32 = 650.0; // uu/s2
static AIR_RESISTANCE: f32 = 0.0305; // % loss per second
static BALL_MAX_SPEED: f32 = 6000.0; // uu/s
static BALL_MAX_ROTATION_SPEED: f32 = 6.0; // rad/s

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
    let mut ball_now = current.clone();
    while t < duration {
        trajectory.push(ball_now);
        t += TICK;
        ball_now = next_ball_state_dt(trajectory.last().unwrap(), TICK);
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
    let mut next;

    if let Some(normal) = arena_contact_normal(&current) {
        if na::dot(&current.velocity, &normal) < 0.0 {
            // we're going towards the arena contact, so let's bounce
            next = calculate_bounce(&current, &normal);
        } else {
            // already bounced
            next = (*current).clone();
        }
    } else {
        next = (*current).clone();
    }

    let acceleration = Vector3::new(0.0, 0.0, -GRAVITY) - AIR_RESISTANCE * next.velocity;

    if next.velocity.norm() > BALL_MAX_SPEED {
        // TODO is there a better to_unit_vector method or something?
        next.velocity = (next.velocity / next.velocity.norm()) * BALL_MAX_SPEED;
    }

    if next.angular_velocity.norm() > BALL_MAX_ROTATION_SPEED {
        // TODO is there a better to_unit_vector method or something?
        next.angular_velocity = (next.angular_velocity / next.angular_velocity.norm()) * BALL_MAX_ROTATION_SPEED;
    }

    next.position += time_step * next.velocity;
    next.velocity += time_step * acceleration;

    next
}


/// returns normal at contact point if ball is currently colliding with the arena
#[no_mangle]
pub extern fn arena_contact_normal(current: &BallState) -> Option<Unit<Vector3<f32>>> {
    let ball = ncollide::shape::Ball::new(BALL_RADIUS);
    let ball_pos = Isometry3::new(current.position.clone(), na::zero()); // TODO if we want to handle cube ball, track and pass on the rotation
    let arena_pos = Isometry3::new(na::zero(), na::zero());

    let margin = 0.0;
    let contact = ncollide::query::contact(&arena_pos, &(*ARENA), &ball_pos, &ball, margin);

    contact.map(|c| c.normal)
}

fn calculate_bounce(current: &BallState, normal: &Unit<Vector3<f32>>) -> BallState {
    let mut bounced = (*current).clone();

    let v_perp = na::dot(&current.velocity, &normal.unwrap()) * normal.unwrap();
    let v_para = current.velocity - v_perp;
    let v_spin = BALL_RADIUS * normal.cross(&current.angular_velocity); // velocity of edge of ball, relative to ball center
    let s = v_para + v_spin; // this is the velocity at point of impact (edge of ball) in global coords

    let ratio = v_perp.norm() / s.norm();

    let delta_v_perp = - (1.0 + RESTITUION) * v_perp;
    let delta_v_para = - f32::min(1.0, Y * ratio) * MU * s;

    bounced.velocity += delta_v_perp + delta_v_para;
    bounced.angular_velocity += A * BALL_RADIUS * delta_v_para.cross(&normal);

    bounced
}
