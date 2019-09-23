extern crate brain;

use std::error::Error;
use brain::predict::sample;
use std::fs;

fn main() -> Result<(), Box<Error>> {
    let mut count_good = 0;
    let mut count_bad = 0;

    for f in sample::csv_files("./data/samples/flat_ground") {
        let sample = sample::load_sample_file(&f);

        let mut last_player = &sample[0];
        let bad = sample[1..].iter().any(|player| {
            let v = 0.5 * (player.velocity + last_player.velocity);
            let d = (player.position - last_player.position).norm();
            let physics_ticks = (120.0 * d / v.norm()).round() as i32;
            last_player = player;
            physics_ticks != 2
        });

        if bad {
            count_bad += 1;
            fs::remove_file(f)?;
        } else {
            count_good += 1;
        }
    }

    println!("good: {}, bad: {}", count_good, count_bad);

    Ok(())
}
