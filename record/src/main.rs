extern crate csv;
extern crate flatbuffers;
extern crate rlbot;
extern crate state;
use rlbot::{ flat, ControllerState };
use std::fs::{ File, create_dir_all };
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
        let dir = format!("data/samples/flat_ground/{}", self.name);
        create_dir_all(&path).unwrap();

        let path = format!("{}/{}_{}.csv", sdir, self.speed, self.angular_speed);
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

        let position = rlbot::Vector3Partial::new()
            .x(0.0)
            .y(0.0)
            .z(18.65); // batmobile resting z

        let velocity = rlbot::Vector3Partial::new()
            .x(0.0)
            .y(self.speed as f32)
            .z(0.0);

        let angular_velocity = rlbot::Vector3Partial::new()
            .x(0.0)
            .y(0.0)
            .z(self.angular_speed as f32 * ANGULAR_GRID);

        let rotation = rlbot::RotatorPartial::new()
            .pitch(0.0)
            .yaw(PI / 2.0)
            .roll(0.0);

        let physics = rlbot::DesiredPhysics::new()
            .location(position)
            .rotation(rotation)
            .velocity(velocity)
            .angular_velocity(angular_velocity);

        let car_state = rlbot::DesiredCarState::new()
            .physics(physics);

        let desired_game_state = rlbot::DesiredGameState::new()
            .car_state(0, car_state);

        rlbot.set_game_state(&desired_game_state)?;

        Ok(())
    }

    pub fn sample_complete(&self) -> bool {
        self.records.len() > 120
    }

    pub fn all_samples_complete(&self) -> bool {
        self.angular_speed > MAX_ANGULAR_SPEED
    }
}

fn move_ball_out_of_the_way(rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
    let position = rlbot::Vector3Partial::new()
        .x(3800.0)
        .y(4800.0)
        .z(98.0);

    let physics = rlbot::DesiredPhysics::new()
        .location(position);

    let ball_state = rlbot::DesiredBallState::new()
        .physics(physics);

    let desired_game_state = rlbot::DesiredGameState::new()
        .ball_state(ball_state);

    rlbot.set_game_state(&desired_game_state)?;

    Ok(())
}

fn record_set(rlbot: &rlbot::RLBot, name: &'static str, input: ControllerState) -> Result<(), Box<Error>> {
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

        rlbot.update_player_input(0, &input)?;
    }

    Ok(())
}

fn throttle_straight() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input
}

fn throttle_left() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input.steer = -1.0;
    input
}

fn throttle_right() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input.steer = 1.0;
    input
}

fn throttle_straight_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input.handbrake = true;
    input
}

fn throttle_left_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input.handbrake = true;
    input.steer = -1.0;
    input
}

fn throttle_right_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = 1.0;
    input.handbrake = true;
    input.steer = 1.0;
    input
}

fn boost_straight() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input
}

fn boost_left() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input.steer = -1.0;
    input
}

fn boost_right() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input.steer = 1.0;
    input
}

fn boost_straight_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input.handbrake = true;
    input
}

fn boost_left_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input.handbrake = true;
    input.steer = -1.0;
    input
}

fn boost_right_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.boost = true;
    input.handbrake = true;
    input.steer = 1.0;
    input
}

fn brake_straight() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input
}

fn brake_left() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.steer = -1.0;
    input
}

fn brake_right() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.steer = 1.0;
    input
}

fn brake_straight_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.handbrake = true;
    input
}

fn brake_left_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.handbrake = true;
    input.steer = -1.0;
    input
}

fn brake_right_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.handbrake = true;
    input.steer = 1.0;
    input
}

fn idle_straight() -> ControllerState {
    ControllerState::default()
}

fn idle_left() -> ControllerState {
    let mut input = ControllerState::default();
    input.steer = -1.0;
    input
}

fn idle_right() -> ControllerState {
    let mut input = ControllerState::default();
    input.steer = 1.0;
    input
}

fn idle_straight_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.handbrake = true;
    input
}

fn idle_left_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.handbrake = true;
    input.steer = -1.0;
    input
}

fn idle_right_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.handbrake = true;
    input.steer = 1.0;
    input
}


fn main() -> Result<(), Box<Error>> {
    let rlbot = rlbot::init()?;
    let mut settings = rlbot::MatchSettings::new()
        .player_configurations(vec![
            rlbot::PlayerConfiguration::new(
                rlbot::PlayerClass::RLBotPlayer,
                "Recorder",
                0,
            )
        ]);

    settings.mutator_settings = rlbot::MutatorSettings::new()
        .match_length(rlbot::MatchLength::Unlimited);

    rlbot.start_match(&settings)?;
    rlbot.wait_for_match_start()?;

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
