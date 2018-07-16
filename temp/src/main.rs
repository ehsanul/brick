extern crate rlbot;
extern crate state;
extern crate gilrs;
extern crate csv;

use gilrs::{Gilrs, Button, Axis, Event, EventType};
use rlbot::*;
use state::*;
use std::thread;

static mut GAMEPAD_RT: f32 = 0.0;
static mut GAMEPAD_LS_X: f32 = 0.0;
static mut GAMEPAD_LS_Y: f32 = 0.0;
static mut BOT_ACTIVE: bool = false;


fn process_gamepad_events(gilrs: &mut Gilrs) {
    while let Some(Event { id, event, time }) = gilrs.next_event() {
        println!("{:?} New event from {}: {:?}", time, id, event);
        match event {
            EventType::ButtonChanged(button, value, _code) => {
                match button {
                    Button::RightTrigger2 => {
                        unsafe { GAMEPAD_RT = value; }
                    }
                    Button::LeftTrigger2 => {
                        unsafe { GAMEPAD_RT = -value; }
                    }
                    _ => {}
                }
            }
            EventType::ButtonPressed(button, _code) => {
                match button {
                    Button::Select => {
                        unsafe { BOT_ACTIVE = !BOT_ACTIVE; }
                    }
                    _ => {}
                }
            }
            EventType::AxisChanged(axis, value, _code) => {
                match axis {
                    Axis::LeftStickX => {
                        unsafe { GAMEPAD_LS_X = value; }
                    }
                    Axis::LeftStickY => {
                        unsafe { GAMEPAD_LS_Y = value; }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn human_input(gamepad: &gilrs::Gamepad) -> PlayerInput {
    let mut input = PlayerInput::default();
    input.Jump = gamepad.is_pressed(Button::South);
    input.Boost = gamepad.is_pressed(Button::East);
    input.Handbrake = gamepad.is_pressed(Button::West);
    unsafe {
        //  use of mutable static requires unsafe function or block
        input.Throttle = GAMEPAD_RT;
        input.Steer = GAMEPAD_LS_X;
        input.Yaw = GAMEPAD_LS_X;
        input.Pitch = -GAMEPAD_LS_Y;
    }

    input.Roll = if gamepad.is_pressed(Button::RightTrigger) {
        1.0
    } else if gamepad.is_pressed(Button::LeftTrigger) {
        -1.0
    } else {
        0.0
    };
    input
}

fn bot_input(packet: &LiveDataPacket, records: &mut Vec<(f32, PlayerState)>) -> PlayerInput {
    let mut input = PlayerInput::default();
    input.Throttle = 1.0;
    input.Steer = -1.0;

    // record player
    let mut game_state = GameState::default();
    update_game_state(&mut game_state, &packet, 0);
    records.push((packet.GameInfo.TimeSeconds, game_state.player.clone()));
    if records.len() > 2000 {
        unsafe { BOT_ACTIVE = false; }
        write_records(&records);
        records.clear();
    }

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
    let mut gilrs = Gilrs::new().unwrap();
    let mut records = vec![];

    loop {
        rlbot::update_live_data_packet(&mut packet);
        //println!("{}: {:?}", count, packet.GameBall);

        process_gamepad_events(&mut gilrs); // manipulates global state. sue me.
        let input = if unsafe{BOT_ACTIVE} { bot_input(&packet, &mut records) } else { human_input(&gilrs[0]) };

        rlbot::update_player_input(input, 0);
        thread::sleep_ms(1000/500);
    }
}
