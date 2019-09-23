extern crate csv;
extern crate flatbuffers;
extern crate predict;
extern crate rlbot;
extern crate state;
use rlbot::{flat, ControllerState};
use state::*;
use std::collections::HashMap;
use std::error::Error;
use std::f32::consts::PI;
use std::fs::create_dir_all;
use std::path::PathBuf;

const MAX_BOOST_SPEED: i16 = 2300;
const MAX_ANGULAR_SPEED: i16 = 6; // TODO check
const ANGULAR_GRID: f32 = 0.2;
const SPEED_GRID: i16 = 100;
const VELOCITY_MARGIN: f32 = 15.0;
const ANGULAR_SPEED_MARGIN: f32 = 0.5;

#[derive(Debug, Clone)]
struct MaxAttempts {
    local_vx: i16,
    local_vy: i16,
    angular_speed: i16,
}

impl std::fmt::Display for MaxAttempts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Failed to set game state accurately for: {}, {}, {}",
            self.local_vx, self.local_vy, self.angular_speed
        )
    }
}

impl Error for MaxAttempts {
    fn cause(&self) -> Option<&Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

struct RecordState {
    local_vx: i16,
    local_vy: i16,
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

    pub fn path(&self) -> String {
        let dir = format!("./data/samples/flat_ground/{}", self.name);
        create_dir_all(&dir).unwrap();
        format!(
            "{}/{}_{}_{}.csv",
            dir, self.local_vx, self.local_vy, self.angular_speed
        )
    }

    pub fn save(&mut self) {
        let mut wtr =
            csv::Writer::from_path(self.path()).expect("couldn't open file for writing csv");

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
    }

    pub fn advance(&mut self) {
        self.records.clear();
        self.local_vx += 100;
        if self.local_vx > MAX_BOOST_SPEED {
            self.local_vx = -MAX_BOOST_SPEED;
            self.local_vy += 100;
            if self.local_vy > MAX_BOOST_SPEED {
                self.local_vy = -MAX_BOOST_SPEED;
                self.angular_speed += 1;
            }
        }
    }

    pub fn set_next_game_state(&mut self, rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
        self.started = false;

        let position = rlbot::Vector3Partial::new().x(0.0).y(0.0).z(18.65); // batmobile resting z

        let velocity = rlbot::Vector3Partial::new()
            .x(-self.local_vx as f32)
            .y(self.local_vy as f32)
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

        let car_state = rlbot::DesiredCarState::new().physics(physics);

        let desired_game_state = rlbot::DesiredGameState::new().car_state(0, car_state);

        rlbot.set_game_state(&desired_game_state)?;

        Ok(())
    }

    pub fn reset_game_state(&mut self, rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
        let position = rlbot::Vector3Partial::new().x(-2000.0).y(2000.0).z(18.65); // batmobile resting z

        let velocity = rlbot::Vector3Partial::new()
            .x(0.0)
            .y(0.0)
            .z(0.0);

        let angular_velocity = rlbot::Vector3Partial::new()
            .x(0.0)
            .y(0.0)
            .z(0.0);

        let rotation = rlbot::RotatorPartial::new()
            .pitch(0.0)
            .yaw(PI / 2.0)
            .roll(0.0);

        let physics = rlbot::DesiredPhysics::new()
            .location(position)
            .rotation(rotation)
            .velocity(velocity)
            .angular_velocity(angular_velocity);

        let car_state = rlbot::DesiredCarState::new().physics(physics);

        let desired_game_state = rlbot::DesiredGameState::new().car_state(0, car_state);

        rlbot.set_game_state(&desired_game_state)?;

        Ok(())
    }

