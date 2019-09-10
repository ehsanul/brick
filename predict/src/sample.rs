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
        load_all_samples("./data/samples/flat_ground/throttle_straight/");
    pub static ref THROTTLE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_STRAIGHT_ALL);
    pub static ref THROTTLE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/throttle_right/");
    pub static ref THROTTLE_RIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_RIGHT_ALL);
    pub static ref THROTTLE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/throttle_left/");
    pub static ref THROTTLE_LEFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_LEFT_ALL);
    pub static ref BOOST_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_straight/");
    pub static ref BOOST_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_STRAIGHT_ALL);
    pub static ref BOOST_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_right/");
    pub static ref BOOST_RIGHT_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_RIGHT_ALL);
    pub static ref BOOST_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_left/");
    pub static ref BOOST_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&BOOST_LEFT_ALL);
    pub static ref IDLE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_straight/");
    pub static ref IDLE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&IDLE_STRAIGHT_ALL);
    pub static ref IDLE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_right/");
    pub static ref IDLE_RIGHT_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_RIGHT_ALL);
    pub static ref IDLE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_left/");
    pub static ref IDLE_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&IDLE_LEFT_ALL);
    pub static ref REVERSE_STRAIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_straight/");
    pub static ref REVERSE_STRAIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_STRAIGHT_ALL);
    pub static ref REVERSE_RIGHT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_right/");
    pub static ref REVERSE_RIGHT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_RIGHT_ALL);
    pub static ref REVERSE_LEFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_left/");
    pub static ref REVERSE_LEFT_INDEXED: SampleMap<'static> = index_all_samples(&REVERSE_LEFT_ALL);
}

lazy_static! {
    pub static ref THROTTLE_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/throttle_straight_drift/");
    pub static ref THROTTLE_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_STRAIGHT_DRIFT_ALL);
    pub static ref THROTTLE_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/throttle_right_drift/");
    pub static ref THROTTLE_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_RIGHT_DRIFT_ALL);
    pub static ref THROTTLE_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/throttle_left_drift/");
    pub static ref THROTTLE_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&THROTTLE_LEFT_DRIFT_ALL);
    pub static ref BOOST_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_straight_drift/");
    pub static ref BOOST_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_STRAIGHT_DRIFT_ALL);
    pub static ref BOOST_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_right_drift/");
    pub static ref BOOST_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_RIGHT_DRIFT_ALL);
    pub static ref BOOST_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/boost_left_drift/");
    pub static ref BOOST_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&BOOST_LEFT_DRIFT_ALL);
    pub static ref IDLE_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_straight_drift/");
    pub static ref IDLE_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&IDLE_STRAIGHT_DRIFT_ALL);
    pub static ref IDLE_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_right_drift/");
    pub static ref IDLE_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&IDLE_RIGHT_DRIFT_ALL);
    pub static ref IDLE_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/idle_left_drift/");
    pub static ref IDLE_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&IDLE_LEFT_DRIFT_ALL);
    pub static ref REVERSE_STRAIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_straight_drift/");
    pub static ref REVERSE_STRAIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_STRAIGHT_DRIFT_ALL);
    pub static ref REVERSE_RIGHT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_right_drift/");
    pub static ref REVERSE_RIGHT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_RIGHT_DRIFT_ALL);
    pub static ref REVERSE_LEFT_DRIFT_ALL: Vec<Vec<PlayerState>> =
        load_all_samples("./data/samples/flat_ground/reverse_left_drift/");
    pub static ref REVERSE_LEFT_DRIFT_INDEXED: SampleMap<'static> =
        index_all_samples(&REVERSE_LEFT_DRIFT_ALL);
}

