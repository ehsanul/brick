extern crate csv;
extern crate brain;
extern crate state;
extern crate nalgebra as na;

use std::error::Error;
use brain::predict::sample;
use std::fs::create_dir_all;
use std::path::Path;
use na::{Quaternion, UnitQuaternion};

fn main() -> Result<(), Box<dyn Error>> {
    for f in sample::csv_files("./data/samples/flat_ground") {
        let sample = sample::load_sample_file(&f);

        println!("{}", f.to_string_lossy());
        let new_path = f.to_string_lossy().replace("flat_ground", "flat_ground_new");
        create_dir_all(&Path::new(&new_path).parent().expect("no parent for this f?"))?;
        let mut wtr =
            csv::Writer::from_path(new_path).expect("couldn't open file for writing csv");

        for player in sample {
            let pos = player.position;
            let vel = player.velocity;
            let avel = player.angular_velocity;
            let rlq = player.rotation.quaternion().coords;
            let translated = UnitQuaternion::from_quaternion(Quaternion::new(rlq[3], rlq[0], -rlq[1], -rlq[2]));
            let (roll, pitch, yaw) = translated.euler_angles();

            #[rustfmt::skip]
            let row = [
                avel.x, // temporary hack to make use of this to store the frame, as it's normally just zero for ground driving
                pos.x, pos.y, pos.z,
                vel.x, vel.y, vel.z,
                0.0, avel.y, avel.z,
                roll, pitch, yaw,
            ].iter().map(|x| x.to_string()).collect::<Vec<_>>();

            wtr.write_record(&row).expect("csv write failed");
        }
    }

    Ok(())
}