    pub fn set_game_state_accurately(
        &mut self,
        rlbot: &rlbot::RLBot,
        physicist: &mut rlbot::Physicist,
        index: &mut HashMap<predict::sample::NormalizedPlayerState, PlayerState>,
        adjustment: &mut Adjustment,
    ) -> Result<(), Box<Error>> {
        let original_local_vx = self.local_vx;
        let original_local_vy = self.local_vy;
        let original_angular_speed = self.angular_speed;

        let mut attempts = 0;

        loop {
            // adjust values to account for differences between the value we set and the first
            // value we actually receive back
            self.local_vx = original_local_vx
                + adjustment
                    .local_vx
                    .get(&original_angular_speed)
                    .map(|e| e.round() as i16)
                    .unwrap_or(0i16);
            self.local_vy = original_local_vy
                + adjustment
                    .local_vy
                    .get(&original_angular_speed)
                    .map(|e| e.round() as i16)
                    .unwrap_or(0i16);
            self.angular_speed = original_angular_speed
                + adjustment
                    .angular_speed
                    .get(&original_angular_speed)
                    .map(|e| e.round() as i16)
                    .unwrap_or(0i16);
            println!("local_vx: {}, local_vy: {}, angular_speed: {}", self.local_vx, self.local_vy, self.angular_speed);

            self.set_next_game_state(rlbot)?;

            // check if we match the expected state now
            'inner: loop {
                if attempts > 5 {
                    // we tried, but now bail
                    adjustment
                        .local_vx
                        .entry(original_angular_speed)
                        .and_modify(|e| *e = 0f32);
                    adjustment
                        .local_vy
                        .entry(original_angular_speed)
                        .and_modify(|e| *e = 0f32);
                    adjustment
                        .angular_speed
                        .entry(original_angular_speed)
                        .and_modify(|e| *e = 0f32);
                    return Err(MaxAttempts {
                        local_vx: original_local_vx,
                        local_vy: original_local_vy,
                        angular_speed: original_angular_speed,
                    }
                    .into());
                }

                let tick = physicist.next_flat()?;
                let mut game_state = GameState::default();
                state::update_game_state(&mut game_state, &tick, 0);

                if !self.is_initial_state(&game_state) {
                    // there's a delay between setting state and it become available in the tick
                    // data. let's try again
                    continue 'inner;
                }

                let vx_diff = original_local_vx as f32 - game_state.player.local_velocity().x;
                let vy_diff = original_local_vy as f32 - game_state.player.local_velocity().y;
                let avz_diff = original_angular_speed as f32
                    - (game_state.player.angular_velocity.z / ANGULAR_GRID);
                println!("game local vx: {}, game local vy: {}, game avz: {}", game_state.player.local_velocity().x, game_state.player.local_velocity().y, (game_state.player.angular_velocity.z / ANGULAR_GRID));

                if vx_diff.abs() <= VELOCITY_MARGIN
                    && vy_diff.abs() <= VELOCITY_MARGIN
                    && avz_diff.abs() < ANGULAR_SPEED_MARGIN
                {
                    // close enough, we're good!
                    // XXX must record now since borrowck doesn't understand that ticket is an
                    // independent value that shouldn't extend the lifetime of the record_state
                    // borrow. same issue with index.insert.
                    self.record(&tick);
                    index.insert(
                        predict::sample::normalized_player_rounded(&game_state.player),
                        game_state.player.clone(),
                    );

                    return Ok(());
                } else {
                    adjustment
                        .local_vx
                        .entry(original_angular_speed)
                        .and_modify(|e| *e += vx_diff)
                        .or_insert(vx_diff);
                    adjustment
                        .local_vy
                        .entry(original_angular_speed)
                        .and_modify(|e| *e += vy_diff)
                        .or_insert(vy_diff);
                    adjustment
                        .angular_speed
                        .entry(original_angular_speed)
                        .and_modify(|e| *e += avz_diff)
                        .or_insert(avz_diff);
                    attempts += 1;

                    self.reset_game_state(&rlbot)?;
                    // wait till we're far, so is_initial_state works after this
                    'inner2: loop {
                        let tick = physicist.next_flat()?;
                        let mut game_state = GameState::default();
                        state::update_game_state(&mut game_state, &tick, 0);

                        if (game_state.player.position.x - 2000.0).abs() < 200.0 {
                            break 'inner2;
                        }
                    }

                    break 'inner;
                }
            }
        }
    }

    pub fn sample_complete(&self) -> bool {
        self.records.len() >= predict::sample::MIN_SAMPLE_LENGTH
    }

    pub fn sample_valid(&self) -> bool {
        let mut last_player = &self.records[0].1;

        // there must be two physics ticks between each measurement for the sample to be valid as
        // a whole, given a 60fps record rate. it's 60fps by default apparently unless something is
        // done. in practice, i found that sometimes records would be 1 tick or 3 ticks apart, once
        // in a while, which messes up the sample and this this is now validated
        assert!(predict::sample::RECORD_FPS == 60);
        self.records[1..].iter().all(|(_frame, player)| {
            let v = 0.5 * (player.velocity + last_player.velocity);
            let d = (player.position - last_player.position).norm();
            let physics_ticks = (FPS * d / v.norm()).round() as i32;
            last_player = player;
            physics_ticks == 2
        })
    }

    // angular speed is the outer loop, so we're done when that's done
    pub fn all_samples_complete(&self) -> bool {
        self.angular_speed > (1.0 / ANGULAR_GRID).round() as i16 * MAX_ANGULAR_SPEED
    }
}

