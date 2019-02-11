extern crate state;
extern crate brain;
extern crate nalgebra as na;
extern crate bincode;
extern crate flate2;
extern crate csv;
extern crate walkdir;

use state::*;
use bincode::deserialize_from;
use flate2::read::GzDecoder;
use walkdir::{ DirEntry, WalkDir };

use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::io::BufReader;
use std::error::Error;

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}


fn files<'a>(dir: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
    WalkDir::new(dir).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|entry| {
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

fn main() -> Result<(), Box<Error>> {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];
    let output_path = &args[2];
    let mut wtr = csv::Writer::from_path(output_path)?;

    for path in files(dir) {
        let plan = load_plan(&path)?;
        let total_cost: f32 = plan.iter().map(|(_, _, cost)| cost).sum();

        let player = plan[0].0;
        let pos = player.position;
        let vel = player.velocity;
        let avel = player.angular_velocity;
        let (roll, pitch, yaw) = player.rotation.euler_angles();

        let row = [
            total_cost,
            pos.x, pos.y, pos.z,
            vel.x, vel.y, vel.z,
            avel.x, avel.y, avel.z,
            roll, pitch, yaw,
        ].iter().map(|x| x.to_string()).collect::<Vec<_>>();

        wtr.write_record(&row)?;
    }
    Ok(())
}
