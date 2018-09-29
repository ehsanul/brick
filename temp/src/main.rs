extern crate csv;
extern crate flatbuffers;
extern crate rlbot;
extern crate state;

use rlbot::ffi::{LiveDataPacket, MatchSettings, PlayerInput};
use rlbot::flat;
use state::*;
use std::error::Error;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

const MAX_BOOST_SPEED: i16 = 2300;
const MAX_ANGULAR_SPEED: i16 = 6; // TODO check

struct RecordState {
    speed: i16,
    angular_speed: i16,
    started: bool,
    records: Vec<(f32, PlayerState)>,
}

impl RecordState {
    pub fn record(&mut self, packet: &LiveDataPacket) {
        let mut game_state = GameState::default();
        state::update_game_state(&mut game_state, &packet, 0);
        if !self.is_started(&game_state) {
            return;
        }

        let latest = (packet.GameInfo.TimeSeconds, game_state.player.clone());
        if self.records.len() > 0 {
            let last = self.records[self.records.len() - 1];
            if last != latest {
                self.records.push(latest);
            }
        } else {
            self.records.push(latest);
        }
    }

    fn is_started(&self, game_state: &GameState) -> bool {
        let pos_threshold = 5.0;
        let speed_threshold = 0.1;
        let pos = game_state.player.position;
        let speed = game_state.player.velocity.norm();
        pos.x.abs() < pos_threshold && pos.y.abs() < pos_threshold && speed < speed_threshold
    }

    pub fn save_and_advance(&mut self) {
        let path = format!("./throttle_left/{}_{}.csv", self.speed, self.angular_speed);
        let mut wtr = csv::Writer::from_path(path).expect("couldn't open file for writing csv");

        for (t, player) in &self.records {
            let pos = player.position;
            let vel = player.velocity;
            let avel = player.angular_velocity;
            let (roll, pitch, yaw) = player.rotation.to_euler_angles();

            #[rustfmt_skip]
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
        self.speed += 100;
        if self.speed > MAX_BOOST_SPEED {
            self.speed = 0;
            self.angular_speed += 1;
        }
    }

    pub fn set_next_game_state(&mut self, rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
        self.started = false;

        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let mut car_offsets = Vec::with_capacity(1);

        let position = flat::Vector3Partial::create(
            &mut builder,
            &flat::Vector3PartialArgs {
                x: Some(&flat::Float::new(0.0)),
                y: Some(&flat::Float::new(0.0)),
                z: Some(&flat::Float::new(18.0)),
            },
        );

        let velocity = flat::Vector3Partial::create(
            &mut builder,
            &flat::Vector3PartialArgs {
                x: Some(&flat::Float::new(0.0)),
                y: Some(&flat::Float::new(self.speed as f32)),
                z: Some(&flat::Float::new(0.0)),
            },
        );

        let angular_velocity = flat::Vector3Partial::create(
            &mut builder,
            &flat::Vector3PartialArgs {
                x: Some(&flat::Float::new(0.0)),
                y: Some(&flat::Float::new(0.0)),
                z: Some(&flat::Float::new(self.angular_speed as f32)),
            },
        );

        let rotation = flat::RotatorPartial::create(
            &mut builder,
            &flat::RotatorPartialArgs {
                pitch: Some(&flat::Float::new(0.0)),
                yaw: Some(&flat::Float::new(PI / 2.0)),
                roll: Some(&flat::Float::new(0.0)),
            },
        );

        let physics = flat::DesiredPhysics::create(
            &mut builder,
            &flat::DesiredPhysicsArgs {
                location: Some(position),
                rotation: Some(rotation),
                velocity: Some(velocity),
                angularVelocity: Some(angular_velocity),
                ..Default::default()
            },
        );

        let car_state = flat::DesiredCarState::create(
            &mut builder,
            &flat::DesiredCarStateArgs {
                physics: Some(physics),
                ..Default::default()
            },
        );
        car_offsets.push(car_state);
        let car_states = builder.create_vector(&car_offsets);

        let desired_game_state = flat::DesiredGameState::create(
            &mut builder,
            &flat::DesiredGameStateArgs {
                carStates: Some(car_states),
                ..Default::default()
            },
        );

        builder.finish(desired_game_state, None);
        rlbot.set_game_state(&builder.finished_data())?;

        Ok(())
    }

    pub fn sample_complete(&self) -> bool {
        self.records.len() > 120
    }

    pub fn all_samples_complete(&self) -> bool {
        self.speed > MAX_BOOST_SPEED && self.angular_speed > MAX_ANGULAR_SPEED
    }
}

fn bot_input(_packet: &LiveDataPacket, _record_state: &mut RecordState) -> PlayerInput {
    let mut input = PlayerInput::default();
    input.Throttle = 1.0;
    input.Steer = -1.0;

    input
}

fn main() -> Result<(), Box<Error>> {
    let mut packet = LiveDataPacket::default();
    let mut record_state = RecordState {
        speed: 0,
        angular_speed: 5 * -MAX_ANGULAR_SPEED, // 5 is our scale-up factor used to round to the nearest integer
        started: false,
        records: vec![],
    };

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

        record_state.record(&packet);
        if record_state.sample_complete() {
            record_state.save_and_advance();

            if record_state.all_samples_complete() {
                break;
            } else {
                record_state.set_next_game_state(&rlbot)?;
            }
        }

        let input = bot_input(&packet, &mut record_state);
        rlbot.update_player_input(input, 0)?;
        thread::sleep(Duration::from_millis(1000 / 250));
    }

    Ok(())
}