#[derive(Default)]
struct Adjustment {
    local_vx: HashMap<i16, f32>,
    local_vy: HashMap<i16, f32>,
    angular_speed: HashMap<i16, f32>,
}

fn move_ball_out_of_the_way(rlbot: &rlbot::RLBot) -> Result<(), Box<Error>> {
    let position = rlbot::Vector3Partial::new().x(3800.0).y(4800.0).z(98.0);

    let physics = rlbot::DesiredPhysics::new().location(position);

    let ball_state = rlbot::DesiredBallState::new().physics(physics);

    let desired_game_state = rlbot::DesiredGameState::new().ball_state(ball_state);

    rlbot.set_game_state(&desired_game_state)?;

    Ok(())
}

fn _record_set(
    rlbot: &rlbot::RLBot,
    name: &'static str,
    input: ControllerState,
) -> Result<(), Box<Error>> {
    let mut record_state = RecordState {
        local_vx: -MAX_BOOST_SPEED,
        local_vy: -MAX_BOOST_SPEED,
        angular_speed: (1.0 / ANGULAR_GRID).round() as i16 * -MAX_ANGULAR_SPEED,
        started: false,
        records: vec![],
        name: name,
    };

    record_state.set_next_game_state(&rlbot)?;
    let mut physicist = rlbot.physicist();
    loop {
        // skip unreachable velocities
        if record_state.local_vx.pow(2) + record_state.local_vy.pow(2) > MAX_BOOST_SPEED.pow(2) {
            record_state.advance();
            continue;
        }

        while PathBuf::from(&record_state.path()).exists() {
            record_state.advance();
            continue;
        }

        let tick = physicist.next_flat()?;

        record_state.record(&tick);
        if record_state.sample_complete() {
            record_state.save();
            record_state.advance();
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

fn record_all_missing(
    rlbot: &rlbot::RLBot,
    name: &'static str,
    input: ControllerState,
    sample_index: &predict::sample::SampleMap<'static>,
) -> Result<(), Box<Error>> {
    let mut record_state = RecordState {
        local_vx: 0,
        local_vy: 0,
        angular_speed: 0,
        started: false,
        records: vec![],
        name: name,
    };

    let mut adjustment = Adjustment::default();

    // so we can insert newly created values in and skip when we've pre-emptively recorded
    // we're cloning to avoid some complicated lifetime issues, since our SampleMap currently
    // relies on static vecs, whereas we are dealing with mutable vecs right now and that means the
    // pointer to the vec could be invalidated, which means we can't really a slice of the vec as
    // the value, like we do with SampleMap
    let mut index = HashMap::default();
    for (key, val) in sample_index.iter() {
        index.insert(key.clone(), val[0].clone());
    }

    let min_avz = -(MAX_ANGULAR_SPEED as f32 / ANGULAR_GRID).round() as i16;
    let max_avz = (MAX_ANGULAR_SPEED as f32 / ANGULAR_GRID).round() as i16;
    let local_vx = 0; // TODO loop over these too
    for local_vy in 0..(MAX_BOOST_SPEED / SPEED_GRID) {
        // TODO negative vy
        for avz in min_avz..max_avz {
            let normalized = predict::sample::NormalizedPlayerState {
                local_vy: local_vy,
                local_vx: 0,
                avz,
            };

            if let Some(player) = index.get(&normalized) {
                // sample was found.
                // check if the sample is within our acceptable margin of closeness to the
                // actual valus we want, and if so, skip
                let vx_diff = 100.0 * local_vx as f32 - player.local_velocity().x;
                let vy_diff = 100.0 * local_vy as f32 - player.local_velocity().y;
                let avz_diff = avz as f32 - (player.angular_velocity.z / ANGULAR_GRID);

                if vx_diff.abs() <= VELOCITY_MARGIN
                    && vy_diff.abs() <= VELOCITY_MARGIN
                    && avz_diff.abs() < ANGULAR_SPEED_MARGIN
                {
                    continue;
                }
            }

            // no sample found, or no sample within our margin, so let's get it!
            record_state.local_vx = local_vx * 100;
            record_state.local_vy = local_vy * 100;
            record_state.angular_speed = avz;
            if let Err(e) = record_missing_record_state(
                &rlbot,
                &input,
                &mut index,
                &mut record_state,
                &mut adjustment,
            ) {
                println!("Error recording missing record state: {}", e);
            }
        }
    }

    Ok(())
}

fn record_missing_record_state<'a>(
    rlbot: &rlbot::RLBot,
    input: &ControllerState,
    index: &mut HashMap<predict::sample::NormalizedPlayerState, PlayerState>,
    record_state: &mut RecordState,
    adjustment: &mut Adjustment,
) -> Result<(), Box<Error>> {
    record_state.records.clear();
    let mut physicist = rlbot.physicist();
    rlbot.update_player_input(0, &input)?;

    loop {
        // waits and checks the tick to ensure it meets our conditions. and it records the first tick
        record_state.set_game_state_accurately(&rlbot, &mut physicist, index, adjustment)?;

        loop {
            //rlbot.update_player_input(0, &input)?;
            let tick = physicist.next_flat()?;
            record_state.record(&tick);

            if record_state.sample_complete() {
                break;
            }
        }

        if record_state.sample_valid() {
            record_state.save();
            break;
        } else {
            println!("invalid sample, retrying");
            record_state.records.clear();
        }
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

fn reverse_straight() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input
}

fn reverse_left() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.steer = -1.0;
    input
}

fn reverse_right() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.steer = 1.0;
    input
}

fn reverse_straight_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.handbrake = true;
    input
}

