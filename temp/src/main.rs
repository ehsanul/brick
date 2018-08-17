extern crate rlbot;
extern crate state;
extern crate csv;
extern crate passthrough;

use rlbot::*;
use state::*;
use std::thread;
use passthrough::{Gilrs, Gamepad, human_input, update_gamepad};

fn bot_input(packet: &LiveDataPacket, records: &mut Vec<(f32, PlayerState)>) -> PlayerInput {
    let mut input = PlayerInput::default();
    input.Throttle = 1.0;
    input.Steer = -1.0;

    // record player
    let mut game_state = GameState::default();
    update_game_state(&mut game_state, &packet, 0);
    records.push((packet.GameInfo.TimeSeconds, game_state.player.clone()));

    input
}

fn write_records(records: &Vec<(f32, PlayerState)>) {
    let mut wtr = csv::Writer::from_path("./throttle_left.csv").expect("couldn't open file for writing csv");

    for (t, player) in records {
        let pos = player.position;
        let vel = player.velocity;
        let (roll, pitch, yaw) = player.rotation.to_euler_angles();
        wtr.write_record(&[*t, pos.x, pos.y, pos.z, vel.x, vel.y, vel.z, roll, pitch, yaw].iter().map(|x| x.to_string()).collect::<Vec<_>>()).expect("csv write failed");
    }
}

fn main() {
    let mut packet = LiveDataPacket::default();
    let mut records = vec![];
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();

    loop {
        rlbot::update_live_data_packet(&mut packet);
        //println!("{}: {:?}", count, packet.GameBall);

        update_gamepad(&mut gilrs, &mut gamepad);
        let input = if gamepad.select_toggled {
            bot_input(&packet, &mut records)
        } else {
            human_input(&gamepad)
        };

        if records.len() > 2000 {
            gamepad.select_toggled = false;
            write_records(&records);
            records.clear();
        }

        rlbot::update_player_input(input, 0);
        thread::sleep_ms(1000/500);
    }
}
