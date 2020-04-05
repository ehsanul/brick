use na::{Rotation3, UnitQuaternion, Vector3};
use sample;
use driving_model;
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
) -> Result<PlayerState, String> {
    let mut next = (*current).clone();

    let (translation, velocity, angular_velocity, rotation) =
        ground_turn_prediction(&current, &controller, time_step)?;

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

    Ok(next)
}

fn ground_turn_matching_transformation(
    normalized: &sample::NormalizedPlayerState,
    controller: &BrickControllerState,
    time_step: f32,
    xrange: i16,
    yrange: i16,
    skipx: Option<i16>,
    skipy: Option<i16>,
) -> Option<&'static driving_model::PlayerTransformation> {
    // based on current player state, and steer, throttle and boost, gets the right transformation,
    // with some wiggle room based on xrange/yrange
    let mut local_normalized = normalized.clone();
    let mut transformation: Option<&'static driving_model::PlayerTransformation> = None;

    // step_by is not yet stabilized... so using plain loops instead
    let ystep = if yrange < 0 { -1 } else { 1 };
    let xstep = if xrange < 0 { -1 } else { 1 };
    let mut dy = 0;
    'outer: loop {
        let mut dx = 0;
        loop {
            if skipy != Some(dy) || skipx != Some(dx) {
                local_normalized.local_vy = normalized.local_vy + dy;
                local_normalized.local_vx = normalized.local_vx + dx;
                //println!("local_normalized: {:?}", local_normalized);

                transformation = driving_model::get_relevant_transformation(&local_normalized, &controller, time_step);
                if transformation .is_some() { break 'outer }
            }

            dx += xstep;
            if dx.abs() > xrange.abs() { break }
        }

        dy += ystep;
        if dy.abs() > yrange.abs() { break }
    }

    transformation
}

