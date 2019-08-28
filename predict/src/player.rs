use na::{self, Rotation3, UnitQuaternion, Vector3};
use sample;
use state::*;
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

pub fn find_prediction_category(_current: &PlayerState) -> PredictionCategory {
    // hard-coded the only thing we can handle right now
    PredictionCategory::Ground
}

/// for now, doesn't handle landing sideways or at any angle really, nor drifting. collision with
/// arena is also not handled. collisions with other players or ball will never be handled here
fn next_player_state_grounded(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> PlayerState {
    let mut next = (*current).clone();

    let (translation, velocity, angular_velocity, rotation) =
        ground_turn_prediction(&current, &controller, time_step);

    // because we extrapolate around the edges of our measurements, it's possible we calculate
    // a velocity beyond what's possible in the game. so we must scale it down here.
    let scale = if velocity.norm() > MAX_BOOST_SPEED {
        MAX_BOOST_SPEED / velocity.norm()
    } else {
        1.0
    };

    // XXX NOTE using the velocity scaling for the translation isn't correct at all, but it's
    // likely to be at least proportional, and likely closer to correct than not scaling at all.
    next.position = current.position + scale * translation;
    next.position.z = RESTING_Z; // avoid drifting upward/downward when we're just driving on the ground!
    next.velocity = scale * velocity;
    next.angular_velocity = angular_velocity;
    next.rotation = UnitQuaternion::from_rotation_matrix(&rotation); // was easier to just return the end rotation directly. TODO stop using quaternion

    next
}

fn ground_turn_matching_samples(
    normalized: &sample::NormalizedPlayerState,
    controller: &BrickControllerState,
    time_step: f32,
    xrange: i16,
    yrange: i16,
    skipx: Option<i16>,
    skipy: Option<i16>,
) -> Option<(&'static PlayerState, &'static PlayerState)> {
    // based on current player state, and steer, throttle and boost, gets the right samples, with
    // some wiggle room based on xrange/yrange
    let mut local_normalized = normalized.clone();
    let mut samples: Option<&'static [PlayerState]> = None;

    // step_by is not yet stabilized... so using plain loops instead
    let ystep = if yrange < 0 { -1 } else { 1 };
    let xstep = if xrange < 0 { -1 } else { 1 };
    let mut dy = 0;
    'outer: loop {
        if skipy != Some(dy) {
            let mut dx = 0;
            loop {
                if skipx != Some(dx) {
                    local_normalized.local_vy = normalized.local_vy + dy;
                    local_normalized.local_vx = normalized.local_vx + dx;
                    //println!("local_normalized: {:?}", local_normalized);

                    samples = sample::get_relevant_turn_samples(&local_normalized, &controller);
                    if samples.is_some() { break 'outer }
                }

                dx += xstep;
                if dx.abs() > xrange.abs() { break }
            }
        }

        dy += ystep;
        if dy.abs() > yrange.abs() { break }
    }

    if samples.is_none() {
        // TODO log if none?
        //let samples = samples.expect(&format!(
        //    "Missing turn sample for player: {:?} & controller: {:?} & xrange {} & yrange {}",
        //    normalized, controller, xrange, yrange
        //));
        return None
    }

    let samples = samples.unwrap();

    let start_index = 0;
    let end_index = start_index + (time_step * sample::RECORD_FPS as f32).round() as usize;

    let sample_start_state: &PlayerState = samples.get(start_index).expect(&format!(
        "ground_turn_prediction start_index missing: {}, normalized player: {:?}, controller: {:?}",
        start_index, normalized, controller
    ));

    let sample_end_state: &PlayerState = samples.get(end_index).expect(&format!(
        "ground_turn_prediction end_index missing: {}, normalized player: {:?}, controller: {:?}",
        end_index, normalized, controller
    ));

    Some((sample_start_state, sample_end_state))
}

fn ground_turn_surrounding_quad(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> [Option<(&'static PlayerState, &'static PlayerState)>; 4] {
    // TODO handle missing values properly: go lower/higher to find another point to use as an
    // interpolation anchor
    let normalized = sample::normalized_player(&current, false, false);
    let mut x1y1 = ground_turn_matching_samples(&normalized, &controller, time_step, -3, -3, None, None);

    let normalized = sample::normalized_player(&current, true, false);
    let mut x2y1 = ground_turn_matching_samples(&normalized, &controller, time_step, 3, -3, None, None);

    // when we fail in on direction, search in the other
    if x1y1.is_some() && x2y1.is_none() {
        //println!("-- x2y1 fallback --");
        let x1y1_player = x1y1.as_ref().unwrap().0;
        let normalized = sample::normalized_player_rounded(x1y1_player);
        x2y1 = ground_turn_matching_samples(&normalized, &controller, time_step, -3, -3, Some(normalized.local_vy), Some(normalized.local_vx));
    } else if x2y1.is_some() && x1y1.is_none() {
        //println!("-- x1y1 fallback --");
        let x2y1_player = x2y1.as_ref().unwrap().0;
        let normalized = sample::normalized_player_rounded(x2y1_player);
        x1y1 = ground_turn_matching_samples(&normalized, &controller, time_step, 3, -3, Some(normalized.local_vy), Some(normalized.local_vx));
    } else if x2y1.is_none() && x1y1.is_none() {
        //println!("-- BOTH FAILED --");
    }

    let normalized = sample::normalized_player(&current, false, true);
    let mut x1y2 = ground_turn_matching_samples(&normalized, &controller, time_step, -3, 3, None, None);

    let normalized = sample::normalized_player(&current, true, true);
    let mut x2y2 = ground_turn_matching_samples(&normalized, &controller, time_step, 3, 3, None, None);

    // when we fail in on direction, search in the other
    if x1y2.is_some() && x2y2.is_none() {
        //println!("-- x2y2 fallback --");
        let x1y2_player = x1y2.as_ref().unwrap().0;
        let normalized = sample::normalized_player_rounded(x1y2_player);
        x2y2 = ground_turn_matching_samples(&normalized, &controller, time_step, -3, 3, Some(normalized.local_vy), Some(normalized.local_vx));
    } else if x2y2.is_some() && x1y2.is_none() {
        //println!("-- x1y2 fallback --");
        let x2y2_player = x2y2.as_ref().unwrap().0;
        let normalized = sample::normalized_player_rounded(x2y2_player);
        x1y2 = ground_turn_matching_samples(&normalized, &controller, time_step, 3, 3, Some(normalized.local_vy), Some(normalized.local_vx));
    } else if x2y2.is_none() && x1y2.is_none() {
        //println!("-- BOTH FAILED --");
    }

    [x1y1, x2y1, x1y2, x2y2]
}


/// factor: number from 0.0 to 1.0 for interpolation between start and end, 0.0 being 100% at
/// start, 1.0 being 100% at end. Note that this actually also handles factors outside the 0.0 to
/// 1.0 range, in which case it's a linear extrapolation
fn interpolate(start: Vector3<f32>, end: Vector3<f32>, factor: f32) -> Vector3<f32> {
    (1.0 - factor) * start + factor * end
}

fn interpolate_scalar(start: f32, end: f32, factor: f32) -> f32 {
    (1.0 - factor) * start + factor * end
}

/// returns tuple of (translation, acceleration, angular_acceleration, rotation)
fn ground_turn_prediction(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>, Rotation3<f32>) {
    //println!("-----------------------------");
    //println!("local_velocity: {:?}", current.local_velocity());
    //println!("controller: {:?}", controller);

    let quad = ground_turn_surrounding_quad(current, controller, time_step);

    // TODO error
    let (x1y1_start, x1y1_end)= quad[0].expect(&format!(
        "Missing turn x1y1 for player: {:?} & controller: {:?}",
        current, controller
    ));
    let (x2y1_start, x2y1_end)= quad[0].expect(&format!(
        "Missing turn x1y1 for player: {:?} & controller: {:?}",
        current, controller
    ));
    let (x1y2_start, x1y2_end)= quad[0].expect(&format!(
        "Missing turn x1y1 for player: {:?} & controller: {:?}",
        current, controller
    ));
    let (x2y2_start, x2y2_end)= quad[0].expect(&format!(
        "Missing turn x1y1 for player: {:?} & controller: {:?}",
        current, controller
    ));

    let current_vx = current.local_velocity().x;
    let current_vy = current.local_velocity().y;

    let y1_vx1 = x1y1_start.local_velocity().x;
    let y1_vx2 = x2y1_start.local_velocity().x;

    let y1_vy1 = x1y1_start.local_velocity().y;
    let y1_vy2 = x2y1_start.local_velocity().y;

    let y2_vx1 = x1y2_start.local_velocity().x;
    let y2_vx2 = x2y2_start.local_velocity().x;

    let y2_vy1 = x1y2_start.local_velocity().y;
    let y2_vy2 = x2y2_start.local_velocity().y;

    // for interpolating along vx at y1 end
    let y1_vx_diff = y1_vx2 - y1_vx1;
    let y1_vx_factor = if y1_vx_diff == 0.0 {
        0.0
    } else {
        (current_vx - y1_vx1) / y1_vx_diff
    };

    // for interpolating along vx at y2 end
    let y2_vx_diff = y2_vx2 - y2_vx1;
    let y2_vx_factor = if y2_vx_diff == 0.0 {
        0.0
    } else {
        (current_vx - y2_vx1) / y2_vx_diff
    };

    // for final interpolation along vy
    let y1_vy = interpolate_scalar(y1_vy1, y1_vy2, y1_vx_factor);
    let y2_vy = interpolate_scalar(y2_vy1, y2_vy2, y2_vx_factor);
    let vy_diff = y2_vy - y1_vy;
    let vy_factor = if vy_diff == 0.0 {
        0.0
    } else {
        (current_vy - y1_vy) / vy_diff
    };

    // get rotation that when multiplied with closer_sample_start.rotation, gives us current_rotation
    // normalization_rotation . sample_start.rotation = current_rotation
    let normalization_rotation_x1y1 = current.rotation.to_rotation_matrix()
        * na::inverse(&x1y1_start.rotation.to_rotation_matrix());
    let normalization_rotation_x2y1 = current.rotation.to_rotation_matrix()
        * na::inverse(&x2y1_start.rotation.to_rotation_matrix());
    let normalization_rotation_x1y2 = current.rotation.to_rotation_matrix()
        * na::inverse(&x1y2_start.rotation.to_rotation_matrix());
    let normalization_rotation_x2y2 = current.rotation.to_rotation_matrix()
        * na::inverse(&x2y2_start.rotation.to_rotation_matrix());

    let translation_x1y1 = normalization_rotation_x1y1 * (x1y1_end.position - x1y1_start.position);
    let translation_x2y1 = normalization_rotation_x2y1 * (x2y1_end.position - x2y1_start.position);
    let translation_x1y2 = normalization_rotation_x1y2 * (x1y2_end.position - x1y2_start.position);
    let translation_x2y2 = normalization_rotation_x2y2 * (x2y2_end.position - x2y2_start.position);
    let translation_y1 = interpolate(translation_x1y1, translation_x2y1, y1_vx_factor);
    let translation_y2 = interpolate(translation_x1y2, translation_x2y2, y2_vx_factor);
    let translation = interpolate(translation_y1, translation_y2, vy_factor);

    let end_velocity_x1y1 = normalization_rotation_x1y1 * x1y1_end.velocity;
    let end_velocity_x2y1 = normalization_rotation_x2y1 * x2y1_end.velocity;
    let end_velocity_x1y2 = normalization_rotation_x1y2 * x1y2_end.velocity;
    let end_velocity_x2y2 = normalization_rotation_x2y2 * x2y2_end.velocity;
    let end_velocity_y1 = interpolate(end_velocity_x1y1, end_velocity_x2y1, y1_vx_factor);
    let end_velocity_y2 = interpolate(end_velocity_x1y2, end_velocity_x2y2, y2_vx_factor);
    let end_velocity = interpolate(end_velocity_y1, end_velocity_y2, vy_factor);

    (
        translation,
        end_velocity,
        // assuming they are all pretty similar
        x1y1_end.angular_velocity,
        // TODO interpolate yaw, but have to handle the fact that it's circular
        normalization_rotation_x1y1 * x1y1_end.rotation.to_rotation_matrix(),
    )
}

#[no_mangle]
pub extern "C" fn predict_test() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

#[no_mangle]
pub extern "C" fn next_player_state(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> PlayerState {
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

    fn resting_position() -> Vector3<f32> {
        Vector3::new(0.0, 0.0, RESTING_Z)
    }
    fn resting_velocity() -> Vector3<f32> {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn resting_angular_velocity() -> Vector3<f32> {
        Vector3::new(0.0, 0.0, 0.0)
    }
    fn resting_rotation() -> UnitQuaternion<f32> {
        UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0)
    }

    fn resting_player_state() -> PlayerState {
        PlayerState {
            position: resting_position(),
            velocity: resting_velocity(),
            angular_velocity: resting_angular_velocity(),
            rotation: resting_rotation(),
            team: Team::Blue,
        }
    }

    fn max_throttle_velocity() -> Vector3<f32> {
        Vector3::new(0.0, 1545.0, 0.0)
    } // FIXME reference constant/static

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
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
    }

    #[test]
    fn throttle_from_resting_forward() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, 590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, 1006.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting() {
        let mut current = resting_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, -590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, -1006.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_backwards() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, -590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, -1006.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn reverse_from_resting_backwards() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Reverse;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(0.0, 590.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, 1006.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q1_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI * 3.0 / 4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(417.0, 417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(712.0, 712.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q2_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI * 3.0 / 4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(417.0, -417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(712.0, -712.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q3_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI * 1.0 / 4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-417.0, -417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(-712.0, -712.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_from_resting_q4_angle() {
        let mut current = resting_player_state();
        current.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI * 1.0 / 4.0);
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        assert_eq!(round(next.position), Vector3::new(-417.0, 417.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(-712.0, 712.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn throttle_at_max_throttle() {
        let mut current = max_throttle_player_state();
        let mut controller = BrickControllerState::new();
        controller.throttle = Throttle::Forward;
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1545.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, 1545.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_at_max_throttle() {
        let mut current = max_throttle_player_state();
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 1495.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, 1445.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
    }

    #[test]
    fn no_input_roll_to_a_stop() {
        let mut current = resting_player_state();
        current.velocity.y = 50.0;
        let controller = BrickControllerState::new();
        let next = next_player_state(&current, &controller, 1.0);

        // FIXME reference static/constant
        assert_eq!(round(next.position), Vector3::new(0.0, 12.0, 15.0)); // FIXME confirm if this value is actually correct in graph
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(current.rotation)
        );
        assert_eq!(
            round(next.velocity),
            Vector3::new(0.0, 0.0, RESTING_Z_VELOCITY)
        ); // FIXME confirm if this value is actually correct in graph
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
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(expected_rotation)
        );
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
        current.rotation =
            UnitQuaternion::from_euler_angles(-0.00009587344, -0.005944175, 1.3977439);

        let next = next_player_state(&current, &controller, 1.0);

        // from data file, 1240th line
        // 287.21667,-3266.7107,18.65,1081.4581,602.2005,8.33,0.00000000011641738,-0.0059441756,-2.5908935
        let expected_position = Vector3::new(287.21667, -3266.7107, 18.65);
        let expected_velocity = Vector3::new(1081.4581, 602.2005, 8.33);
        let expected_rotation =
            UnitQuaternion::from_euler_angles(0.00000000011641738, -0.0059441756, -2.5908935);

        assert_eq!(round(next.position), round(expected_position));
        assert_eq!(
            round_rotation(next.rotation),
            round_rotation(expected_rotation)
        );
        assert_eq!(round(next.velocity), round(expected_velocity));
    }

}
