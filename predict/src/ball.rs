use std::f32;
use std::error::Error;

use na::{self, Isometry3, Unit, Vector3};
use ncollide;

use arena::ARENA;
use state::*;

const RESTITUTION: f32 = 0.6; // was: C_R = 0.6

const Y: f32 = 2.0; // parallel bounce friction constant
const MU: f32 = 0.285; // parallel bounce friction constant
const A: f32 = 0.0003; // friction -> angular velocity factor

const GRAVITY: f32 = 650.0; // uu/s2
const SLIDING_SPEED: f32 = 565.0; // uu/s
const SLIDING_DECELERATION: f32 = 230.0; // uu/s2
const AIR_RESISTANCE: f32 = 0.0305; // % loss per second
const ROLLING_RESISTANCE: f32 = 0.022; // % loss per second
const BALL_MAX_SPEED: f32 = 6000.0; // uu/s
const BALL_MAX_ROTATION_SPEED: f32 = 6.0; // rad/s

const BALL_MASS: f32 = 30.0;
const BALL_INERTIA: f32 = 0.4 * BALL_MASS * BALL_INERTIAL_RADIUS * BALL_INERTIAL_RADIUS;

enum PredictionCategory {
    Soaring,
    Rolling,
}

fn find_prediction_category(ball: &BallState) -> PredictionCategory {
    // NOTE using the "soaring" calculations when interacting with wall/curves, even when rolling,
    // since we at least have arena collision for that and will get a better prediction even if not
    // 100% accurate
    let in_air = ball.position.z > BALL_COLLISION_RADIUS || ball.velocity.z.abs() < 1.0;
    let on_side_curve = ball.position.x.abs() > SIDE_CURVE_DISTANCE;
    let on_back_curve = ball.position.y.abs() > BACK_CURVE_DISTANCE;
    if in_air || on_side_curve || on_back_curve {
        PredictionCategory::Soaring
    } else {
        PredictionCategory::Rolling
    }
}

#[no_mangle]
pub extern "C" fn ball_trajectory(ball: &BallState, duration: f32) -> Vec<BallState> {
    let mut t = 0.0;
    let mut trajectory = Vec::with_capacity((duration / TICK).ceil() as usize);
    let mut ball_now = ball.clone();
    trajectory.push(ball_now);
    while t < duration {
        t += TICK;
        ball_now = next_ball_state(trajectory.last().unwrap(), TICK);
        trajectory.push(ball_now);
    }
    trajectory
}

pub fn trajectory_enters_soccar_goal(ball: &BallState) -> bool {
    // FIXME check full ball trajectory, with bouncing
    //let goal = Vector3::new(0.0, BACK_WALL_DISTANCE, GOAL_Z / 2.0);
    let goal = na::Vector2::new(0.0, BACK_WALL_DISTANCE);
    let pos = na::Vector2::new(ball.position.x, ball.position.y);
    let v = na::Vector2::new(ball.velocity.x, ball.velocity.y);

    // check if 2d ball velocty is towards goal
    // TODO use exact angle of left side and right side of goal from current ball position instead
    // of the dot product approximate check
    (goal - pos).normalize().dot(&v.normalize()) > 0.95
}

pub fn next_ball_state(ball: &BallState, time_step: f32) -> BallState {
    match find_prediction_category(&ball) {
        PredictionCategory::Soaring => next_ball_state_soaring(&ball, time_step),
        PredictionCategory::Rolling => next_ball_state_rolling(&ball, time_step),
    }
}

fn next_ball_state_soaring(ball: &BallState, time_step: f32) -> BallState {
    let mut next;

    if let Some(normal) = arena_contact_normal(&ball) {
        if na::Matrix::dot(&ball.velocity, &normal) < 0.0 {
            // we're going towards the arena contact, so let's bounce
            next = calculate_bounce(&ball, &normal);
        } else {
            // already bounced
            next = (*ball).clone();
        }
    } else {
        next = (*ball).clone();
    }

    // FIXME rl utils uses prev velocity instead of next velocity, we should too?
    let acceleration = Vector3::new(0.0, 0.0, -GRAVITY) - AIR_RESISTANCE * next.velocity;

    if next.velocity.norm() > BALL_MAX_SPEED {
        next.velocity = next.velocity.normalize() * BALL_MAX_SPEED;
    }

    if next.angular_velocity.norm() > BALL_MAX_ROTATION_SPEED {
        next.angular_velocity = next.angular_velocity.normalize() * BALL_MAX_ROTATION_SPEED;
    }

    next.position += time_step * next.velocity;
    next.velocity += time_step * acceleration;

    next
}

