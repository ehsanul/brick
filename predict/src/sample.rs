use state::*;
use std::fs::File;
use na::{Vector3, UnitQuaternion, Rotation3};
use csv;

lazy_static! {
    pub static ref REST_THROTTLE_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file("./data/samples/rest_throttle_right.csv");
    pub static ref MAX_SPEED_THROTTLE_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file("./data/samples/max_speed_throttle_right.csv");

    pub static ref REST_BOOST_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO
    pub static ref MAX_SPEED_BOOST_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO

    pub static ref MAX_SPEED_IDLE_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO
    pub static ref MAX_SPEED_BRAKE_RIGHT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO

    pub static ref REST_THROTTLE_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file("./data/samples/rest_throttle_left.csv");
    pub static ref MAX_SPEED_THROTTLE_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file("./data/samples/max_speed_throttle_left.csv");

    pub static ref REST_BOOST_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO
    pub static ref MAX_SPEED_BOOST_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO

    pub static ref MAX_SPEED_IDLE_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO
    pub static ref MAX_SPEED_BRAKE_LEFT_TURN_SAMPLE: Vec<PlayerState> = load_sample_file(""); // TODO
}

fn load_sample_file(path: &str) -> Vec<PlayerState> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(File::open(path).expect(&format!("File doesn't exist: {}", path)));
    rdr.records().map(|record| {
        let record = record.expect("CSV parse failed?");
        PlayerState {
            position: Vector3::new(
                         record.get(0).expect("Invalid row?").parse().expect("Can't convert x to f32"),
                         record.get(1).expect("Invalid row?").parse().expect("Can't convert y to f32"),
                         record.get(2).expect("Invalid row?").parse().expect("Can't convert z to f32"),
                      ),
            velocity: Vector3::new(
                         record.get(3).expect("Invalid row?").parse().expect("Can't convert vx to f32"),
                         record.get(4).expect("Invalid row?").parse().expect("Can't convert vy to f32"),
                         record.get(5).expect("Invalid row?").parse().expect("Can't convert vz to f32"),
                      ),
            rotation: UnitQuaternion::from_euler_angles(
                         record.get(6).expect("Invalid row?").parse().expect("Can't convert roll to f32"),
                         record.get(7).expect("Invalid row?").parse().expect("Can't convert pitch to f32"),
                         record.get(8).expect("Invalid row?").parse().expect("Can't convert yaw to f32"),
                      ),

            team: Team::Blue, // doesn't matter
        }
    }).collect()
}

pub(crate) fn get_relevant_turn_samples(controller: &BrickControllerState, decelerating: bool) -> &'static Vec<PlayerState> {
    match((decelerating, &controller.steer, &controller.throttle, controller.boost)) {
        (false, &Steer::Right, &Throttle::Forward, false) => &REST_THROTTLE_RIGHT_TURN_SAMPLE,
        (false, &Steer::Right, _                 , true ) => &REST_BOOST_RIGHT_TURN_SAMPLE, // TODO confirm braking plus boosting is same as boosting
        (true , &Steer::Right, &Throttle::Forward, false) => &MAX_SPEED_THROTTLE_RIGHT_TURN_SAMPLE,
        (true , &Steer::Right, _                 , true ) => &MAX_SPEED_BOOST_RIGHT_TURN_SAMPLE,

        (_    , &Steer::Right, &Throttle::Idle   , false) => &MAX_SPEED_IDLE_RIGHT_TURN_SAMPLE,
        (_    , &Steer::Right, &Throttle::Reverse, false) => &MAX_SPEED_BRAKE_RIGHT_TURN_SAMPLE,

        (false, &Steer::Left , &Throttle::Forward, false) => &REST_THROTTLE_LEFT_TURN_SAMPLE,
        (false, &Steer::Left , _                 , true ) => &REST_BOOST_LEFT_TURN_SAMPLE,
        (true , &Steer::Left , &Throttle::Forward, false) => &MAX_SPEED_THROTTLE_LEFT_TURN_SAMPLE,
        (true , &Steer::Left , _                 , true ) => &MAX_SPEED_BOOST_LEFT_TURN_SAMPLE,

        (_    , &Steer::Left , &Throttle::Idle   , false) => &MAX_SPEED_IDLE_LEFT_TURN_SAMPLE,
        (_    , &Steer::Left , &Throttle::Reverse, false) => &MAX_SPEED_BRAKE_LEFT_TURN_SAMPLE,

        (_, &Steer::Straight, _, _) => panic!("Going straight isn't handled here."),
    }
}