fn reverse_left_drift() -> ControllerState {
    let mut input = ControllerState::default();
    input.throttle = -1.0;
    input.handbrake = true;
    input.steer = -1.0;
    input
}

fn reverse_right_drift() -> ControllerState {
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

    let batmobile = rlbot::PlayerLoadout::new().car_id(803);
    let mut settings =
        rlbot::MatchSettings::new().player_configurations(vec![rlbot::PlayerConfiguration::new(
            rlbot::PlayerClass::RLBotPlayer,
            "Recorder",
            0,
        )
        .loadout(batmobile)]);

    settings.mutator_settings =
        rlbot::MutatorSettings::new().
        match_length(rlbot::MatchLength::Unlimited).
        boost_option(rlbot::BoostOption::Unlimited_Boost);

    rlbot.start_match(&settings)?;
    rlbot.wait_for_match_start()?;

    // set initial state
    move_ball_out_of_the_way(&rlbot)?;

    record_all_missing(
        &rlbot,
        "throttle_straight",
        throttle_straight(),
        &predict::sample::THROTTLE_STRAIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "throttle_left",
        throttle_left(),
        &predict::sample::THROTTLE_LEFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "throttle_right",
        throttle_right(),
        &predict::sample::THROTTLE_RIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "throttle_straight_drift",
        throttle_straight_drift(),
        &predict::sample::THROTTLE_STRAIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "throttle_left_drift",
        throttle_left_drift(),
        &predict::sample::THROTTLE_LEFT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "throttle_right_drift",
        throttle_right_drift(),
        &predict::sample::THROTTLE_RIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_straight",
        boost_straight(),
        &predict::sample::BOOST_STRAIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_left",
        boost_left(),
        &predict::sample::BOOST_LEFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_right",
        boost_right(),
        &predict::sample::BOOST_RIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_straight_drift",
        boost_straight_drift(),
        &predict::sample::BOOST_STRAIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_left_drift",
        boost_left_drift(),
        &predict::sample::BOOST_LEFT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "boost_right_drift",
        boost_right_drift(),
        &predict::sample::BOOST_RIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_straight",
        reverse_straight(),
        &predict::sample::REVERSE_STRAIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_left",
        reverse_left(),
        &predict::sample::REVERSE_LEFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_right",
        reverse_right(),
        &predict::sample::REVERSE_RIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_straight_drift",
        reverse_straight_drift(),
        &predict::sample::REVERSE_STRAIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_left_drift",
        reverse_left_drift(),
        &predict::sample::REVERSE_LEFT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "reverse_right_drift",
        reverse_right_drift(),
        &predict::sample::REVERSE_RIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_straight",
        idle_straight(),
        &predict::sample::IDLE_STRAIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_left",
        idle_left(),
        &predict::sample::IDLE_LEFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_right",
        idle_right(),
        &predict::sample::IDLE_RIGHT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_straight_drift",
        idle_straight_drift(),
        &predict::sample::IDLE_STRAIGHT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_left_drift",
        idle_left_drift(),
        &predict::sample::IDLE_LEFT_DRIFT_INDEXED,
    )?;
    record_all_missing(
        &rlbot,
        "idle_right_drift",
        idle_right_drift(),
        &predict::sample::IDLE_RIGHT_DRIFT_INDEXED,
    )?;

    Ok(())
}
