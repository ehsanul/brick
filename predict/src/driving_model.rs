use bincode::deserialize_from;

use fnv::FnvHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::error::Error;

use flate2::read::GzDecoder;
use std::io::BufReader;
use std::fs::File;

use sample;
use state::*;

type MyHasher = BuildHasherDefault<FnvHasher>;

lazy_static! {
    static ref THROTTLE_RIGHT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_right.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_RIGHT: DrivingModel = DrivingModel::load("models/flat_ground/boost_right.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_RIGHT: DrivingModel = DrivingModel::load("models/flat_ground/idle_right.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_RIGHT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_right.bincode.gz").expect("Failed to load driving model");

    static ref THROTTLE_RIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_right_drift.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_RIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/boost_right_drift.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_RIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/idle_right_drift.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_RIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_right_drift.bincode.gz").expect("Failed to load driving model");

    static ref THROTTLE_LEFT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_left.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_LEFT: DrivingModel = DrivingModel::load("models/flat_ground/boost_left.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_LEFT: DrivingModel = DrivingModel::load("models/flat_ground/idle_left.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_LEFT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_left.bincode.gz").expect("Failed to load driving model");

    static ref THROTTLE_LEFT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_left_drift.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_LEFT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/boost_left_drift.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_LEFT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/idle_left_drift.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_LEFT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_left_drift.bincode.gz").expect("Failed to load driving model");

    static ref THROTTLE_STRAIGHT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_straight.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_STRAIGHT: DrivingModel = DrivingModel::load("models/flat_ground/boost_straight.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_STRAIGHT: DrivingModel = DrivingModel::load("models/flat_ground/idle_straight.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_STRAIGHT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_straight.bincode.gz").expect("Failed to load driving model");

    static ref THROTTLE_STRAIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/throttle_straight_drift.bincode.gz").expect("Failed to load driving model");
    static ref BOOST_STRAIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/boost_straight_drift.bincode.gz").expect("Failed to load driving model");
    static ref IDLE_STRAIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/idle_straight_drift.bincode.gz").expect("Failed to load driving model");
    static ref REVERSE_STRAIGHT_DRIFT: DrivingModel = DrivingModel::load("models/flat_ground/reverse_straight_drift.bincode.gz").expect("Failed to load driving model");
}

pub type TransformationMap = HashMap<sample::NormalizedPlayerState, PlayerTransformation, MyHasher>;

#[derive(Serialize, Deserialize, Default)]
pub struct DrivingModel {
    //pub tick32: TransformationMap,
    pub tick16: TransformationMap,
    //pub tick8: TransformationMap,
    //pub tick4: TransformationMap,
    pub tick2: TransformationMap,
}

impl DrivingModel {
    pub fn load(path: &str) -> Result<DrivingModel, Box<dyn Error>> {
        let f = BufReader::new(File::open(path)?);
        let mut decoder = GzDecoder::new(f);
        Ok(deserialize_from(&mut decoder)?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerTransformation {
    pub start_local_vx: i16,
    pub start_local_vy: i16,
    pub translation_x: i16, // normalized
    pub translation_y: i16, // normalized
    pub end_velocity_x: i16, // normalized
    pub end_velocity_y: i16, // normalized
    pub end_yaw: f32, // normalized
    pub end_angular_velocity_z: f32,
}

impl PlayerTransformation {
    pub fn normalized_player(&self, avz: f32) -> sample::NormalizedPlayerState {
        sample::NormalizedPlayerState {
            local_vx: (self.start_local_vx as f32 / sample::GROUND_SPEED_GRID_FACTOR).round() as i16,
            local_vy: (self.start_local_vy as f32 / sample::GROUND_SPEED_GRID_FACTOR).round() as i16,
            avz: (avz / sample::GROUND_AVZ_GRID_FACTOR).round() as i16,
        }
    }

    pub fn from_samples(samples: &[PlayerState], num_ticks: usize) -> PlayerTransformation {
        let start = samples[0];
        let ratio = FPS as usize / sample::RECORD_FPS; // if we record at 60fps instead of 120fps, we should ensure we use the right index
        let end = samples[num_ticks / ratio];
        let local_v = start.local_velocity();
        let normalization_rotation = start.rotation.to_rotation_matrix().inverse();
        let translation = normalization_rotation * (end.position - start.position);
        let end_velocity = normalization_rotation * end.velocity;
        let end_yaw = (normalization_rotation * end.rotation.to_rotation_matrix()).euler_angles().2;

        PlayerTransformation {
            start_local_vx: local_v.x.round() as i16,
            start_local_vy: local_v.y.round() as i16,
            translation_x: translation.x.round() as i16,
            translation_y: translation.y.round() as i16,
            end_velocity_x: end_velocity.x.round() as i16,
            end_velocity_y: end_velocity.y.round() as i16,
            end_yaw: end_yaw,
            end_angular_velocity_z: end.angular_velocity.z,
        }
    }
}

pub(crate) fn get_relevant_transformation(
    normalized: &sample::NormalizedPlayerState,
    controller: &BrickControllerState,
    time_step: f32,
) -> Option<&'static PlayerTransformation> {

    #[rustfmt::skip]
    let driving_model: &DrivingModel = match (
        &controller.steer,
        &controller.throttle,
        controller.boost,
        controller.handbrake,
    ) {
        (&Steer::Right   , &Throttle::Forward, false, false) => &THROTTLE_RIGHT,
        (&Steer::Right   , _                 , true , false) => &BOOST_RIGHT, // TODO confirm braking plus boosting is same as boosting
        (&Steer::Right   , &Throttle::Idle   , false, false) => &IDLE_RIGHT,
        (&Steer::Right   , &Throttle::Reverse, false, false) => &REVERSE_RIGHT,
        (&Steer::Right   , &Throttle::Forward, false, true ) => &THROTTLE_RIGHT_DRIFT,
        (&Steer::Right   , &Throttle::Forward, true , true ) => &BOOST_RIGHT_DRIFT,

        (&Steer::Left    , &Throttle::Forward, false, false) => &THROTTLE_LEFT,
        (&Steer::Left    , _                 , true , false) => &BOOST_LEFT,
        (&Steer::Left    , &Throttle::Idle   , false, false) => &IDLE_LEFT,
        (&Steer::Left    , &Throttle::Reverse, false, false) => &REVERSE_LEFT,
        (&Steer::Left    , &Throttle::Forward, false, true ) => &THROTTLE_LEFT_DRIFT,
        (&Steer::Left    , _                 , true , true ) => &BOOST_LEFT_DRIFT,

        (&Steer::Straight, &Throttle::Forward, false, false) => &THROTTLE_STRAIGHT,
        (&Steer::Straight, _                 , true , false) => &BOOST_STRAIGHT,
        (&Steer::Straight, &Throttle::Idle   , false, false) => &IDLE_STRAIGHT,
        (&Steer::Straight, &Throttle::Reverse, false, false) => &REVERSE_STRAIGHT,
        (&Steer::Straight, &Throttle::Forward, false, true ) => &THROTTLE_STRAIGHT_DRIFT,
        (&Steer::Straight, _                 , true , true ) => &BOOST_STRAIGHT_DRIFT,

        // ignoring the other drift variants (idle/reverse) for now
        (_               , _                 , _    , true ) => unimplemented!(),
    };

    // TODO use const fn + match when possible: https://github.com/rust-lang/rust/issues/57240
    if time_step == 16.0 * TICK {
        driving_model.tick16.get(&normalized)
    //} else if time_step == 32.0 * TICK {
    //    driving_model.tick32.get(&normalized)
    } else if time_step == 2.0 * TICK {
        driving_model.tick2.get(&normalized)
    } else {
        panic!(format!("Don't know how to model time step: {}", time_step));
    }
}
