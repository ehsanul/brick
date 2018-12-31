use na::{self, Vector3, UnitQuaternion, Rotation3};
use state::*;
use sample;
use std::f32;

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

    let (translation, velocity, angular_velocity, rotation) = ground_turn_prediction(&current, &controller, time_step);

    next.position = current.position + translation;
    next.position.z = RESTING_Z; // avoid drifting upward/downward when we're just driving on the ground!
    next.velocity = velocity;
    next.angular_velocity = angular_velocity;
    next.rotation = UnitQuaternion::from_rotation_matrix(&rotation); // was easier to just return the end rotation directly. TODO stop using quaternion

    next
}


fn ground_turn_matching_samples(current: &PlayerState, controller: &BrickControllerState, time_step: f32, ceil: bool) -> (&'static PlayerState, &'static PlayerState) {
    // based on current player state, and steer, throttle and boost, gets the right samples
    let samples: &'static [PlayerState] = sample::get_relevant_turn_samples(&current, &controller, ceil);

    let start_index = 0;
    // TODO use the time steps in the file
    let end_index = start_index + (time_step * sample::RECORD_FPS as f32).round() as usize;

    let sample_start_state: &PlayerState = samples.get(start_index).expect(&format!("ground_turn_prediction start_index missing: {}, player: {:?}, controller: {:?}", start_index, current, controller));
    let sample_end_state: &PlayerState = samples.get(end_index).expect(&format!("ground_turn_prediction end_index missing: {}, player: {:?}, controller: {:?}", end_index, current, controller));

    (sample_start_state, sample_end_state)
}

/// factor: number from 0.0 to 1.0 for interpolation between start and end, 0.0 being 100% at
/// start, 1.0 being 100% at end.
fn interpolate(start: Vector3<f32>, end: Vector3<f32>, factor: f32) -> Vector3<f32> {
    (1.0 - factor) * start + factor * end
}

/// returns tuple of (translation, acceleration, angular_acceleration, rotation)
fn ground_turn_prediction(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, Rotation3<f32>) {
    let current_speed = current.velocity.norm();
    let (sample1_start, sample1_end) = ground_turn_matching_samples(&current, &controller, time_step, false);

    let (sample2_start, sample2_end) = if current_speed >= 2300.0 { // TODO MAX_BOOST_SPEED
        (sample1_start, sample1_end)
    } else {
        ground_turn_matching_samples(&current, &controller, time_step, true)
    };
    let speed1 = sample1_start.velocity.norm();
    let speed2 = sample2_start.velocity.norm();
    let speed_diff1 = current_speed - speed1;
    let speed_diff2 = speed2 - current_speed;
    let (closer_sample_start, closer_sample_end) = if speed_diff1 < speed_diff2 {
        (sample1_start, sample1_end)
    } else {
        (sample2_start, sample2_end)
    };

    let sample_speed_diff = speed2 - speed1;
    let mut factor = if sample_speed_diff == 0.0 {
        0.0
    } else {
        (current_speed - speed1) / sample_speed_diff
    };
    assert!(factor < 1.1);
    assert!(factor > -0.1);

    // TODO use Rotation3 instead of UnitQuaternion for player.rotation
    // get rotation that when multiplied with closer_sample_start.rotation, gives us current_rotation
    // normalization_rotation . sample_start.rotation = current_rotation
    let normalization_rotation1 = current.rotation.to_rotation_matrix() * na::inverse(&sample1_start.rotation.to_rotation_matrix());
    let normalization_rotation2 = current.rotation.to_rotation_matrix() * na::inverse(&sample2_start.rotation.to_rotation_matrix());
    let closer_normalization_rotation = current.rotation.to_rotation_matrix() * na::inverse(&closer_sample_start.rotation.to_rotation_matrix());

    // relative position is translation. same for velocity -> acceleration
    let translation1 = normalization_rotation1 * (sample1_end.position - sample1_start.position);
    let translation2 = normalization_rotation2 * (sample2_end.position - sample2_start.position);
    let translation = interpolate(translation1, translation2, factor);

    let end_velocity1 = normalization_rotation1 * sample1_end.velocity;
    let end_velocity2 = normalization_rotation2 * sample2_end.velocity;
    let end_velocity = interpolate(end_velocity1, end_velocity2, factor);

    (
        translation,
        end_velocity,
        closer_sample_end.angular_velocity,
        closer_normalization_rotation * closer_sample_end.rotation.to_rotation_matrix(),
    )
}


#[no_mangle]
pub extern fn predict_test() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

