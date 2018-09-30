use state::*;
use std::fs;
use na::{Vector3, UnitQuaternion};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
type MyHasher = BuildHasherDefault<FnvHasher>;
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


    pub static ref THROTTLE_RIGHT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/throttle_right/");
    pub static ref THROTTLE_RIGHT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&THROTTLE_RIGHT_TURN_ALL);
    pub static ref THROTTLE_LEFT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/throttle_left/");
    pub static ref THROTTLE_LEFT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&THROTTLE_LEFT_TURN_ALL);
    pub static ref BOOST_RIGHT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/boost_right/");
    pub static ref BOOST_RIGHT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_RIGHT_TURN_ALL);
    pub static ref BOOST_LEFT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/boost_left/");
    pub static ref BOOST_LEFT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_LEFT_TURN_ALL);
    pub static ref IDLE_RIGHT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/idle_right/");
    pub static ref IDLE_RIGHT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_RIGHT_TURN_ALL);
    pub static ref IDLE_LEFT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/idle_left/");
    pub static ref IDLE_LEFT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_LEFT_TURN_ALL);
    pub static ref BRAKE_RIGHT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/brake_right/");
    pub static ref BRAKE_RIGHT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&BRAKE_RIGHT_TURN_ALL);
    pub static ref BRAKE_LEFT_TURN_ALL: Vec<Vec<PlayerState>> = load_all_samples("./data/samples/turning/brake_left/");
    pub static ref BRAKE_LEFT_TURN_INDEXED: SampleMap<'static> = index_all_samples(&BRAKE_LEFT_TURN_ALL);
}

fn load_sample_file(path: &str) -> Vec<PlayerState> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(fs::File::open(path).expect(&format!("File doesn't exist: {}", path)));
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

            angular_velocity: Vector3::new(
                         record.get(6).expect("Invalid row?").parse().expect("Can't convert avx to f32"),
                         record.get(7).expect("Invalid row?").parse().expect("Can't convert avy to f32"),
                         record.get(8).expect("Invalid row?").parse().expect("Can't convert avz to f32"),
                      ),

            rotation: UnitQuaternion::from_euler_angles(
                         record.get(9).expect("Invalid row?").parse().expect("Can't convert roll to f32"),
                         record.get(10).expect("Invalid row?").parse().expect("Can't convert pitch to f32"),
                         record.get(11).expect("Invalid row?").parse().expect("Can't convert yaw to f32"),
                      ),

            team: Team::Blue, // doesn't matter
        }
    }).collect()
}

fn load_all_samples(dir: &str) -> Vec<Vec<PlayerState>> {
    let files: Vec<String> = files(dir);
    files.iter().map(|f| load_sample_file(f)).collect()
}

fn files(dir: &str) -> Vec<String> {
    let entries = fs::read_dir(dir).expect(&format!("Directory doesn't exist?: {}", dir));
    entries.map(|entry| {
        let path = entry.expect(&format!("IO Error for dir {} entry", dir)).path();
        path.to_str().expect(&format!("Failed to_str for path: {:?}", path)).to_owned()
    }).collect::<Vec<_>>()
}

pub type SampleMap<'a> = HashMap<NormalizedPlayerState, &'a [PlayerState], MyHasher>;

pub fn index_all_samples<'a>(all_samples: &'a Vec<Vec<PlayerState>>) -> SampleMap<'a> {
    let mut indexed = SampleMap::default();

    for i in 0..all_samples.len() {
        let sample = &all_samples[i];
        let mut j = 0;
        while j < sample.len() - 240 { // subtract 240 frames to ensure at least 1 second of simulation ahead in the slice
            let key = normalized_player(&sample[j]);
            // don't overwite values already inserted. this way we keep longer sample slices given
            // an asymptotic sample
            // TODO we may have a better sample here than before? in which case we should evaluate
            // the sample quality (eg how far away is the rounded value from the real? or how many
            // samples do we have following, given more is better up to a second).
            indexed.entry(key).or_insert(&all_samples[i][j..]);
            j += 1;
        }
    }

    indexed
}

// XXX is the use of i16 here actually helping?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NormalizedPlayerState {
    speed: i16,
    //avx: i16,
    //avy: i16,
    avz: i16,
    //roll: i16,
    //pitch: i16,
    //yaw: i16,
}

pub fn normalized_player(player: &PlayerState) -> NormalizedPlayerState {
    NormalizedPlayerState {
        speed: (player.velocity.norm() / 100.0).round() as i16,
        avz: (player.angular_velocity.z * 5.0).round() as i16,
    }
}

pub(crate) fn get_relevant_turn_samples(controller: &BrickControllerState, decelerating: bool) -> &'static Vec<PlayerState> {
    match(decelerating, &controller.steer, &controller.throttle, controller.boost) {
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

pub(crate) fn get_relevant_turn_samples_v2(player: &PlayerState, controller: &BrickControllerState) -> &'static [PlayerState] {
    let normalized = normalized_player(&player);

    let sample_map: &SampleMap = match(&controller.steer, &controller.throttle, controller.boost) {
        (&Steer::Right, &Throttle::Forward, false) => &THROTTLE_RIGHT_TURN_INDEXED,
        (&Steer::Right, _                 , true ) => &BOOST_RIGHT_TURN_INDEXED, // TODO confirm braking plus boosting is same as boosting
        (&Steer::Right, &Throttle::Idle   , false) => &IDLE_RIGHT_TURN_INDEXED,
        (&Steer::Right, &Throttle::Reverse, false) => &BRAKE_RIGHT_TURN_INDEXED,

        (&Steer::Left , &Throttle::Forward, false) => &THROTTLE_LEFT_TURN_INDEXED,
        (&Steer::Left , _                 , true ) => &BOOST_LEFT_TURN_INDEXED,
        (&Steer::Left , &Throttle::Idle   , false) => &IDLE_LEFT_TURN_INDEXED,
        (&Steer::Left , &Throttle::Reverse, false) => &BRAKE_LEFT_TURN_INDEXED,

        (&Steer::Straight, _, _) => panic!("Going straight isn't handled here."),
    };

    sample_map.get(&normalized).expect(&format!("Missing turn sample for player: {:?} & controller: {:?}", normalized, controller))
}