// source: https://www.youtube.com/watch?v=9uh8-nBlufM
fn next_ball_state_rolling(ball: &BallState, time_step: f32) -> BallState {
    let mut next = ball.clone();

    let acceleration;
    if ball.velocity.norm() > SLIDING_SPEED {
        let heading = ball.velocity / ball.velocity.norm();
        acceleration = -SLIDING_DECELERATION * heading - AIR_RESISTANCE * next.velocity;
    } else {
        acceleration = -ROLLING_RESISTANCE * next.velocity
    }

    next.position += time_step * next.velocity;
    next.velocity += time_step * acceleration;

    // hard-coding certain values
    next.velocity.z = 0.0;
    next.position.z = BALL_COLLISION_RADIUS;

    next
}

/// returns normal at contact point if ball is currently colliding with the arena
#[no_mangle]
pub extern "C" fn arena_contact_normal(ball: &BallState) -> Option<Unit<Vector3<f32>>> {
    let sphere = ncollide::shape::Ball::new(BALL_COLLISION_RADIUS);
    let ball_pos = Isometry3::new(ball.position.clone(), na::zero()); // TODO if we want to handle cube ball, track and pass on the rotation
    let arena_pos = Isometry3::new(na::zero(), na::zero());

    let margin = 0.0;
    let contact = ncollide::query::contact(&arena_pos, &(*ARENA), &ball_pos, &sphere, margin);

    contact.map(|c| c.normal)
}

// FIXME lot more going in the rl utils now, probably helps handle more types of bounces:
// https://github.com/samuelpmish/RLUtilities/blob/master/src/simulation/ball.cc#L36
fn calculate_bounce(ball: &BallState, normal: &Unit<Vector3<f32>>) -> BallState {
    let mut bounced = (*ball).clone();

    let v_perp = na::Matrix::dot(&ball.velocity, &normal.into_inner()) * normal.into_inner();
    let v_para = ball.velocity - v_perp;
    let v_spin = BALL_COLLISION_RADIUS * normal.cross(&ball.angular_velocity); // velocity of edge of ball, relative to ball center
    let s = v_para + v_spin; // this is the velocity at point of impact (edge of ball) in global coords

    let ratio = v_perp.norm() / s.norm();

    let delta_v_perp = -(1.0 + RESTITUTION) * v_perp;
    let delta_v_para = -f32::min(1.0, Y * ratio) * MU * s;

    bounced.velocity += delta_v_perp + delta_v_para;
    bounced.angular_velocity += A * BALL_COLLISION_RADIUS * delta_v_para.cross(&normal);

    bounced
}

// source: https://github.com/samuelpmish/RLUtilities/blob/master/src/simulation/ball.cc#L90
fn psyonix_scale_impulse(val: f32) -> f32 {
    if val <= 500.0 {
        0.65
    } else if val <= 2300.0 {
        0.65 - 0.1 * (val - 500.0) / (2300.0 - 500.0)
    } else {
        0.55 - 0.25 * (val - 2300.0) / (4600.0 - 2300.0)
    }
}

/// calculates ball state after collision with player. caller of this function must ensure there
/// actually *is* a collision between the ball and player, otherwise the result of this is
/// unpredictable
pub fn calculate_hit(ball: &BallState, player: &PlayerState, collision: &Vector3<f32>) -> Result<BallState, Box<dyn Error>> {
    let n1 = (collision - ball.position).normalize();

    let L_b = (collision - ball.position).cross_matrix();
    let L_c = (collision - player.position).cross_matrix();

    let player_rotation = player.rotation.to_rotation_matrix();
    let invI_c = player_rotation * (*CAR_INVERSE_INERTIA * player_rotation.transpose());

    let invM = ((1.0 / BALL_MASS) + (1.0 / CAR_MASS)) * na::Matrix3::identity() - ((L_b * L_b) / BALL_INERTIA) - (L_c * (invI_c * L_c));
    let M = invM.try_inverse().ok_or("M matrix inversion failed")?;

    let delta_v = (player.velocity - (L_c * player.angular_velocity)) - (ball.velocity - (L_b * ball.angular_velocity));

    // compute the impulse that is consistent with an inelastic collision
    let J1 = M * delta_v;

    let J1_perp = J1.dot(&n1).min(-1.0f32) * n1;
    let J1_para = J1 - J1_perp;

    let ratio = J1_perp.norm() / 0.001f32.max(J1_para.norm());

    // scale the parallel component of J1 such that the
    // Coulomb friction model is satisfied
    let J1 = J1_perp + 1.0f32.min(MU * ratio) * J1_para;

    let heading: Vector3<f32> = player.heading();
    let mut n2: Vector3<f32> = ball.position - player.position;
    n2.z *= 0.35;
    let n2 = (n2 - (0.35 * (n2.dot(&heading))) * heading).normalize();

    let dv = 4600.0f32.min((ball.velocity - player.velocity).norm());
    let J2 = BALL_MASS * dv * psyonix_scale_impulse(dv) * n2;

    let mut hit_ball = ball.clone();
    hit_ball.angular_velocity += (L_b * J1) / BALL_INERTIA;
    hit_ball.velocity += (J1 + J2) / BALL_MASS;

    Ok(hit_ball)
}
