use fnv::FnvHasher;
use na::{UnitQuaternion, Vector3};
use state::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fs;
use std::hash::BuildHasherDefault;
type MyHasher = BuildHasherDefault<FnvHasher>;
use csv;

pub const RECORD_FPS: usize = 120;

lazy_static! {
    pub static ref THROTTLE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_straight/");
    pub static ref THROTTLE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_STRAIGHT_ALL);
    pub static ref THROTTLE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_right/");
    pub static ref THROTTLE_RIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_RIGHT_ALL);
    pub static ref THROTTLE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_left/");
    pub static ref THROTTLE_LEFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_LEFT_ALL);
    pub static ref BOOST_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_straight/");
    pub static ref BOOST_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_STRAIGHT_ALL);
    pub static ref BOOST_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_right/");
    pub static ref BOOST_RIGHT_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_RIGHT_ALL);
    pub static ref BOOST_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_left/");
    pub static ref BOOST_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_LEFT_ALL);
    pub static ref IDLE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/idle_straight/");
    pub static ref IDLE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&IDLE_STRAIGHT_ALL);
    pub static ref IDLE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/idle_right/");
    pub static ref IDLE_RIGHT_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_RIGHT_ALL);
    pub static ref IDLE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/idle_left/");
    pub static ref IDLE_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_LEFT_ALL);
    pub static ref REVERSE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/reverse_straight/");
    pub static ref REVERSE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_STRAIGHT_ALL);
    pub static ref REVERSE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/reverse_right/");
    pub static ref REVERSE_RIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_RIGHT_ALL);
    pub static ref REVERSE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/reverse_left/");
    pub static ref REVERSE_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&REVERSE_LEFT_ALL);
    pub static ref BRAKE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/brake_straight/");
    pub static ref BRAKE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&BRAKE_STRAIGHT_ALL);
    pub static ref BRAKE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/brake_right/");
    pub static ref BRAKE_RIGHT_INDEXED: SampleMap<'static> = index_all_samples(&BRAKE_RIGHT_ALL);
    pub static ref BRAKE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/brake_left/");
    pub static ref BRAKE_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&BRAKE_LEFT_ALL);
}

lazy_static! {
    pub static ref THROTTLE_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_straight_drift/");
    pub static ref THROTTLE_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_STRAIGHT_DRIFT_ALL);
    pub static ref THROTTLE_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_right_drift/");
    pub static ref THROTTLE_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_RIGHT_DRIFT_ALL);
    pub static ref THROTTLE_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/throttle_left_drift/");
    pub static ref THROTTLE_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_LEFT_DRIFT_ALL);
    pub static ref BOOST_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_straight_drift/");
    pub static ref BOOST_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_STRAIGHT_DRIFT_ALL);
    pub static ref BOOST_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_right_drift/");
    pub static ref BOOST_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_RIGHT_DRIFT_ALL);
    pub static ref BOOST_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/turning/boost_left_drift/");
    pub static ref BOOST_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_LEFT_DRIFT_ALL);
}

fn load_sample_file(path: &str) -> Vec<PlayerState> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(fs::File::open(path).expect(&format!("File doesn't exist: {}", path)));
    rdr.records()
        .map(|record| {
            let record = record.expect("CSV parse failed?");
            let _time: f32 = record
                .get(0)
                .expect("Invalid row?")
                .parse()
                .expect("Can't convert time to f32");
            PlayerState {
                position: Vector3::new(
                    record
                        .get(0)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert x to f32"),
                    record
                        .get(1)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert y to f32"),
                    record
                        .get(2)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert z to f32"),
                ),
                velocity: Vector3::new(
                    record
                        .get(3)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vx to f32"),
                    record
                        .get(4)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vy to f32"),
                    record
                        .get(5)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vz to f32"),
                ),

                angular_velocity: Vector3::new(
                    record
                        .get(6)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert roll to f32"),
                    record
                        .get(7)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert pitch to f32"),
                    record
                        .get(8)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert yaw to f32"),
                ),

                rotation: UnitQuaternion::from_euler_angles(
                    record
                        .get(9)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert avx to f32"),
                    record
                        .get(10)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert avy to f32"),
                    -record
                        .get(11)
                        .expect("Invalid row?")
                        .parse::<f32>()
                        .expect("Can't convert avz to f32"), // FIXME temp hack
                ),

                team: Team::Blue, // doesn't matter
            }
        })
        .collect()
}

fn load_all_samples(dir: &str) -> Vec<Vec<PlayerState>> {
    let files: Vec<String> = files(dir);
    files.iter().map(|f| load_sample_file(f)).collect()
}

fn files(dir: &str) -> Vec<String> {
    let entries = fs::read_dir(dir).expect(&format!("Directory doesn't exist?: {}", dir));
    entries
        .map(|entry| {
            let path = entry
                .expect(&format!("IO Error for dir {} entry", dir))
                .path();
            path.to_str()
                .expect(&format!("Failed to_str for path: {:?}", path))
                .to_owned()
        })
        .collect::<Vec<_>>()
}

pub type SampleMap<'a> = HashMap<NormalizedPlayerState, &'a [PlayerState], MyHasher>;

