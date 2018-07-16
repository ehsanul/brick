use na::{self, Vector3, Translation3, UnitQuaternion, Rotation3};
use std;
use state::*;
use sample;
use std::f32;
use std::f32::consts::E;

pub static NO_INPUT_DECELERATION: f32 = 100.0; // deceleration constant FIXME get actual value from graph
pub static THROTTLE_ACCELERATION_FACTOR: f32 = 1575.0;
pub static BOOST_ACCELERATION_FACTOR: f32 = 1000.0; // FIXME get actula value from graph
pub static MAX_THROTTLE_SPEED: f32 = 1545.0; // max speed without boost/flipping FIXME get exact known value from graph
pub static MAX_BOOST_SPEED: f32 = 1000.0; // max speed if boosting FIXME get exact known value from graph

pub enum PredictionCategory {
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

pub fn find_prediction_category(current: &PlayerState) -> PredictionCategory {
    // hard-coded the only thing we can handle right now
    PredictionCategory::Ground
}

/// for now, doesn't handle landing sideways or at any angle really, nor drifting. collision with
/// arena is also not handled. collisions with other players or ball will never be handled here
fn next_player_state_grounded(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    let mut next = (*current).clone();
    let mut next_speed;
    let distance;
    let current_speed = current.velocity.norm(); // current speed (norm is magnitude)

    // probably make it a method on our player info
    // TODO look at code from main.rs with this stuff to confirm it's right
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);

    // TODO handle boost
    match controller.throttle {
        Throttle::Forward | Throttle::Reverse => {
            // the acceleration factor is different if you are turning
            let k = match controller.steer {
                Steer::Straight => THROTTLE_ACCELERATION_FACTOR,
                Steer::Right | Steer::Left => 1200.0, // TODO get this and confirm velocity curve while steering
            };

            let t1 = f32::ln(k) - f32::ln(k - current_speed); // confirm this, forgot
            let t2 = t1 + time_step;
            let t_intercept = 3.294; // FIXME get exact known value from graph. could also calculate: ln(1575) - ln(1575 - max_speed)
            // FIXME doesn't handle braking, nor accelerating forwards when moving backwards
            if current_speed > 1544.0 { // FIXME reference constant, find exact variance from RL
                distance = (t2 - t1) * MAX_THROTTLE_SPEED;
            } else if t2 > t_intercept {
                // we are hitting max speed, so have to calculate distance in two sections for the two
                // different curves
                let d1 = k*t_intercept + k*E.powf(-t_intercept) - k*t1 - k*E.powf(-t1);
                let d2 = (t2 - t_intercept) * MAX_THROTTLE_SPEED;
                distance = d1 + d2;
            } else {
                distance = k*t2 + k*E.powf(-t2) - k*t1 - k*E.powf(-t1);
            }

            next_speed = k * (1.0 - E.powf(-t2));
        },
        Throttle::Idle => {
            let speed_delta = NO_INPUT_DECELERATION * time_step;

            if current_speed <= speed_delta {
                next_speed = 0.0;
                let time_to_rest = current_speed / NO_INPUT_DECELERATION;
                distance = current_speed * time_to_rest / 2.0;
            } else {
                next_speed = current_speed - speed_delta;
                distance = (current_speed + next_speed) * time_step / 2.0;
            }
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

            let next_heading = next.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);

            match controller.throttle {
                Throttle::Forward | Throttle::Reverse => {
                    if next_speed > MAX_THROTTLE_SPEED {
                        next_speed = MAX_THROTTLE_SPEED;
                    }
                    next.velocity = next_heading * next_speed * controller.throttle.value();
                },
                Throttle::Idle => {
                    next.velocity = next_heading * next_speed;
                },
            }
        },
        Steer::Right | Steer::Left => {
            let (translation, acceleration, rotation) = ground_turn_prediction(&current, &controller, time_step);
            next.position = current.position + translation;
            next.velocity = current.velocity + acceleration;
            next.rotation = UnitQuaternion::from_rotation_matrix(&rotation); // was easier to just return the end rotation directly. TODO stop using quaternion
        },
    }

    next
}