fn load_sample_file(path: &str) -> Vec<PlayerState> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(fs::File::open(path).expect(&format!("File doesn't exist: {}", path)));
    let data: Vec<PlayerState> = rdr.records()
        .map(|record| {
            let record = record.expect("CSV parse failed?");
            let _frame: f32 = record
                .get(0)
                .expect("Invalid row?")
                .parse()
                .expect("Can't convert time to f32");
            PlayerState {
                position: Vector3::new(
                    record
                        .get(1)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert x to f32"),
                    record
                        .get(2)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert y to f32"),
                    record
                        .get(3)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert z to f32"),
                ),
                velocity: Vector3::new(
                    record
                        .get(4)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vx to f32"),
                    record
                        .get(5)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vy to f32"),
                    record
                        .get(6)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert vz to f32"),
                ),

                angular_velocity: Vector3::new(
                    record
                        .get(0) // FIXME just to easily grep for the frame later and find the file
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert avx to f32"),
                    record
                        .get(8)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert avy to f32"),
                    record
                        .get(9)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert avz to f32"),
                ),

                rotation: UnitQuaternion::from_euler_angles(
                    record
                        .get(10)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert roll to f32"),
                    record
                        .get(11)
                        .expect("Invalid row?")
                        .parse()
                        .expect("Can't convert pitch to f32"),
                    -record // FIXME negative is a temp hack since data is recorded incorrectly
                        .get(12)
                        .expect("Invalid row?")
                        .parse::<f32>()
                        .expect("Can't convert yaw to f32"),
                ),

                team: Team::Blue, // doesn't matter
            }
        })
        .collect();
    if data.len() < 32 {
        println!("BAD FILE: {}", path);
    }
    data
}

fn load_all_samples(dir: &str) -> Vec<Vec<PlayerState>> {
    let files: Vec<String> = csv_files(dir);
    files.iter().map(|f| load_sample_file(f)).collect()
}

fn csv_files(dir: &str) -> Vec<String> {
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
        .filter(|path| path.ends_with(".csv"))
        .collect::<Vec<_>>()
}

pub type SampleMap<'a> = HashMap<NormalizedPlayerState, &'a [PlayerState], MyHasher>;

//fn continuous_sample(sample: &[PlayerState]) {
//    let mut last = sample[0];
//    for player in sample[1..] {
//        let predicted = player.position + TICK * last.velocity;
//        if last.velocity.x()
//
//        last = player;
//    }
//    return true
//}

pub fn index_all_samples<'a>(all_samples: &'a Vec<Vec<PlayerState>>) -> SampleMap<'a> {
    let mut indexed = SampleMap::default();

    for i in 0..all_samples.len() {
        let sample = &all_samples[i];

        if sample.len() < 32 {
            println!("bad sample: {:?}", sample[0]);
        }

        let mut j = 0;

        // subtract 32 frames to ensure 32 frames of simulation ahead in the slice
        while j < sample.len() - 32 {
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

                        let existing_lv = existing_sample[0].local_velocity();
                        let existing_delta_x = (existing_lv.x
                            - GROUND_SPEED_GRID_FACTOR * e.key().local_vx as f32)
                            .abs();
                        let existing_delta_y = (existing_lv.y
                            - GROUND_SPEED_GRID_FACTOR * e.key().local_vy as f32)
                            .abs();
                        let existing_delta =
                            (existing_delta_x.powf(2.0) + existing_delta_y.powf(2.0)).sqrt();

                        let new_lv = sample[j].local_velocity();
                        let new_delta_x =
                            (new_lv.x - GROUND_SPEED_GRID_FACTOR * e.key().local_vx as f32).abs();
                        let new_delta_y =
                            (new_lv.y - GROUND_SPEED_GRID_FACTOR * e.key().local_vy as f32).abs();
                        let new_delta = (new_delta_x.powf(2.0) + new_delta_y.powf(2.0)).sqrt();

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

    indexed
}

const GROUND_SPEED_GRID_FACTOR: f32 = 100.0;
const GROUND_AVZ_GRID_FACTOR: f32 = 0.2;

// XXX is the use of i16 here actually helping?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NormalizedPlayerState {
    pub local_vx: i16,
    pub local_vy: i16,
    //pub local_vz: i16,
    //avx: i16,
    //avy: i16,
    pub avz: i16,
    //roll: i16,
    //pitch: i16,
    //yaw: i16,
}