pub fn index_all_samples<'a>(all_samples: &'a Vec<Vec<PlayerState>>) -> SampleMap<'a> {
    let mut indexed = SampleMap::default();

    for i in 0..all_samples.len() {
        let sample = &all_samples[i];
        let mut j = 0;
        while j < sample.len() - (RECORD_FPS / 2) {
            // subtract 0.5s worth of frames to ensure at least 0.5 seconds of simulation ahead in the slice
            let key = normalized_player_rounded(&sample[j]);
            match indexed.entry(key) {
                Vacant(e) => {
                    e.insert(&all_samples[i][j..]);
                }
                Occupied(mut e) => {
                    // replace the sample in case we have one closer to the intended normalized
                    // velocity value
                    let should_replace = {
                        let existing_sample = e.get();
                        let existing_delta = (existing_sample[0].velocity.norm()
                            - GROUND_SPEED_GRID_FACTOR * e.key().speed as f32)
                            .abs();
                        let new_delta = (sample[j].velocity.norm()
                            - GROUND_SPEED_GRID_FACTOR * e.key().speed as f32)
                            .abs();
                        new_delta < existing_delta
                    };
                    if should_replace {
                        e.insert(&all_samples[i][j..]);
                    }
                }
            };
            j += 1;
        }
    }

    assert_index_complete(&indexed);

    indexed
}

const GROUND_SPEED_GRID_FACTOR: f32 = 100.0;
const GROUND_AVZ_GRID_FACTOR: f32 = 0.2;

fn assert_index_complete<'a>(index: &SampleMap<'a>) {
    let min_avz = -(MAX_GROUND_ANGULAR_SPEED / GROUND_AVZ_GRID_FACTOR).round() as i16;
    let max_avz = (MAX_GROUND_ANGULAR_SPEED / GROUND_AVZ_GRID_FACTOR).round() as i16;
    for speed in 0..(MAX_BOOST_SPEED / GROUND_SPEED_GRID_FACTOR).round() as i16 {
        for avz in min_avz..max_avz {
            let normalized = NormalizedPlayerState { speed, avz };
            if index.get(&normalized).is_none() {
                panic!(format!("Missing: {:?}", normalized));
            }
        }
    }
}

// XXX is the use of i16 here actually helping?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NormalizedPlayerState {
    pub speed: i16,
    //avx: i16,
    //avy: i16,
    pub avz: i16,
    //roll: i16,
    //pitch: i16,
    //yaw: i16,
}

pub fn normalized_player_rounded(player: &PlayerState) -> NormalizedPlayerState {
    NormalizedPlayerState {
        speed: (player.velocity.norm() / GROUND_SPEED_GRID_FACTOR).round() as i16,
        avz: (player.angular_velocity.z / GROUND_AVZ_GRID_FACTOR).round() as i16,
    }
}

pub fn normalized_player(player: &PlayerState, ceil: bool) -> NormalizedPlayerState {
    if ceil {
        NormalizedPlayerState {
            speed: (player.velocity.norm() / GROUND_SPEED_GRID_FACTOR).ceil() as i16,
            avz: (player.angular_velocity.z / GROUND_AVZ_GRID_FACTOR).round() as i16,
        }
    } else {
        NormalizedPlayerState {
            speed: (player.velocity.norm() / GROUND_SPEED_GRID_FACTOR).floor() as i16,
            avz: (player.angular_velocity.z / GROUND_AVZ_GRID_FACTOR).round() as i16,
        }
    }
}

pub(crate) fn get_relevant_turn_samples(
    player: &PlayerState,
    controller: &BrickControllerState,
    ceil: bool,
) -> &'static [PlayerState] {
    let normalized = normalized_player(&player, ceil);

    let sample_map: &SampleMap = match (
        &controller.steer,
        &controller.throttle,
        controller.boost,
        controller.handbrake,
    ) {
        (&Steer::Right, &Throttle::Forward, false, false) => &THROTTLE_RIGHT_INDEXED,
        (&Steer::Right, _, true, false) => &BOOST_RIGHT_INDEXED, // TODO confirm braking plus boosting is same as boosting
        (&Steer::Right, &Throttle::Idle, false, false) => &IDLE_RIGHT_INDEXED,
        (&Steer::Right, &Throttle::Reverse, false, false) => &REVERSE_RIGHT_INDEXED,
        (&Steer::Right, &Throttle::Forward, false, true) => &THROTTLE_RIGHT_DRIFT_INDEXED,
        (&Steer::Right, &Throttle::Forward, true, true) => &BOOST_RIGHT_DRIFT_INDEXED,

        (&Steer::Left, &Throttle::Forward, false, false) => &THROTTLE_LEFT_INDEXED,
        (&Steer::Left, _, true, false) => &BOOST_LEFT_INDEXED,
        (&Steer::Left, &Throttle::Idle, false, false) => &IDLE_LEFT_INDEXED,
        (&Steer::Left, &Throttle::Reverse, false, false) => &REVERSE_LEFT_INDEXED,
        (&Steer::Left, &Throttle::Forward, false, true) => &THROTTLE_LEFT_DRIFT_INDEXED,
        (&Steer::Left, _, true, true) => &BOOST_LEFT_DRIFT_INDEXED,

        (&Steer::Straight, &Throttle::Forward, false, false) => &THROTTLE_STRAIGHT_INDEXED,
        (&Steer::Straight, _, true, false) => &BOOST_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Idle, false, false) => &IDLE_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Reverse, false, false) => &REVERSE_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Forward, false, true) => &THROTTLE_STRAIGHT_DRIFT_INDEXED,
        (&Steer::Straight, _, true, true) => &BOOST_STRAIGHT_DRIFT_INDEXED,

        // ignoring the other drift variants (idle/reverse) for now
        (_, _, _, true) => unimplemented!(),
    };

    sample_map.get(&normalized).expect(&format!(
        "Missing turn sample for player: {:?} & controller: {:?}",
        normalized, controller
    ))
}
