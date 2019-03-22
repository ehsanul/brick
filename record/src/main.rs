extern crate csv;
extern crate flatbuffers;
extern crate rlbot;
extern crate state;
use rlbot::{ ffi, flat };
use state::*;
use std::error::Error;
use std::f32::consts::PI;

const MAX_BOOST_SPEED: i16 = 2300;
const MAX_ANGULAR_SPEED: i16 = 6; // TODO check
const ANGULAR_GRID: f32 = 0.2;

struct RecordState {
    speed: i16,
    angular_speed: i16,
    started: bool,
    records: Vec<(i32, PlayerState)>,
    name: &'static str,
}

impl RecordState {
    pub fn record(&mut self, tick: &flat::RigidBodyTick) {
        let mut game_state = GameState::default();
        state::update_game_state(&mut game_state, &tick, 0);

        if !self.started {
            if self.is_initial_state(&game_state) {
                self.started = true
            } else {
                return;
            }
        }

        let latest = (game_state.frame, game_state.player.clone());
        self.records.push(latest);
    }

    fn is_initial_state(&self, game_state: &GameState) -> bool {
        let pos = game_state.player.position;
        pos.x.abs() < 100.0 && pos.y.abs() < 100.0
    }

    pub fn save_and_advance(&mut self) {
        let path = format!("data/samples/flat_ground/{}/{}_{}.csv", self.name, self.speed, self.angular_speed);
        let mut wtr = csv::Writer::from_path(path).expect("couldn't open file for writing csv");

        for (frame, player) in &self.records {
            let pos = player.position;
            let vel = player.velocity;
            let avel = player.angular_velocity;
            let (roll, pitch, yaw) = player.rotation.euler_angles();

            #[rustfmt::skip]
            let row = [
                *frame as f32,
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
                z: Some(&flat::Float::new(18.65)), // batmobile resting z
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
                z: Some(&flat::Float::new(self.angular_speed as f32 * ANGULAR_GRID)),
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
        self.angular_speed > MAX_ANGULAR_SPEED
    }
}

fn wait_for_match_start(rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
    // `packets.next()` sleeps until the next packet is available,
    // so this loop will not roast your CPU :)
    let mut packets = rlbot.packeteer();
    while !packets.next()?.GameInfo.RoundActive {};
    Ok(())
}

fn move_ball_out_of_the_way(rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

    let position = flat::Vector3Partial::create(
        &mut builder,
        &flat::Vector3PartialArgs {
            x: Some(&flat::Float::new(3800.0)),
            y: Some(&flat::Float::new(4800.0)),
            z: Some(&flat::Float::new(98.0)),
        },
    );

    let physics = flat::DesiredPhysics::create(
        &mut builder,
        &flat::DesiredPhysicsArgs {
            location: Some(position),
            ..Default::default()
        },
    );

    let ball_state = flat::DesiredBallState::create(
        &mut builder,
        &flat::DesiredBallStateArgs {
            physics: Some(physics),
            ..Default::default()
        },
    );

    let desired_game_state = flat::DesiredGameState::create(
        &mut builder,
        &flat::DesiredGameStateArgs {
            ballState: Some(ball_state),
            ..Default::default()
        },
    );

    builder.finish(desired_game_state, None);
    rlbot.set_game_state(&builder.finished_data())?;

    Ok(())
}

fn record_set(rlbot: &rlbot::RLBot, name: &'static str, input: ffi::PlayerInput) -> Result<(), Box<Error>> {
    let mut record_state = RecordState {
        speed: -MAX_BOOST_SPEED,
        angular_speed: (1.0 / ANGULAR_GRID).round() as i16 * -MAX_ANGULAR_SPEED,
        started: false,
        records: vec![],
        name: name,
    };

    record_state.set_next_game_state(&rlbot)?;
    let mut physicist = rlbot.physicist();
    loop {
        let tick = physicist.next_flat()?;

        record_state.record(&tick);
        if record_state.sample_complete() {
            record_state.save_and_advance();
            if record_state.all_samples_complete() {
                break;
            } else {
                record_state.set_next_game_state(&rlbot)?;
            }
        }

        rlbot.update_player_input(input, 0)?;
    }

    Ok(())
}

fn throttle_straight() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input
}

fn throttle_left() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input.Steer = -1.0;
    input
}

