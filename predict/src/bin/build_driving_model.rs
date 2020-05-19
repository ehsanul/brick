extern crate state;
extern crate bincode;
extern crate flate2;
extern crate predict;

use bincode::serialize_into;
use predict::driving_model::{DrivingModel, PlayerTransformation, TransformationMap};
use predict::sample;
use std::io::BufWriter;
use std::path::Path;
use std::fs::{create_dir_all, File};
use state::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error::Error;

fn build_model_for(control_branch: &str) -> DrivingModel {
    let all_samples = sample::load_all_samples(&format!("./data/samples/flat_ground/{}/", control_branch));
    let mut model = DrivingModel::default();
    index_all_samples(&mut model.tick2, &all_samples, 2);
    index_all_samples(&mut model.tick16, &all_samples, 16);
    index_all_samples(&mut model.tick32, &all_samples, 32);
    model
}

fn write_model(path: &Path, model: DrivingModel) -> Result<(), Box<dyn Error>> {
    let f = BufWriter::new(File::create(path)?);
    let mut e = GzEncoder::new(f, Compression::default());
    Ok(serialize_into(&mut e, &model)?)
}

fn index_all_samples(indexed: &mut TransformationMap, all_samples: &Vec<Vec<PlayerState>>, num_ticks: usize) {
    for i in 0..all_samples.len() {
        let sample = &all_samples[i];

        if sample.len() < sample::MIN_SAMPLE_LENGTH {
            println!("bad sample: {:?}", sample[0]);
        }

        let mut j = 0;

        let ratio = FPS as usize / sample::RECORD_FPS; // if we record at 60fps instead of 120fps, we should ensure we use the right index
        while all_samples[i][j..].len() >= 1 + num_ticks / ratio {
            let key = sample::normalized_player_rounded(&sample[j]);

            match indexed.entry(key) {
                Vacant(e) => {
                    e.insert(PlayerTransformation::from_samples(&all_samples[i][j..], num_ticks));
                }
                Occupied(mut e) => {
                    // replace the sample in case we have one closer to the intended normalized
                    // velocity value
                    let should_replace = {
                        let existing_transformation = e.get();

                        let existing_delta_x = (existing_transformation.start_local_vx as f32
                            - sample::GROUND_SPEED_GRID_FACTOR * e.key().local_vx as f32)
                            .abs();
                        let existing_delta_y = (existing_transformation.start_local_vy as f32
                            - sample::GROUND_SPEED_GRID_FACTOR * e.key().local_vy as f32)
                            .abs();
                        let existing_delta =
                            (existing_delta_x.powf(2.0) + existing_delta_y.powf(2.0)).sqrt();

                        let new_lv = sample[j].local_velocity();
                        let new_delta_x =
                            (new_lv.x - sample::GROUND_SPEED_GRID_FACTOR * e.key().local_vx as f32).abs();
                        let new_delta_y =
                            (new_lv.y - sample::GROUND_SPEED_GRID_FACTOR * e.key().local_vy as f32).abs();
                        let new_delta = (new_delta_x.powf(2.0) + new_delta_y.powf(2.0)).sqrt();

                        new_delta < existing_delta
                    };

                    if should_replace {
                        e.insert(PlayerTransformation::from_samples(&all_samples[i][j..], num_ticks));
                    }
                }
            };
            j += 1;
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let control_branches = [
        "throttle_straight",
        "throttle_right",
        "throttle_left",
        "throttle_straight_drift",
        "throttle_right_drift",
        "throttle_left_drift",

        "boost_straight",
        "boost_right",
        "boost_left",
        "boost_straight_drift",
        "boost_right_drift",
        "boost_left_drift",

        "idle_straight",
        "idle_right",
        "idle_left",
        "idle_straight_drift",
        "idle_right_drift",
        "idle_left_drift",

        "reverse_straight",
        "reverse_right",
        "reverse_left",
        "reverse_straight_drift",
        "reverse_right_drift",
        "reverse_left_drift",
    ];

    let base_path = Path::new("./models/flat_ground");
    create_dir_all(&base_path)?;
    for control_branch in control_branches.iter() {
        let model = build_model_for(control_branch);
        let filename = format!("{}.bincode.gz", control_branch);
        write_model(&base_path.join(filename), model)?;
    }

    Ok(())

}
