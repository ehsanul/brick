extern crate nalgebra as na;
extern crate state;

use na::{Vector3, Translation3, UnitQuaternion, Rotation3};
use state::*;
use std::f32;
use std::f32::consts::{PI, E};

enum PredictionCategory {
    /// Wheels on ground
    Ground,
    /* TODO
    /// Top/Sides on ground
    Ground2,
    /// Wheels on wall
    Wall,
    /// Wheels on ceiling
    Ceiling,
    /// Wheels on curve. might want to expand this into top/bottom/corner/etc curves
    CurveWall,
    /// Wheels not touching arena
    Air,
    */
}

fn find_prediction_category(current: &PlayerState) -> PredictionCategory {
    // hard-coded the only thing we can handle right now
    PredictionCategory::Ground
}

/// for now, doesn't handle landing sideways or at any angle really, nor drifting. collision with
/// arena is also not handled. collisions with other players or ball will never be handled here
fn next_player_info_grounded(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    let mut next = (*current).clone();
    let mut next_speed;
    let distance;
    let current_speed = current.velocity.norm(); // current speed (norm is magnitude)

    // probably make it a method on our player info
    // TODO look at code from main.rs with this stuff to confirm it's right
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);

    let max_throttle_speed = 1545.0; // FIXME get exact known value from graph, make it a static?

    match controller.throttle {
        Throttle::Forward | Throttle::Reverse => {
            // the acceleration factor is different if you are turning
            let k = match controller.steer {
                Steer::Straight => 1575.0,
                Steer::Right | Steer::Left => 1200.0, // TODO get this and confirm velocity curve while steering
                _ => panic!("Steering values other than 1.0, -1.0 and 0.0 are not supported!")
            };

            let t1 = f32::ln(k) - f32::ln(k - current_speed); // confirm this, forgot
            let t2 = t1 + time_step;
            let t_intercept = 3.294; // FIXME get exact known value from graph. could also calculate: ln(1575) - ln(1575 - max_speed)
            // FIXME doesn't handle braking, nor accelerating forwards when moving backwards
            if current_speed > 1544.0 { // FIXME reference constant, find exact variance from RL
                distance = (t2 - t1) * max_throttle_speed;
            } else if t2 > t_intercept {
                // we are hitting max speed, so have to calculate distance in two sections for the two
                // different curves
                let d1 = k*t_intercept + k*E.powf(-t_intercept) - k*t1 - k*E.powf(-t1);
                let d2 = (t2 - t_intercept) * max_throttle_speed;
                distance = d1 + d2;
            } else {
                distance = k*t2 + k*E.powf(-t2) - k*t1 - k*E.powf(-t1);
            }

            next_speed = k * (1.0 - E.powf(-t2));
        },
        Throttle::Rest => {
            let k = 100.0; // deceleration constant FIXME get actual value from graph and make it a static/constant

            if current_speed <= k * time_step {
                next_speed = 0.0;
            } else {
                next_speed = current_speed - k * time_step;
            }
            distance = (current_speed + next_speed) * time_step / 2.0;
        },
    }

    // next position/rotation
    match controller.steer {
        Steer::Straight => {
            // straight line is easy
            let mut translation = current_heading * distance;

            // FIXME this will not work for braking
            if controller.throttle == Throttle::Reverse {
                translation *= -1.0;
            }

            next.position = current.position + translation;
            next.rotation = current.rotation.clone(); // if we clone from current, this becomes no-op
        }
        Steer::Right | Steer::Left => {
            unimplemented!()
            // let (translation, rotation) = circle_prediction(current.velocity, controller.steer, time_step);
            // next.position = current.position + translation;
            // next.rotation = current.rotation * rotation;
        }
        _ => panic!("Steering values other than 1.0, -1.0 and 0.0 are not supported!")
    }

    let next_heading = next.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);

    match controller.throttle {
        Throttle::Forward | Throttle::Reverse => {
            if next_speed > max_throttle_speed {
                next_speed = max_throttle_speed;
            }
            println!("next_speed: {}, value: {}", next_speed, controller.throttle.value());
            next.velocity = next_heading * next_speed * controller.throttle.value();
        },
        Throttle::Rest => {
            let k = 100.0; // FIXME get actual value from graph
            next.velocity = next_heading * next_speed;
        },
    }

    next
}


#[no_mangle]
pub extern fn predict_test() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

#[no_mangle]
pub extern fn next_player_state(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    match find_prediction_category(&current) { // whoops, borrowed it.. new let binding?
        PredictionCategory::Ground => next_player_info_grounded(&current, &controller, time_step),
        //PredictionCategory::Ground2 => next_velocity_grounded2(&current, &controller, time_step),
        //PredictionCategory::Wall => next_velocity_walled(&current, &controller, time_step),
        //PredictionCategory::Ceiling => next_velocity_ceilinged(&current, &controller, time_step),
        //PredictionCategory::CurveWall => next_velocity_curve_walled(&current, &controller, time_step),
        //PredictionCategory::Air => next_velocity_flying(&current, &controller, time_step),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resting_position() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_rotation() -> UnitQuaternion<f32> { UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0) }

    fn resting_player_info() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            rotation: resting_rotation(),
        }
    }

    fn max_throttle_velocity() -> Vector3<f32> { Vector3::new(0.0, 1545.0, 0.0) } // FIXME reference constant/static

    fn max_throttle_player_info() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: max_throttle_velocity(),
            rotation: resting_rotation(),
        }
    }

    fn round(v: Vector3<f32>) -> Vector3<f32> {
        Vector3::new(v.x.round(), v.y.round(), v.z.round())
    }

    #[test]
    fn no_input_from_resting() {
        let current = resting_player_info();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, current.position);
        assert_eq!(next.velocity, current.velocity);
        assert_eq!(next.rotation, current.rotation);
    }

    #[test]
    fn throttle_from_resting() {
        let mut current = resting_player_info();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, 579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, 995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting() {
        let mut current = resting_player_info();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, -579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, -995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_backwards() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, -579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, -995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting_backwards() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, 579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, 995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q1_angle() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI*3.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(410.0, 410.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(704.0, 704.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q2_angle() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI*3.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(410.0, -410.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(704.0, -704.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q3_angle() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI*1.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-410.0, -410.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(-704.0, -704.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q4_angle() {
        let mut current = resting_player_info();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI*1.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-410.0, 410.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(-704.0, 704.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }


    #[test]
    fn throttle_at_max_throttle() {
        let mut current = max_throttle_player_info();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1545.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1545.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_at_max_throttle() {
        let mut current = max_throttle_player_info();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1495.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1445.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    // TODO need to graph/model this first
    // #[test]
    // fn reverse_at_max_throttle() {
    //     let mut current = max_throttle_player_info();
    //     let mut controller = BrickControllerState::new();
    //     controller.throttle = Throttle::Reverse;
    //     let next = next_player_state(&current, &controller, 1.0);

    //     assert_eq!(next.position, Vector3::new(0.0, 579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
    //     assert_eq!(next.rotation, current.rotation);
    //     assert_eq!(next.velocity, Vector3::new(0.0, 995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    // }



}