#[no_mangle]
pub extern fn next_player_state(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    let mut next_player = match find_prediction_category(&current) {
        PredictionCategory::Ground => next_player_state_grounded(&current, &controller, time_step),
        //PredictionCategory::Ground2 => next_velocity_grounded2(&current, &controller, time_step),
        //PredictionCategory::Wall => next_velocity_walled(&current, &controller, time_step),
        //PredictionCategory::Ceiling => next_velocity_ceilinged(&current, &controller, time_step),
        //PredictionCategory::CurveWall => next_velocity_curve_walled(&current, &controller, time_step),
        //PredictionCategory::Air => next_velocity_flying(&current, &controller, time_step),
    };

    if next_player.position.z < CAR_DIMENSIONS.z / 2.0 {
        next_player.position.z = CAR_DIMENSIONS.z / 2.0;
    }

    next_player
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn resting_position() -> Vector3<f32> { Vector3::new(0.0, 0.0, RESTING_Z) }
    fn resting_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_angular_velocity() -> Vector3<f32> { Vector3::new(0.0, 0.0, 0.0) }
    fn resting_rotation() -> UnitQuaternion<f32> { UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0) }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            angular_velocity: resting_angular_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn max_throttle_velocity() -> Vector3<f32> { Vector3::new(0.0, 1545.0, 0.0) } // FIXME reference constant/static

    fn max_throttle_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: max_throttle_velocity(),
            angular_velocity: resting_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn round(v: Vector3<f32>) -> Vector3<f32> {
        Vector3::new(v.x.round(), v.y.round(), v.z.round())
    }

    fn round_rotation(r: UnitQuaternion<f32>) -> (f32, f32, f32) {
        let (roll, pitch, yaw) = r.to_euler_angles();
        const factor: f32 = 50.0;
        (
            (roll * factor).round() / factor,
            (pitch * factor).round() / factor,
            (yaw * factor).round() / factor,
        )
    }

    #[test]
    fn no_input_from_resting() {
        let current = resting_player_state();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(next.position, current.position);
        assert_eq!(round(next.velocity), current.velocity);
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
    }

    #[test]
    fn throttle_from_resting_forward() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, 590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1006.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, -590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, -1006.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_backwards() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, -590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, -1006.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting_backwards() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, 590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1006.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q1_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI*3.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(417.0, 417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(712.0, 712.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q2_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI*3.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(417.0, -417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(712.0, -712.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q3_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI*1.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-417.0, -417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(-712.0, -712.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q4_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI*1.0/4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-417.0, 417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(-712.0, 712.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }


    #[test]
    fn throttle_at_max_throttle() {
        let mut current = max_throttle_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1545.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1545.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_at_max_throttle() {
        let mut current = max_throttle_player_state();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1495.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, 1445.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_roll_to_a_stop() {
        let mut current = resting_player_state();
        current.velocity.y = 50.0;
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 12.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
        assert_eq!(round(next.velocity), Vector3::new(0.0, 0.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    }


    // TODO need to graph/model this first
    // #[test]
    // fn reverse_at_max_throttle() {
    //     let mut current = max_throttle_player_state();
    //     let mut controller = BrickControllerState::new();
    //     controller.throttle = Throttle::Reverse;
    //     let next = next_player_state(&current, &controller, 1.0);

    //     assert_eq!(next.position, Vector3::new(0.0, 590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
    //     assert_eq!(round_rotation(next.rotation), round_rotation(current.rotation));
    //     assert_eq!(round(next.velocity), Vector3::new(0.0, 1006.0, RESTING_Z_VELOCITY)); // FIXME confirm if this value is actually correct in graph
    // }


    #[test]
    fn throttle_and_turn_from_resting() {
        let mut current = resting_player_state();

        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        controller.steer = Steer::Right;

        // from data file, first line
        current.position = Vector3::new(-2501.8398, -3171.19, 18.65);
        current.velocity = Vector3::new(0.0, 0.0, 8.32);
        current.rotation = UnitQuaternion::from_euler_angles(0.0, -0.0059441756, -1.5382951);

        let next = next_player_state(&current, &controller, 1.0);

        // from data file, 240th line
        let expected_position = Vector3::new(-2063.0, -2951.0, 18.65);
        let expected_velocity = Vector3::new(866.1724, -262.35745, 8.33);
        let expected_rotation = UnitQuaternion::from_euler_angles(0.0, -0.0059441756, 2.832112);

        assert_eq!(round(next.position), round(expected_position));
        assert_eq!(round_rotation(next.rotation), round_rotation(expected_rotation));
        assert_eq!(round(next.velocity), round(expected_velocity));
    }

    fn throttle_and_turn_from_max_throttle_turning() {
        let mut current = resting_player_state();

        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        controller.steer = Steer::Left;

        // from data file, 1000th line
        // -482.672,-2684.1472,18.65,-263.5451,-1204.4678,8.33,-0.00009587344,-0.005944175,1.3977439
        current.position = Vector3::new(-482.672, -2684.1472, 18.65);
        current.velocity = Vector3::new(-263.5451, -1204.4678, 8.33);
        current.rotation = UnitQuaternion::from_euler_angles(-0.00009587344, -0.005944175, 1.3977439);

        let next = next_player_state(&current, &controller, 1.0);

        // from data file, 1240th line
        // 287.21667,-3266.7107,18.65,1081.4581,602.2005,8.33,0.00000000011641738,-0.0059441756,-2.5908935
        let expected_position = Vector3::new(287.21667, -3266.7107, 18.65);
        let expected_velocity = Vector3::new(1081.4581, 602.2005, 8.33);
        let expected_rotation = UnitQuaternion::from_euler_angles(0.00000000011641738, -0.0059441756, -2.5908935);

        assert_eq!(round(next.position), round(expected_position));
        assert_eq!(round_rotation(next.rotation), round_rotation(expected_rotation));
        assert_eq!(round(next.velocity), round(expected_velocity));
    }

}