pub fn normalized_player_rounded(player: &PlayerState) -> NormalizedPlayerState {
    let lv = player.local_velocity();
    NormalizedPlayerState {
        local_vx: (lv.x / GROUND_SPEED_GRID_FACTOR).round() as i16,
        local_vy: (lv.y / GROUND_SPEED_GRID_FACTOR).round() as i16,
        avz: (player.angular_velocity.z / GROUND_AVZ_GRID_FACTOR).round() as i16,
    }
}

pub fn normalized_player(player: &PlayerState, ceil_vx: bool, ceil_vy: bool) -> NormalizedPlayerState {
    let avz = (player.angular_velocity.z / GROUND_AVZ_GRID_FACTOR).round() as i16;

    let lv = player.local_velocity();

    let local_vx = if ceil_vx {
        (lv.x / GROUND_SPEED_GRID_FACTOR).ceil() as i16
    } else {
        (lv.x / GROUND_SPEED_GRID_FACTOR).floor() as i16
    };

    let local_vy = if ceil_vy {
        (lv.y / GROUND_SPEED_GRID_FACTOR).ceil() as i16
    } else {
        (lv.y / GROUND_SPEED_GRID_FACTOR).floor() as i16
    };

    NormalizedPlayerState { local_vx, local_vy, avz }
}

pub(crate) fn get_relevant_turn_samples(
    normalized: &NormalizedPlayerState,
    controller: &BrickControllerState,
) -> Option<&'static [PlayerState]> {

    #[rustfmt::skip]
    let sample_map: &SampleMap = match (
        &controller.steer,
        &controller.throttle,
        controller.boost,
        controller.handbrake,
    ) {
        (&Steer::Right   , &Throttle::Forward, false, false) => &THROTTLE_RIGHT_INDEXED,
        (&Steer::Right   , _                 , true , false) => &BOOST_RIGHT_INDEXED, // TODO confirm braking plus boosting is same as boosting
        (&Steer::Right   , &Throttle::Idle   , false, false) => &IDLE_RIGHT_INDEXED,
        (&Steer::Right   , &Throttle::Reverse, false, false) => &REVERSE_RIGHT_INDEXED,
        (&Steer::Right   , &Throttle::Forward, false, true ) => &THROTTLE_RIGHT_DRIFT_INDEXED,
        (&Steer::Right   , &Throttle::Forward, true , true ) => &BOOST_RIGHT_DRIFT_INDEXED,

        (&Steer::Left    , &Throttle::Forward, false, false) => &THROTTLE_LEFT_INDEXED,
        (&Steer::Left    , _                 , true , false) => &BOOST_LEFT_INDEXED,
        (&Steer::Left    , &Throttle::Idle   , false, false) => &IDLE_LEFT_INDEXED,
        (&Steer::Left    , &Throttle::Reverse, false, false) => &REVERSE_LEFT_INDEXED,
        (&Steer::Left    , &Throttle::Forward, false, true ) => &THROTTLE_LEFT_DRIFT_INDEXED,
        (&Steer::Left    , _                 , true , true ) => &BOOST_LEFT_DRIFT_INDEXED,

        (&Steer::Straight, &Throttle::Forward, false, false) => &THROTTLE_STRAIGHT_INDEXED,
        (&Steer::Straight, _                 , true , false) => &BOOST_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Idle   , false, false) => &IDLE_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Reverse, false, false) => &REVERSE_STRAIGHT_INDEXED,
        (&Steer::Straight, &Throttle::Forward, false, true ) => &THROTTLE_STRAIGHT_DRIFT_INDEXED,
        (&Steer::Straight, _                 , true , true ) => &BOOST_STRAIGHT_DRIFT_INDEXED,

        // ignoring the other drift variants (idle/reverse) for now
        (_               , _                 , _    , true ) => unimplemented!(),
    };

    sample_map.get(&normalized).map(|x| *x)
}
