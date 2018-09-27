extern crate rlbot;
extern crate state;
extern crate csv;
extern crate passthrough;

use rlbot::ffi::{ MatchSettings, LiveDataPacket, PlayerInput };
use state::*;
use std::thread;
use std::time::Duration;
use std::error::Error;
use passthrough::{Gilrs, Gamepad, human_input, update_gamepad};

struct RecordState {
    min_speed: f32,
    started: bool,
    records: Vec<(f32, PlayerState)>,
}

impl RecordState {
    pub fn save_and_advance(&mut self) {
        let path = format!("./throttle_left/{}.csv", self.min_speed);
        let mut wtr = csv::Writer::from_path(path).expect("couldn't open file for writing csv");

        for (t, player) in &self.records {
            let pos = player.position;
            let vel = player.velocity;
            let avel = player.angular_velocity;
            let (roll, pitch, yaw) = player.rotation.to_euler_angles();
            let row = [
                *t,
                pos.x, pos.y, pos.z,
                vel.x, vel.y, vel.z,
                avel.x, avel.y, avel.z,
                roll, pitch, yaw,
            ].iter().map(|x| x.to_string()).collect::<Vec<_>>();
            wtr.write_record(&row).expect("csv write failed");
        }

        self.records.clear();
        self.min_speed += 100.0;
    }
}

const MAX_BOOST_SPEED: f32 = 2300.0;

fn bot_input(packet: &LiveDataPacket, record_state: &mut RecordState) -> PlayerInput {
    let mut game_state = GameState::default();
    state::update_game_state(&mut game_state, &packet, 0);

    let mut input = PlayerInput::default();
    input.Throttle = 1.0;

    if !record_state.started {
        if game_state.player.velocity.norm() > record_state.min_speed {
            record_state.started = true;
        } else if record_state.min_speed > 1500.0 {
            input.Boost = true;
        }
    }

    if record_state.started {
        input.Steer = -1.0;
    }

    // record player
    record_state.records.push((packet.GameInfo.TimeSeconds, game_state.player.clone()));

    input
}

fn main() -> Result<(), Box<Error>> {
    let mut packet = LiveDataPacket::default();
    let mut record_state = RecordState {
        min_speed: 0.0,
        started: false,
        records: vec![],
    };
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();


    let rlbot = rlbot::init()?;
    let mut settings = MatchSettings {
        NumPlayers: 1,
        ..Default::default()
    };

    settings.PlayerConfiguration[0].Bot = true;
    settings.PlayerConfiguration[0].RLBotControlled = true;
    settings.PlayerConfiguration[0].set_name("Recorder");

    rlbot.start_match(settings)?;

    loop {
        rlbot.update_live_data_packet(&mut packet)?;
        //println!("{}: {:?}", count, packet.GameBall);

        update_gamepad(&mut gilrs, &mut gamepad);
        let input = if gamepad.select_toggled {
            bot_input(&packet, &mut record_state)
        } else {
            human_input(&gamepad)
        };

        if record_state.records.len() > 2000 {
            gamepad.select_toggled = false;
            record_state.save_and_advance();
            record_state.started = false;

            if record_state.min_speed > MAX_BOOST_SPEED {
                break;
            };
        }

        rlbot.update_player_input(input, 0)?;
        thread::sleep(Duration::from_millis(1000/250));
    }

    Ok(())
}
