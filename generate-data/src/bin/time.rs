extern crate bincode;
extern crate brain;
extern crate csv;
extern crate flate2;
extern crate nalgebra as na;
extern crate rand;
extern crate state;
extern crate walkdir;

use bincode::deserialize_from;
use flate2::read::GzDecoder;
use rand::prelude::*;
use state::*;
use walkdir::{DirEntry, WalkDir};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn files<'a>(dir: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                Some(entry.path().to_owned())
            } else {
                None
            }
        })
}

fn load_plan(path: &PathBuf) -> Result<Plan, Box<Error>> {
    let f = BufReader::new(File::open(path)?);
    let mut decoder = GzDecoder::new(f);
    Ok(deserialize_from(&mut decoder)?)
}

fn get_row(plan: &Plan, i: usize, row: &mut Vec<String>) {
    let total_cost: f32 = plan[i..].iter().map(|(_, _, cost)| cost).sum();
    let player = plan[i].0;
    let pos = player.position;
    let vel = player.velocity;
    let avel = player.angular_velocity;
    let (roll, pitch, yaw) = player.rotation.euler_angles();

    row.extend(
        [
            total_cost, pos.x, pos.y, pos.z, vel.x, vel.y, vel.z, avel.x, avel.y, avel.z, roll,
            pitch, yaw,
        ]
        .iter()
        .map(|x| x.to_string()),
    );
}

fn main() -> Result<(), Box<Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];
    let output_path = &args[2];
    let mut wtr = csv::Writer::from_path(output_path)?;
    let mut rng = rand::thread_rng();
    let mut row = vec![];

    for path in files(dir) {
        let plan = load_plan(&path)?;

        row.clear();
        get_row(&plan, 0, &mut row);
        wtr.write_record(&row)?;

        // explode
        let mut plan_result = PlanResult {
            plan: Some(plan),
            ..Default::default()
        };
        brain::plan::explode_plan(&mut plan_result);
        let plan = plan_result.plan.unwrap();

        // choose randomly out of the last 2% of the exploded plan, since we are doing bad on that bit
        let rand_i: usize =
            (plan.len() as f32 * 0.98 + 0.02 * rng.gen::<f32>()).round() as usize - 1;
        row.clear();
        get_row(&plan, rand_i, &mut row);
        wtr.write_record(&row)?;
    }

    Ok(())
}