/// returns tuple of (translation, acceleration, rotation)
// should we return angular acceleration too?
fn ground_turn_prediction(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> (Vector3<f32>, Vector3<f32>, Rotation3<f32>) {
    let translation = Vector3::new(0.0, 0.0, 0.0);
    let rotation = Rotation3::new(Vector3::new(0.0, 0.0, 0.0));

    // based on steer, throttle and boost, gets the right samples
    let samples: &'static Vec<PlayerState> = sample::get_relevant_turn_samples(&controller);

    // find index of closest matching player state
    let current_speed = current.velocity.norm();
    let start_index = samples.binary_search_by(|player_state| {
        let sample_speed = player_state.velocity.norm();
        // `std::cmp::Ord` is not implemented for `f32`
        // that's due to NaN and such. we don't care about that, so just do it.
        if sample_speed == current_speed {
            std::cmp::Ordering::Equal
        } else if sample_speed < current_speed { // uh this might not work when decelerating
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    let start_index = match start_index {
        Ok(i) => i,
        Err(i) => {
            // i is point we can insert into the tree. the value at i is lesser and the next value
            // is greater. we just want the closest one for our purposes
            let candidate1 = samples.get(i).expect(&format!("ground_turn_prediction sample first missing for: {:?} {:?}", current.velocity, controller));
            let candidate2 = samples.get(i + 1).expect(&format!("ground_turn_prediction sample last missing for: {:?} {:?}", current.velocity, controller));
            if candidate1.velocity.norm() - current_speed <= candidate2.velocity.norm() - current_speed {
                i
            } else {
                i + 1
            }
        },
    };

    // NOTE strangely, the physics sample rate for these is 240fps for some reason, not same as the
    // tick normally mentioned at 120fps
    let end_index = start_index + (time_step * 240.0).floor() as usize;

    let sample_start_state: &PlayerState = samples.get(start_index).expect(&format!("ground_turn_prediction start_index missing: {}, {:?}", start_index, controller));
    let sample_end_state: &PlayerState = samples.get(end_index).expect(&format!("ground_turn_prediction end_index missing: {}, {:?}", end_index, controller));

    // TODO use Rotation3 instead of UnitQuaternion for player.rotation
    // get rotation that when multiplied with sample_start_state.rotation, gives us current_rotation
    // normalization_rotation . sample_start_state.rotation = current_rotation
    let normalization_rotation = current.rotation.to_rotation_matrix() * na::inverse(&sample_start_state.rotation.to_rotation_matrix());

    // relative position is translation. same for velocity -> acceleration
    let non_normalized_translation = sample_end_state.position - sample_start_state.position;
    let non_normalized_acceleration = sample_end_state.velocity - sample_start_state.velocity;

    (
        normalization_rotation * non_normalized_translation,
        normalization_rotation * non_normalized_acceleration,
        normalization_rotation * sample_end_state.rotation.to_rotation_matrix(),
    )
}


#[no_mangle]
pub extern fn predict_test() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

#[no_mangle]
pub extern fn next_player_state(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    match find_prediction_category(&current) {
        PredictionCategory::Ground => next_player_state_grounded(&current, &controller, time_step),
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
    use std::f32::consts::PI;

    fn resting_position() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_rotation() -> UnitQuaternion<f32> { UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0) }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn max_throttle_velocity() -> Vector3<f32> { Vector3::new(0.0, 1545.0, 0.0) } // FIXME reference constant/static

    fn max_throttle_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: max_throttle_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn round(v: Vector3<f32>) -> Vector3<f32> {
        Vector3::new(v.x.round(), v.y.round(), v.z.round())
    }

    fn round_rotation(r: UnitQuaternion<f32>) -> UnitQuaternion<f32> {
        let (roll, pitch, yaw) = r.to_euler_angles();
        UnitQuaternion::from_euler_angles(
            (roll * 100.0).round() / 100.0,
            (pitch * 100.0).round() / 100.0,
            (yaw * 100.0).round() / 100.0,
        )
    }

    #[test]
    fn no_input_from_resting() {
        let current = resting_player_state();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, current.position);
        assert_eq!(next.velocity, current.velocity);
        assert_eq!(next.rotation, current.rotation);
    }

    #[test]
    fn throttle_from_resting() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, 579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, 995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, Vector3::new(0.0, -579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(next.velocity, Vector3::new(0.0, -995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_backwards() {
        let mut current = resting_player_state();
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
        let mut current = resting_player_state();
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
        let mut current = resting_player_state();
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
        let mut current = resting_player_state();
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
        let mut current = resting_player_state();
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
        let mut current = resting_player_state();
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
        let mut current = max_throttle_player_state();
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
        let mut current = max_throttle_player_state();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1495.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1445.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_roll_to_a_stop() {
        let mut current = resting_player_state();
        current.velocity.y = 50.0;
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 12.0, 0.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(next.rotation, current.rotation);
        assert_eq!(round(next.velocity), Vector3::new(0.0, 0.0, 0.0)); // FIXME confirm if this value is actually correct in graph
    }


    // TODO need to graph/model this first
    // #[test]
    // fn reverse_at_max_throttle() {
    //     let mut current = max_throttle_player_state();
    //     let mut controller = BrickControllerState::new();
    //     controller.throttle = Throttle::Reverse;
    //     let next = next_player_state(&current, &controller, 1.0);

    //     assert_eq!(next.position, Vector3::new(0.0, 579.4101, 0.0)); // FIXME confirm if this value is actually correct in graph
    //     assert_eq!(next.rotation, current.rotation);
    //     assert_eq!(next.velocity, Vector3::new(0.0, 995.5898, 0.0)); // FIXME confirm if this value is actually correct in graph
    // }


    #[test]
    fn throttle_and_turn_from_resting() {
        let mut current = resting_player_state();

        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        controller.steer = Steer::Right;

        // from data file, first line
        // -2501.8398,-3171.19,18.65,0,0,8.32,0,-0.0059441756,-1.5382951
        current.position = Vector3::new(-2501.8398, -3171.19, 18.65);
        current.velocity = Vector3::new(0.0, 0.0, 8.32);
        current.rotation = UnitQuaternion::from_euler_angles(0.0, -0.0059441756, -1.5382951);

        let next = next_player_state(&current, &controller, 1.0);

        // from data file, 240th line
        // -2087.7048,-2942.8396,18.65,866.1724,-262.35745,8.33,0,-0.0059441756,2.832112
        let expected_position = Vector3::new(-2087.7048, -2942.8396, 18.65);
        let expected_velocity = Vector3::new(866.1724, -262.35745, 8.33);
        let expected_rotation = UnitQuaternion::from_euler_angles(0.0, -0.0059441756, 2.832112);

        assert_eq!(round(next.position), round(expected_position));
        assert_eq!(round_rotation(next.rotation), round_rotation(expected_rotation));
        assert_eq!(round(next.velocity), round(expected_velocity));
    }

}