fn ground_turn_quad_tranformations(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> [Option<&'static driving_model::PlayerTransformation>; 4] {
    // TODO handle missing values properly: go lower/higher to find another point to use as an
    // interpolation anchor
    //println!("x1y1");
    let normalized = sample::normalized_player(&current, false, false);
    let mut x1y1 = ground_turn_matching_transformation(&normalized, &controller, time_step, -3, -3, None, None);

    //println!("x2y1");
    let normalized = sample::normalized_player(&current, true, false);
    let mut x2y1 = ground_turn_matching_transformation(&normalized, &controller, time_step, 3, -3, None, None);

    // when we fail in on direction, search in the other
    if x1y1.is_some() && x2y1.is_none() {
        //println!("-- x2y1 fallback --");
        let x1y1_transformation = x1y1.as_ref().unwrap();
        let skip = x1y1_transformation.normalized_player(current.angular_velocity.z);
        x2y1 = ground_turn_matching_transformation(&normalized, &controller, time_step, -3, -3, Some(skip.local_vx), Some(skip.local_vy));
    } else if x2y1.is_some() && x1y1.is_none() {
        //println!("-- x1y1 fallback --");
        let x2y1_transformation = x2y1.as_ref().unwrap();
        let skip = x2y1_transformation.normalized_player(current.angular_velocity.z);
        x1y1 = ground_turn_matching_transformation(&normalized, &controller, time_step, 3, -3, Some(skip.local_vx), Some(skip.local_vy));
    } else if x2y1.is_none() && x1y1.is_none() {
        //println!("-- BOTH FAILED --");
    }

    //println!("x1y2");
    let normalized = sample::normalized_player(&current, false, true);
    let mut x1y2 = ground_turn_matching_transformation(&normalized, &controller, time_step, -3, 3, None, None);

    //println!("x2y2");
    let normalized = sample::normalized_player(&current, true, true);
    let mut x2y2 = ground_turn_matching_transformation(&normalized, &controller, time_step, 3, 3, None, None);

    // when we fail in on direction, search in the other
    if x1y2.is_some() && x2y2.is_none() {
        //println!("-- x2y2 fallback --");
        let x1y2_transformation = x1y2.as_ref().unwrap();
        let skip = x1y2_transformation.normalized_player(current.angular_velocity.z);
        x2y2 = ground_turn_matching_transformation(&normalized, &controller, time_step, -3, 3, Some(skip.local_vx), Some(skip.local_vy));
    } else if x2y2.is_some() && x1y2.is_none() {
        //println!("-- x1y2 fallback --");
        let x2y2_transformation = x2y2.as_ref().unwrap();
        let skip = x2y2_transformation.normalized_player(current.angular_velocity.z);
        x1y2 = ground_turn_matching_transformation(&normalized, &controller, time_step, 3, 3, Some(skip.local_vx), Some(skip.local_vy));
    } else if x2y2.is_none() && x1y2.is_none() {
        //println!("-- BOTH FAILED 2 --");
    }

    [x1y1, x2y1, x1y2, x2y2]
}

/// returns tuple of (translation, acceleration, angular_acceleration, rotation)
fn ground_turn_prediction(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> Result<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Rotation3<f32>), String> {
    //println!("-----------------------------");
    //println!("local_velocity: {:?}", current.local_velocity());
    //println!("controller: {:?}", controller);

    let quad = ground_turn_quad_tranformations(current, controller, time_step);

    // TODO error
    let x1y1 = quad[0].ok_or_else(|| format!(
        "Missing turn x1y1 for player: {:?} & controller: {:?}",
        sample::normalized_player(&current, false, false), controller
    ))?;
    let x2y1 = quad[1].ok_or_else(|| format!(
        "Missing turn x2y1 for player: {:?} & controller: {:?}",
        sample::normalized_player(&current, true, false), controller
    ))?;
    let x1y2 = quad[2].ok_or_else(|| format!(
        "Missing turn x1y2 for player: {:?} & controller: {:?}",
        sample::normalized_player(&current, false, true), controller
    ))?;
    let x2y2 = quad[3].ok_or_else(|| format!(
        "Missing turn x2y2 for player: {:?} & controller: {:?}",
        sample::normalized_player(&current, true, true), controller
    ))?;

    let current_vx = current.local_velocity().x;
    let current_vy = current.local_velocity().y;

    let y1_vx1 = x1y1.start_local_vx as f32;
    let y1_vx2 = x2y1.start_local_vx as f32;

    let y1_vy1 = x1y1.start_local_vy as f32;
    let y1_vy2 = x2y1.start_local_vy as f32;

    let y2_vx1 = x1y2.start_local_vx as f32;
    let y2_vx2 = x2y2.start_local_vx as f32;

    let y2_vy1 = x1y2.start_local_vy as f32;
    let y2_vy2 = x2y2.start_local_vy as f32;

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

    let current_rotation = current.rotation.to_rotation_matrix();

    let translation_x1y1 = Vector3::new(x1y1.translation_x as f32, x1y1.translation_y as f32, 0.0);
    let translation_x2y1 = Vector3::new(x2y1.translation_x as f32, x2y1.translation_y as f32, 0.0);
    let translation_x1y2 = Vector3::new(x1y2.translation_x as f32, x1y2.translation_y as f32, 0.0);
    let translation_x2y2 = Vector3::new(x2y2.translation_x as f32, x2y2.translation_y as f32, 0.0);
    let translation_y1 = interpolate(translation_x1y1, translation_x2y1, y1_vx_factor);
    let translation_y2 = interpolate(translation_x1y2, translation_x2y2, y2_vx_factor);
    let translation = current_rotation * interpolate(translation_y1, translation_y2, vy_factor);

    // dbg!(current.rotation.to_euler_angles().2);
    // dbg!(x1y1_start.rotation.to_euler_angles().2);
    // dbg!(x2y1_start.rotation.to_euler_angles().2);
    // dbg!(x1y2_start.rotation.to_euler_angles().2);
    // dbg!(x2y2_start.rotation.to_euler_angles().2);

    // dbg!(x1y1_start.angular_velocity.x);
    // dbg!(x2y1_start.angular_velocity.x);
    // dbg!(x1y2_start.angular_velocity.x);
    // dbg!(x2y2_start.angular_velocity.x);

    // dbg!(x1y1_start.position.x);
    // dbg!(x1y1_start.position.y);

    // dbg!(x1y1_start.velocity.y);
    // dbg!(x2y1_start.velocity.y);
    // dbg!(x1y2_start.velocity.y);
    // dbg!(x2y2_start.velocity.y);
    // dbg!(x1y1_start.local_velocity().y);
    // dbg!(x2y1_start.local_velocity().y);
    // dbg!(x1y2_start.local_velocity().y);
    // dbg!(x2y2_start.local_velocity().y);

    // dbg!(x1y1_end.velocity.y);
    // dbg!(x2y1_end.velocity.y);
    // dbg!(x1y2_end.velocity.y);
    // dbg!(x2y2_end.velocity.y);
    // dbg!(x1y1_end.local_velocity().y);
    // dbg!(x2y1_end.local_velocity().y);
    // dbg!(x1y2_end.local_velocity().y);
    // dbg!(x2y2_end.local_velocity().y);

    let end_velocity_x1y1 = Vector3::new(x1y1.end_velocity_x as f32, x1y1.end_velocity_y as f32, 0.0);
    let end_velocity_x2y1 = Vector3::new(x2y1.end_velocity_x as f32, x2y1.end_velocity_y as f32, 0.0);
    let end_velocity_x1y2 = Vector3::new(x1y2.end_velocity_x as f32, x1y2.end_velocity_y as f32, 0.0);
    let end_velocity_x2y2 = Vector3::new(x2y2.end_velocity_x as f32, x2y2.end_velocity_y as f32, 0.0);
    let end_velocity_y1 = interpolate(end_velocity_x1y1, end_velocity_x2y1, y1_vx_factor);
    let end_velocity_y2 = interpolate(end_velocity_x1y2, end_velocity_x2y2, y2_vx_factor);
    let end_velocity = current_rotation * interpolate(end_velocity_y1, end_velocity_y2, vy_factor);

    // dbg!(end_velocity_x1y1);
    // dbg!(end_velocity_x2y1);
    // dbg!(end_velocity_x1y2);
    // dbg!(end_velocity_x2y2);
    // dbg!(end_velocity_y1);
    // dbg!(end_velocity_y2);
    // dbg!(end_velocity);

    Ok((
        translation,
        end_velocity,
        // assuming they are all pretty similar
        Vector3::new(0.0, 0.0, x1y1.end_angular_velocity_z),
        // TODO interpolate yaw, but have to handle the fact that it's circular
        current_rotation * Rotation3::from_euler_angles(0.0, 0.0, x1y1.end_yaw),
    ))
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

#[no_mangle]
pub extern "C" fn next_player_state(
    current: &PlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> Result<PlayerState, String> {
    let mut next_player = match find_prediction_category(&current) {
        PredictionCategory::Ground => next_player_state_grounded(&current, &controller, time_step)?,
        //PredictionCategory::Ground2 => next_velocity_grounded2(&current, &controller, time_step),
        //PredictionCategory::Wall => next_velocity_walled(&current, &controller, time_step),
        //PredictionCategory::Ceiling => next_velocity_ceilinged(&current, &controller, time_step),
        //PredictionCategory::CurveWall => next_velocity_curve_walled(&current, &controller, time_step),
        //PredictionCategory::Air => next_velocity_flying(&current, &controller, time_step),
    };

    if next_player.position.z < CAR_DIMENSIONS.z / 2.0 {
        next_player.position.z = CAR_DIMENSIONS.z / 2.0;
    }

    Ok(next_player)
}