fn throttle_right() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input.Steer = 1.0;
    input
}

fn throttle_straight_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input.Handbrake = true;
    input
}

fn throttle_left_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input.Handbrake = true;
    input.Steer = -1.0;
    input
}

fn throttle_right_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = 1.0;
    input.Handbrake = true;
    input.Steer = 1.0;
    input
}

fn boost_straight() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input
}

fn boost_left() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input.Steer = -1.0;
    input
}

fn boost_right() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input.Steer = 1.0;
    input
}

fn boost_straight_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input.Handbrake = true;
    input
}

fn boost_left_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input.Handbrake = true;
    input.Steer = -1.0;
    input
}

fn boost_right_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Boost = true;
    input.Handbrake = true;
    input.Steer = 1.0;
    input
}

fn brake_straight() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input
}

fn brake_left() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input.Steer = -1.0;
    input
}

fn brake_right() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input.Steer = 1.0;
    input
}

fn brake_straight_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input.Handbrake = true;
    input
}

fn brake_left_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input.Handbrake = true;
    input.Steer = -1.0;
    input
}

fn brake_right_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Throttle = -1.0;
    input.Handbrake = true;
    input.Steer = 1.0;
    input
}

fn idle_straight() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input
}

fn idle_left() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Steer = -1.0;
    input
}

fn idle_right() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Steer = 1.0;
    input
}

fn idle_straight_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Handbrake = true;
    input
}

fn idle_left_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Handbrake = true;
    input.Steer = -1.0;
    input
}

fn idle_right_drift() -> ffi::PlayerInput {
    let mut input = ffi::PlayerInput::default();
    input.Handbrake = true;
    input.Steer = 1.0;
    input
}


fn main() -> Result<(), Box<Error>> {
    let rlbot = rlbot::init()?;
    let mut settings = ffi::MatchSettings {
        NumPlayers: 1,
        ..Default::default()
    };

    settings.PlayerConfiguration[0].Bot = true;
    settings.PlayerConfiguration[0].RLBotControlled = true;
    settings.PlayerConfiguration[0].set_name("Recorder");

    settings.MutatorSettings = ffi::MutatorSettings {
        MatchLength: ffi::MatchLength::Unlimited,
        ..Default::default()
    };

    rlbot.start_match(settings)?;
    wait_for_match_start(&rlbot)?;

    // set initial state
    move_ball_out_of_the_way(&rlbot)?;

    record_set(&rlbot, "throttle_straight", throttle_straight())?;
    record_set(&rlbot, "throttle_left", throttle_left())?;
    record_set(&rlbot, "throttle_right", throttle_right())?;
    record_set(&rlbot, "throttle_straight_drift", throttle_straight_drift())?;
    record_set(&rlbot, "throttle_left_drift", throttle_left_drift())?;
    record_set(&rlbot, "throttle_right_drift", throttle_right_drift())?;
    record_set(&rlbot, "boost_straight", boost_straight())?;
    record_set(&rlbot, "boost_left", boost_left())?;
    record_set(&rlbot, "boost_right", boost_right())?;
    record_set(&rlbot, "boost_straight_drift", boost_straight_drift())?;
    record_set(&rlbot, "boost_left_drift", boost_left_drift())?;
    record_set(&rlbot, "boost_right_drift", boost_right_drift())?;
    record_set(&rlbot, "brake_straight", brake_straight())?;
    record_set(&rlbot, "brake_left", brake_left())?;
    record_set(&rlbot, "brake_right", brake_right())?;
    record_set(&rlbot, "brake_straight_drift", brake_straight_drift())?;
    record_set(&rlbot, "brake_left_drift", brake_left_drift())?;
    record_set(&rlbot, "brake_right_drift", brake_right_drift())?;
    record_set(&rlbot, "idle_straight", idle_straight())?;
    record_set(&rlbot, "idle_left", idle_left())?;
    record_set(&rlbot, "idle_right", idle_right())?;
    record_set(&rlbot, "idle_straight_drift", idle_straight_drift())?;
    record_set(&rlbot, "idle_left_drift", idle_left_drift())?;
    record_set(&rlbot, "idle_right_drift", idle_right_drift())?;

    Ok(())
}
