const USAGE: &'static str = "
Brick

Usage:
  brick --bot
  brick --bot-test
  brick --simulate

Options:
  -h --help     Show this screen.
  --version     Show version.
  --bot         Run regular bot in a match.
  --bot-test    Run test bot during dev in an empty match.
  --simulate    Run bot in a simulation of RL with visualization.
";

extern crate bincode;
extern crate brain;
extern crate csv;
extern crate docopt;
extern crate flate2;
extern crate kiss3d;
extern crate nalgebra as na;
extern crate passthrough;
extern crate rlbot;
extern crate spin_sleep;
extern crate state;

#[macro_use]
extern crate lazy_static;

use docopt::Docopt;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::f32;
use std::f32::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::panic;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use passthrough::{human_input, update_gamepad, Gamepad, Gilrs};
use state::*;

use brain::predict;
use kiss3d::light::Light;
use kiss3d::resource::MeshManager;
use kiss3d::window::Window;
use na::{Point3, Quaternion, Rotation3, Translation3, UnitQuaternion, Vector3};
use spin_sleep::LoopHelper;

#[derive(Default)]
struct BotIoConfig<'a> {
    manipulator: Option<
        fn(
            &mut u32,
            &rlbot::RLBot,
            &GameState,
            &mut BotState,
            &mut rlbot::ControllerState,
            &VecDeque<(GameState, BotState)>,
            &mut Gamepad,
        ) -> Result<(), Box<dyn Error>>,
    >,
    print_turn_errors: bool,
    render_debug_info: bool,
    save_debug_info: bool,
    record_history: bool,
    start_match: bool,
    match_settings: Option<rlbot::MatchSettings<'a>>,
}

lazy_static! {
    static ref GAME_STATE: RwLock<GameState> = { RwLock::new(GameState::default()) };
    static ref LINES: RwLock<Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>> = { RwLock::new(vec![]) };
    static ref POINTS: RwLock<Vec<(Point3<f32>, Point3<f32>)>> = { RwLock::new(vec![]) };
}

fn run_visualization() {
    let mut window = Window::new("Rocket League Visualization");

    // we're dividing everything by 1000 until we can set the camera up to be more zoomed out
    let mut sphere = window.add_sphere(BALL_COLLISION_RADIUS / 1000.0);
    let mut car = window.add_cube(
        CAR_DIMENSIONS.x / 1000.0,
        CAR_DIMENSIONS.y / 1000.0,
        CAR_DIMENSIONS.z / 1000.0,
    );

    let arena_mesh = MeshManager::load_obj(Path::new("./assets/arena.obj"), Path::new("./assets/"), "arena")
        .expect("Can't load arena obj file")
        .pop()
        .expect("Missing arena mesh")
        .1
        .clone();
    let mut arena = window.add_mesh(arena_mesh, Vector3::new(0.0, 0.0, 0.0));
    arena.set_surface_rendering_activation(false);
    arena.set_points_size(0.1);
    arena.set_lines_width(0.1);
    arena.set_local_scale(0.001, 0.001, 0.001);

    sphere.set_color(0.8, 0.8, 0.8);
    car.set_color(0.1, 0.4, 1.0);

    window.set_light(Light::StickToCamera);

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(60.0); // limit to 240 FPS

    while window.render() {
        loop_helper.loop_start();

        let game_state = &GAME_STATE.read().unwrap();
        let lines = &LINES.read().unwrap();
        let points = &POINTS.read().unwrap();

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        sphere.set_local_translation(Translation3::from(game_state.ball.position.map(|c| c / 1000.0)));

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        let hitbox_position = game_state.player.hitbox_center().map(|c| c / 1000.0);
        car.set_local_translation(Translation3::from(hitbox_position));
        car.set_local_rotation(game_state.player.rotation);

        // grid for debugging
        //for x in (-160..160) {
        //    window.draw_line(
        //        &Point3::new((x as f32) * 25.0 / 1000.0, -10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new((x as f32) * 25.0 / 1000.0,  10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.15, 0.15, 0.15)
        //    );
        //}
        //for x in (-40..40) {
        //    window.draw_line(
        //        &Point3::new((x as f32) * 100.0 / 1000.0, -10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new((x as f32) * 100.0 / 1000.0,  10_000.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.3, 0.3, 0.3)
        //    );
        //}
        //for y in (-160..160) {
        //    window.draw_line(
        //        &Point3::new(-10_000.0 / 1000.0, (y as f32) * 25.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new( 10_000.0 / 1000.0, (y as f32) * 25.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.15, 0.15, 0.15)
        //    );
        //}
        //for y in (-40..40) {
        //    window.draw_line(
        //        &Point3::new(-10_000.0 / 1000.0, (y as f32) * 100.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new( 10_000.0 / 1000.0, (y as f32) * 100.0 / 1000.0, 0.0 / 1000.0),
        //        &Point3::new(0.3, 0.3, 0.3)
        //    );
        //}

        for l in lines.iter() {
            window.draw_line(
                &Point3::new(l.0.x / 1000.0, l.0.y / 1000.0, l.0.z / 1000.0),
                &Point3::new(l.1.x / 1000.0, l.1.y / 1000.0, l.1.z / 1000.0),
                &l.2,
            );
        }

        for p in points.iter() {
            window.draw_point(&Point3::new(p.0.x / 1000.0, p.0.y / 1000.0, p.0.z / 1000.0), &p.1);
        }

        loop_helper.loop_sleep();
    }
}

/// main bot playing loop
/// this is the entry point for custom logic for this specific bot
fn run_bot() {
    let (state_sender, state_receiver): (Sender<(GameState, BotState)>, Receiver<(GameState, BotState)>) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        bot_io_loop(state_sender, plan_receiver, BotIoConfig::default());
    });
    bot_logic_loop(plan_sender, state_receiver);
}

fn run_bot_test() {
    let (state_sender, state_receiver): (Sender<(GameState, BotState)>, Receiver<(GameState, BotState)>) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        let _batmobile = rlbot::PlayerLoadout::new().car_id(803);
        let fennec = rlbot::PlayerLoadout::new().car_id(4284); // TODO get recordings of driving fennec for model

        let mut match_settings = rlbot::MatchSettings::new().player_configurations(vec![rlbot::PlayerConfiguration::new(
            rlbot::PlayerClass::RLBotPlayer,
            "Brick Test",
            0,
        )
        .loadout(fennec)]);

        match_settings.mutator_settings = rlbot::MutatorSettings::new().
            //game_speed_option(rlbot::GameSpeedOption::Slo_Mo). // NOTE this mutator doesn't work with bakkesmod
            respawn_time_option(rlbot::RespawnTimeOption::Disable_Goal_Reset).
            match_length(rlbot::MatchLength::Unlimited).
            boost_option(rlbot::BoostOption::Unlimited_Boost);

        let bot_io_config = BotIoConfig {
            manipulator: Some(bot_test_manipulator),
            record_history: true,
            print_turn_errors: true,
            render_debug_info: true,
            save_debug_info: true,
            start_match: true,
            match_settings: Some(match_settings),
        };

        bot_io_loop(state_sender, plan_receiver, bot_io_config);
    });
    bot_logic_loop_test(plan_sender, state_receiver);
}

#[allow(dead_code)]
fn get_desired_car_state(player: &PlayerState) -> rlbot::DesiredCarState {
    let pos = player.position;
    let vel = player.velocity;
    let avel = player.angular_velocity;

    let uq = player.rotation;
    let q = uq.quaternion();
    // converting from left handed to left right coordinate system (goes with the x axis flip)
    // see state::update_game_state
    // https://stackoverflow.com/a/34366144/127219
    let (roll, pitch, yaw) =
        UnitQuaternion::from_quaternion(Quaternion::new(q.scalar(), -q.vector()[0], q.vector()[1], -q.vector()[2]))
            .euler_angles();

    let position = rlbot::Vector3Partial::new().x(-pos.x).y(pos.y).z(pos.z);
    let velocity = rlbot::Vector3Partial::new().x(-vel.x).y(vel.y).z(vel.z);
    let angular_velocity = rlbot::Vector3Partial::new().x(-avel.x).y(avel.y).z(avel.z);
    let rotation = rlbot::RotatorPartial::new().pitch(pitch).yaw(yaw).roll(roll);
    let physics = rlbot::DesiredPhysics::new()
        .location(position)
        .rotation(rotation)
        .velocity(velocity)
        .angular_velocity(angular_velocity);

    rlbot::DesiredCarState::new().physics(physics)
}

#[allow(dead_code)]
fn get_desired_ball_state(ball: &BallState) -> rlbot::DesiredBallState {
    let pos = ball.position;
    let vel = ball.velocity;
    let avel = ball.angular_velocity;

    let position = rlbot::Vector3Partial::new().x(-pos.x).y(pos.y).z(pos.z);
    let velocity = rlbot::Vector3Partial::new().x(-vel.x).y(vel.y).z(vel.z);
    let angular_velocity = rlbot::Vector3Partial::new().x(-avel.x).y(avel.y).z(avel.z);
    let physics = rlbot::DesiredPhysics::new()
        .location(position)
        .velocity(velocity)
        .angular_velocity(angular_velocity);

    rlbot::DesiredBallState::new().physics(physics)
}

thread_local! {
    pub static SNAPSHOT_NUMBER: RefCell<u32> = RefCell::new(1);
}

#[allow(dead_code)]
fn record_snapshot(game: &GameState, bot: &BotState) -> Result<(), Box<dyn Error>> {
    let dir = "data/snapshots";
    create_dir_all(dir)?;

    let file_path = SNAPSHOT_NUMBER.with(|num| {
        let mut path;
        loop {
            path = Path::new(dir).join(format!("snapshot{}.bincode.gz", *num.borrow()));
            if !path.exists() {
                break;
            }
            (*num.borrow_mut()) += 1;
        }
        path
    });
    let f = BufWriter::new(File::create(file_path)?);
    let mut e = GzEncoder::new(f, Compression::default());
    Ok(bincode::serialize_into(&mut e, &(game, bot))?)
}

#[allow(dead_code)]
fn restore_snapshot(rlbot: &rlbot::RLBot, bot: &mut BotState, frame: &mut u32, name: &str) -> Result<(), Box<dyn Error>> {
    let dir = "data/snapshots";
    let path = Path::new(dir).join(name.to_owned() + ".bincode.gz");
    let f = BufReader::new(File::open(path)?);
    let mut decoder = GzDecoder::new(f);
    let (historical_game, historical_bot): (GameState, BotState) = bincode::deserialize_from(&mut decoder)?;

    // replace our bot data with the historical bot
    *bot = historical_bot;

    // update RL game state with historical game state
    *frame = historical_game.frame;
    let player = historical_game.player;
    let ball = historical_game.ball;
    let car_state = get_desired_car_state(&player);
    let ball_state = get_desired_ball_state(&ball);
    let desired_game_state = rlbot::DesiredGameState::new().car_state(0, car_state).ball_state(ball_state);
    Ok(rlbot.set_game_state(&desired_game_state)?)
}

#[allow(unused)]
fn bot_test_manipulator(
    frame: &mut u32,
    rlbot: &rlbot::RLBot,
    game: &GameState,
    bot: &mut BotState,
    input: &mut rlbot::ControllerState,
    history: &VecDeque<(GameState, BotState)>,
    gamepad: &mut Gamepad,
) -> Result<(), Box<dyn Error>> {
    if gamepad.select_toggled {
        if gamepad.south {
            // 0.2 seconds ago
            if let Some((game, bot)) = history.get(history.len() - 3) {
                record_snapshot(game, bot)?;
                gamepad.south = false;
            }
        }
        if gamepad.west {
            // 1 second ago
            if let Some((game, bot)) = history.get(history.len() - 11) {
                record_snapshot(game, bot)?;
                gamepad.west = false;
            }
        }
        if gamepad.north {
            // 2 seconds ago
            if let Some((game, bot)) = history.get(history.len() - 21) {
                record_snapshot(game, bot)?;
                gamepad.north = false;
            }
        }
        if gamepad.east {
            // 3 seconds ago
            if let Some((game, bot)) = history.get(history.len() - 31) {
                record_snapshot(game, bot)?;
                gamepad.east = false;
            }
        }
    }

    //// loop recorded snapshot
    // if (game.frame.saturating_sub(220)) % 400 == 0 {
    //     restore_snapshot(rlbot, bot, frame, "misses6")?;
    //     input.throttle = 0.0;
    //     input.steer = 0.0;
    //     bot.turn_errors.clear();
    //     std::thread::sleep(Duration::from_millis(10));
    // }

    // loop constructed scenario
    //if game.frame % 360 == 0 {
    //    let mut player = PlayerState::default();
    //    let mut ball = BallState::default();
    //    player.position.y = -3000.0;
    //    // up
    //    player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0);
    //    // down
    //    //player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
    //    ball.position.x = -400.0;
    //    ball.velocity.y = 100.0;
    //    ball.velocity.x = 600.0;
    //    player.velocity.y = 100.0;

    //    let car_state = get_desired_car_state(&player);
    //    let ball_state = get_desired_ball_state(&ball);
    //    let desired_game_state = rlbot::DesiredGameState::new().car_state(0, car_state).ball_state(ball_state);
    //    rlbot.set_game_state(&desired_game_state)?;

    //    bot.turn_errors.clear();
    //    input.throttle = 0.0;
    //    bot.plan = None;
    //    std::thread::sleep(Duration::from_millis(100))
    //}

    // // a basic snek
    // if game.player.position.y < 400.0 {
    //     input.steer = 0.0;
    // } else if game.player.position.y < 800.0 {
    //     input.steer = -1.0;
    // } else if game.player.position.y < 1200.0 {
    //     input.steer = 1.0;
    // } else if game.player.position.y < 1600.0 {
    //     input.steer = -1.0;
    // } else if game.player.position.y < 2000.0 {
    //     input.steer = 1.0;
    // } else if game.player.position.y < 3000.0 {
    //     input.steer = 0.0;
    // }

    Ok(())
}

fn bot_logic_loop(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    let mut model = brain::get_model();
    loop {
        let (mut game, mut bot) = receiver.recv().expect("Couldn't receive game state");

        // make sure we have the latest, drop earlier states
        while let Ok((g, b)) = receiver.try_recv() {
            game = g;
            bot = b;
        }

        let plan_result = brain::play::play(&mut model, &game, &mut bot);
        sender.send(plan_result).expect("Failed to send plan result");
    }
}

fn bot_test_plan<H: brain::HeuristicModel>(model: &mut H, game: &GameState, bot: &mut BotState) -> PlanResult {
    // canned plans
    //
    // // let player = &game.player.lag_compensated_player(&bot.controller_history, LAG_FRAMES);
    // let player = PlayerState::default();
    //
    // let mut plan_result = if let Ok(plan) = snek_plan(&player) {
    //     //for (i, (_next_player, controller, cost)) in plan.iter().enumerate() {
    //     //    println!("i: {}, steer: {:?}, steps: {}", i, controller.steer, (cost / TICK).round() as i32);
    //     //}
    //     PlanResult {
    //         plan: Some(plan),
    //         cost_diff: 0.0,
    //         visualization_lines: vec![],
    //         visualization_points: vec![],
    //     }
    // } else {
    //     PlanResult {
    //         plan: None,
    //         cost_diff: 0.0,
    //         visualization_lines: vec![],
    //         visualization_points: vec![],
    //     }
    // };
    // match brain::plan::explode_plan(&plan_result) {
    //     Ok(exploded) => {
    //         plan_result.plan = exploded;
    //         //println!("============= EXPLODED =============");
    //         //if let Some(x) = plan_result.plan.as_ref() {
    //         //    for (i, (_next_player, controller, cost)) in x.iter().enumerate() {
    //         //        println!("i: {}, steer: {:?}, steps: {}", i, controller.steer, (cost / TICK).round() as i32);
    //         //    }
    //         //}
    //         //println!("============= DONE =============");
    //     },
    //     Err(e) => {
    //         eprintln!("Exploding plan failed: {}", e);
    //         plan_result.plan = None;
    //     }
    // };
    // plan_result

    // just play
    brain::play::play(model, &game, bot)
}

fn bot_logic_loop_test(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();
    let mut model = brain::get_model();

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(1000.0); // limit to 1000 FPS
                                                                                //.build_with_target_rate(0.2); // limit to 0.2 FPS

    loop {
        loop_helper.loop_start();
        let (mut game, mut bot) = receiver.recv().expect("Couldn't receive game state");

        // make sure we have the latest, drop earlier states
        while let Ok((g, b)) = receiver.try_recv() {
            game = g;
            bot = b;
        }

        update_gamepad(&mut gilrs, &mut gamepad);
        if !gamepad.select_toggled {
            bot.turn_errors.clear();
            loop_helper.loop_sleep();
            continue;
        }

        sender
            .send(bot_test_plan(&mut model, &game, &mut bot))
            .expect("Failed to send plan result");

        loop_helper.loop_sleep();

        // below is for figuring out good PD control parameters, with a canned plan
        //
        // let mut square_errors = vec![];
        // loop {
        //     let (game, _bot) = receiver.recv().expect("Coudln't receive game state");

        //     let closest_index = brain::play::closest_plan_index(&game.player, &plan);
        //     plan = plan.split_off(closest_index);

        //     let square_error = (plan[0].0.position - &game.player.position)
        //         .norm()
        //         .powf(2.0);
        //     square_errors.push(square_error);

        //     if plan.len() <= 2 {
        //         break;
        //     }

        //     update_gamepad(&mut gilrs, &mut gamepad);
        //     if !gamepad.select_toggled {
        //         break;
        //     }

        //     let square_error = (plan[0].0.position - &game.player.position)
        //         .norm()
        //         .powf(2.0);
        //     square_errors.push(square_error);

        //     loop_helper.loop_sleep();
        // }
        // println!("========================================");
        // println!("Steps: {}", square_errors.len());
        // println!(
        //     "RMS Error: {}",
        //     (square_errors.iter().sum::<f32>() / (square_errors.len() as f32)).sqrt()
        // );
        // println!("========================================");
    }
}

pub fn try_next_flat(rlbot: &rlbot::RLBot, last_time: f32) -> Option<rlbot::GameTickPacket> {
    if let Some(packet) = rlbot.interface().update_live_data_packet_flatbuffer() {
        let game_time = packet.game_info.seconds_elapsed;
        if game_time != last_time {
            return Some(packet);
        }
    }
    None
}

#[allow(dead_code)]
fn move_ball_out_of_the_way(rlbot: &rlbot::RLBot) -> Result<(), Box<dyn Error>> {
    let position = rlbot::Vector3Partial::new().x(3800.0).y(4800.0).z(98.0);
    let physics = rlbot::DesiredPhysics::new().location(position);
    let ball_state = rlbot::DesiredBallState::new().physics(physics);
    let desired_game_state = rlbot::DesiredGameState::new().ball_state(ball_state);
    rlbot.set_game_state(&desired_game_state)?;
    Ok(())
}

fn bot_io_loop(sender: Sender<(GameState, BotState)>, receiver: Receiver<PlanResult>, bot_io_config: BotIoConfig) {
    let mut bot = BotState::default();
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();
    let rlbot = rlbot::init().expect("rlbot init failed");

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(1000.0); // bot io limited to 1000 FPS

    if bot_io_config.start_match {
        if let Some(match_settings) = bot_io_config.match_settings {
            rlbot.start_match(&match_settings).expect("Failed to start match");
            rlbot.wait_for_match_start().expect("Failed waiting for match start");
            //move_ball_out_of_the_way(&rlbot).expect("Failed to move ball out of the way");
            println!("Started Match!");
        } else {
            eprintln!("WARNING: Trying to start a match, but no match settings given. Ignoring start_match option.");
        }
    }

    // FIXME find out the new method for setting the correct player index since the tcp server
    // thing is gone afaik. just a command line argument now perhaps?
    let player_index = 0;

    //let mut start = std::time::Instant::now();
    let mut frame = 0u32;
    let mut logic_lag = 0u32; // measured in frames
    let mut history = VecDeque::new();

    let mut csv_writer = csv::Writer::from_path("debug.csv").expect("csv writer construction failed");

    let mut last_time = 0.0;
    loop {
        loop_helper.loop_start();
        if let Some(tick) = try_next_flat(&rlbot, last_time) {
            last_time = tick.game_info.seconds_elapsed;
            update_game_state(&mut GAME_STATE.write().unwrap(), &tick, player_index, frame);
            send_to_bot_logic(&sender, &bot, logic_lag);

            // make sure we have the latest results in case there are multiple, though note we may save
            // the plan from an earlier run if it happens to be the best one
            while let Ok(mut plan_result) = receiver.try_recv() {
                // track lag in our bot logic, which can vary considerably depending on the exact
                // game state. we use this to try to start our calculations from a future predicted
                // state, so that by the time the calculations are done they are not all completely
                // invalid due to the game state not proceeding according to that calculation's
                // plan. we simulate what will happen for logic_lag frames, then start our planning
                // from that point forward
                let latest_logic_lag = frame.saturating_sub(plan_result.source_frame);
                if latest_logic_lag > logic_lag {
                    // bump up immediately if higher, with an added margin
                    logic_lag = latest_logic_lag + 5;
                } else {
                    // lower slowly, moving average
                    logic_lag = (9 * logic_lag + latest_logic_lag) / 10;
                }

                // since the plan given is potentially starting in the future, if we
                // overcompensated, we need to stitch our current plan with the old plan
                // NOTE must be after tracking planning lag but before updating bot state, for
                // accurate debug logging of the planned player values
                stitch_with_current_plan(&GAME_STATE.read().unwrap(), &bot, &mut plan_result);

                update_bot_state(&GAME_STATE.read().unwrap(), &mut bot, &plan_result);
                match update_in_game_visualization(&rlbot, &bot, &plan_result) {
                    Ok(_) => {}
                    Err(e) => eprintln!("Failed rendering to rlbot: {}", e),
                };
            }

            // // remove part of plan that is no longer relevant since we've already passed it
            // if let Some(ref mut plan) = bot.plan {
            //     let closest_index = brain::play::closest_plan_index(&GAME_STATE.read().unwrap().player, &plan);
            //     //println!("closest index: {}, plan len: {}", closest_index, plan.len());
            //     *plan = plan.split_off(closest_index);
            // } else {
            //     //println!("no plan");
            // }

            update_gamepad(&mut gilrs, &mut gamepad);
            let mut input = if gamepad.select_toggled {
                brain::play::next_input(&GAME_STATE.read().unwrap().player, &mut bot)
            } else {
                bot.plan = None;
                bot.turn_errors.clear();
                human_input(&gamepad)
            };

            if bot_io_config.print_turn_errors {
                if bot.turn_errors.len() % 20 == 0 && bot.turn_errors.len() >= 20 {
                    // last 20
                    let errors = bot
                        .turn_errors
                        .iter()
                        .skip(bot.turn_errors.len() - 20)
                        .cloned()
                        .collect::<Vec<_>>();
                    //let sum = errors.iter().map(f32::abs).sum::<f32>();
                    //let avg = sum / 20.0;
                    let squared_sum = errors.iter().map(|x| x * x).sum::<f32>();
                    let rms = (squared_sum / 20.0).powf(0.5);
                    let max = errors.iter().map(|x| x.abs()).fold(-1.0f32 / 0.0 /* -inf */, f32::max);
                    let min = errors.iter().map(|x| x.abs()).fold(1.0f32 / 0.0 /* inf */, f32::min);
                    //println!("errors: {:?}", errors);
                    println!("rms: {}, min: {}, max: {}", rms, min, max);
                    //println!("first error: {}", bot.turn_errors[0]);
                }
            }

            // let's some kind of testing mode update the game state
            if let Some(manipulator) = bot_io_config.manipulator {
                manipulator(
                    &mut frame,
                    &rlbot,
                    &GAME_STATE.read().unwrap(),
                    &mut bot,
                    &mut input,
                    &history,
                    &mut gamepad,
                )
                .unwrap_or_else(|e| println!("manipulator error: {}", e));
            }

            bot.controller_history.push_back((&input).into());
            if bot.controller_history.len() > 1000 {
                // keep last 100
                bot.controller_history = bot.controller_history.split_off(900);
            }

            if bot_io_config.record_history {
                // 10 times per second at 120fps
                if frame % 12 == 0 {
                    history.push_back((GAME_STATE.read().unwrap().clone(), bot.clone()))
                }
                if history.len() > 100 {
                    // keep last 50
                    history = history.split_off(50);
                }
            }

            if bot_io_config.render_debug_info {
                let mut group = rlbot.begin_render_group(VISUALIZATION_GROUP_ID + 1000);
                let white = group.color_rgb(255, 255, 255);
                group.draw_string_2d((10.0, 5.0), (1, 1), format!("Steer: {}", input.steer), white);
                group.draw_string_2d((10.0, 20.0), (1, 1), format!("Boost: {}", input.boost), white);
                group.draw_string_2d((10.0, 35.0), (1, 1), format!("Throttle: {}", input.throttle), white);
                if bot.turn_errors.len() > 0 {
                    group.draw_string_2d(
                        (10.0, 50.0),
                        (1, 1),
                        format!("Error: {:?}", bot.turn_errors.get(bot.turn_errors.len() - 1).unwrap()),
                        white,
                    );
                } else {
                    group.draw_string_2d((10.0, 50.0), (1, 1), "Error: -", white);
                }
                let pos = GAME_STATE.read().unwrap().player.position;
                group.draw_string_2d((10.0, 65.0), (1, 1), format!("Position: ({}, {})", pos.x, pos.y), white);
                group.render().unwrap_or_else(|e| println!("render error: {}", e));
            }

            if bot_io_config.save_debug_info {
                if let Some(plan) = bot.plan.as_ref() {
                    if let Some((planned_player, planned_controller, _)) =
                        plan.get((frame.wrapping_sub(bot.plan_source_frame)) as usize)
                    {
                        if let Some(planned_ball) = bot.planned_ball.as_ref() {
                            let game = GAME_STATE.read().unwrap();
                            let player = &game.player;
                            let ball = game.ball;
                            //bot.clone())
                            let (roll, pitch, yaw) = player.rotation.euler_angles();
                            let (planned_roll, planned_pitch, planned_yaw) = planned_player.rotation.euler_angles();
                            let planned_input: rlbot::ControllerState = (*planned_controller).into();
                            let row: Vec<String> = [
                                frame as f32,
                                bot.plan_source_frame as f32,
                                logic_lag as f32,
                                player.position.x,
                                player.position.y,
                                player.position.z,
                                planned_player.position.x,
                                planned_player.position.y,
                                planned_player.position.z,
                                player.velocity.x,
                                player.velocity.y,
                                player.velocity.z,
                                planned_player.velocity.x,
                                planned_player.velocity.y,
                                planned_player.velocity.z,
                                player.angular_velocity.x,
                                player.angular_velocity.y,
                                player.angular_velocity.z,
                                planned_player.angular_velocity.x,
                                planned_player.angular_velocity.y,
                                planned_player.angular_velocity.z,
                                roll,
                                pitch,
                                yaw,
                                planned_roll,
                                planned_pitch,
                                planned_yaw,
                                input.throttle,
                                input.steer,
                                (if input.boost { 1.0 } else { 0.0 }),
                                planned_input.throttle,
                                planned_input.steer,
                                (if planned_input.boost { 1.0 } else { 0.0 }),
                                ball.position.x,
                                ball.position.y,
                                ball.position.z,
                                planned_ball.position.x,
                                planned_ball.position.y,
                                planned_ball.position.z,
                                ball.velocity.x,
                                ball.velocity.y,
                                ball.velocity.z,
                                planned_ball.velocity.x,
                                planned_ball.velocity.y,
                                planned_ball.velocity.z,
                                ball.angular_velocity.x,
                                ball.angular_velocity.y,
                                ball.angular_velocity.z,
                                planned_ball.angular_velocity.x,
                                planned_ball.angular_velocity.y,
                                planned_ball.angular_velocity.z,
                            ]
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>();
                            csv_writer.write_record(&row).expect("Writing debug csv failed");
                        }
                    }
                }
            }

            rlbot
                .update_player_input(player_index as i32, &input)
                .expect("update_player_input failed");
            frame = frame.wrapping_add(1);
            //if frame % 120 == 0 {
            //    println!("120 frames took: {:?}", start.elapsed());
            //    start = std::time::Instant::now();
            //}
        }
        loop_helper.loop_sleep();
    }
}

fn stitch_with_current_plan(game: &GameState, bot: &BotState, plan_result: &mut PlanResult) {
    if let Some(last_plan) = adjusted_plan_or_fallback(&game, bot) {
        if let Some(plan) = &plan_result.plan {
            let closest_index_now = brain::play::closest_plan_index(&game.player, &last_plan);
            let closest_index_plan = brain::play::closest_plan_index(&plan[0].0, &last_plan);

            if closest_index_plan < closest_index_now {
                // we had NOT compensated enough for the logic lag
                eprintln!("Aborting plan stitching due to inadequate logic lag compensation");
                return;
            }

            // we had compensated enough for the logic lag, now adjust the plan
            let mut stitched_plan = last_plan
                .iter()
                .cloned()
                .skip(closest_index_now)
                .take(closest_index_plan - closest_index_now)
                .collect::<Vec<_>>();
            stitched_plan.extend(plan.clone());

            plan_result.plan = Some(stitched_plan);

            // adjust the source frame to be the one now, given our plan has been adjusted to
            // start *right now*. this is required for debug logging to get the right intended
            // index in the new plan. XXX must be done after we calculate logic lag, but before
            // updating bot state
            plan_result.source_frame = game.frame;
            println!("Plan stitched successfully");
        }
    }
}

fn plan_is_valid(game: &GameState, plan: &Plan) -> bool {
    let closest_index = brain::play::closest_plan_index(&game.player, &plan);
    if let Some((player, _, _)) = plan.get(closest_index) {
        let ball_trajectory = predict::ball::ball_trajectory(&game.ball, (plan.len() - 1 - closest_index) as f32 * TICK);
        let is_player_accurate = (player.position - game.player.position).norm() < 30.0
            && (player.velocity - game.player.velocity).norm() < 200.0;

        let last_player = &plan.last().unwrap().0;
        let last_ball = ball_trajectory.last().unwrap();
        let is_ball_colliding = (predict::player::closest_point_for_collision(last_ball, last_player) - last_ball.position)
            .norm()
            < (BALL_COLLISION_RADIUS + 20.0); // 20.0 fudge factor added
        is_player_accurate && is_ball_colliding
    } else {
        false
    }
}

fn update_bot_state(game: &GameState, bot: &mut BotState, plan_result: &PlanResult) {
    if let Some(ref new_plan) = plan_result.plan {
        if let Some(ref existing_plan) = bot.plan {
            let new_plan_cost = new_plan.iter().map(|(_, _, cost)| cost).sum::<f32>();

            let closest_index = brain::play::closest_plan_index(&game.player, &existing_plan);
            let existing_plan_cost = existing_plan
                .iter()
                .enumerate()
                .filter(|(index, _val)| *index > closest_index)
                .map(|(_index, (_, _, cost))| cost)
                .sum::<f32>();

            // bail, we got a worse plan!
            if new_plan_cost >= existing_plan_cost && plan_is_valid(&game, &existing_plan) {
                //println!("bailing longer plan! existing_plan_cost: {}, new_plan_cost: {}", existing_plan_cost, new_plan_cost);
                return;
            }

            //let existing_diff = bot.cost_diff.abs();
            //let new_diff = plan_result.cost_diff.abs();
            //if new_diff > existing_diff && plan_is_valid(&game, &existing_plan) {
            //    //println!("bailing less accurate plan! existing_diff: {}, new_diff: {}", existing_diff, new_diff);
            //    return;
            //} else if new_diff == existing_diff && new_plan_cost >= existing_plan_cost && plan_is_valid(&game, &existing_plan) {
            //    //println!("bailing longer plan! existing_plan_cost: {}, new_plan_cost: {}", existing_plan_cost, new_plan_cost);
            //    return;
            //}
        }

        //let cost = new_plan.iter().map(|(_, _, cost)| cost).sum::<f32>();
        //println!("new best plan! cost: {}", cost);
        bot.plan = Some(new_plan.clone());
        bot.planned_ball = plan_result.planned_ball.clone();
        bot.cost_diff = plan_result.cost_diff;
        bot.turn_errors.clear();
        bot.plan_source_frame = plan_result.source_frame;
    }
}

fn plan_lines(plan: &Plan, color: Point3<f32>) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
    let mut lines = Vec::with_capacity(plan.len());
    let pos = plan
        .get(0)
        .map(|(p, _, _)| p.position)
        .unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));
    let mut last_point = Point3::new(pos.x, pos.y, pos.z);
    for (ps, _, _) in plan {
        let point = Point3::new(ps.position.x, ps.position.y, ps.position.z + 0.1);
        lines.push((last_point.clone(), point.clone(), color));
        last_point = point;
    }
    lines
}

fn update_simulation_visualization(bot: &BotState, plan_result: &PlanResult) {
    let PlanResult {
        plan,
        visualization_lines: lines,
        visualization_points: points,
        ..
    } = plan_result;

    let mut visualization_lines = LINES.write().unwrap();
    visualization_lines.clear();

    // lines directly from plan result
    visualization_lines.append(&mut lines.clone());

    // white line showing best planned path
    if let Some(ref plan) = bot.plan {
        visualization_lines.append(&mut plan_lines(&plan, Point3::new(1.0, 1.0, 1.0)));
    }

    // blue line showing most recently calculated path
    if let Some(plan) = plan {
        visualization_lines.append(&mut plan_lines(&plan, Point3::new(0.0, 1.0, 1.0)));
    }

    let mut visualization_points = POINTS.write().unwrap();
    visualization_points.clear();
    visualization_points.append(&mut points.clone());
}

const VISUALIZATION_GROUP_ID: i32 = 7323; // trying to not overlap with other bots, though idk if they CAN overlap

fn draw_lines(
    rlbot: &rlbot::RLBot,
    lines: &Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>,
    chunk_num: &mut i32,
) -> Result<(), Box<dyn Error>> {
    for chunk in lines.chunks(200) {
        // TODO in case of multiple bricks, add player index * 1000 to the group id
        let mut group = rlbot.begin_render_group(VISUALIZATION_GROUP_ID + *chunk_num);

        for &l in chunk.iter() {
            let p1 = l.0;
            let p2 = l.1;
            let p3 = l.2;
            let color = group.color_rgb((255.0 * p3.x) as u8, (255.0 * p3.y) as u8, (255.0 * p3.z) as u8);
            group.draw_line_3d((-p1.x, p1.y, p1.z), (-p2.x, p2.y, p2.z), color);
        }

        group.render()?;

        *chunk_num += 1;
    }

    Ok(())
}

fn icosahedron_lines(ball: &BallState) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
    let mut vertices = [
        Vector3::new(-0.262865, 0.00000, 0.42532500),
        Vector3::new(0.262865, 0.00000, 0.42532500),
        Vector3::new(-0.262865, 0.00000, -0.42532500),
        Vector3::new(0.262865, 0.00000, -0.42532500),
        Vector3::new(0.00000, 0.425325, 0.26286500),
        Vector3::new(0.00000, 0.425325, -0.26286500),
        Vector3::new(0.00000, -0.425325, 0.26286500),
        Vector3::new(0.00000, -0.425325, -0.26286500),
        Vector3::new(0.425325, 0.262865, 0.0000000),
        Vector3::new(-0.425325, 0.262865, 0.0000000),
        Vector3::new(0.425325, -0.262865, 0.0000000),
        Vector3::new(-0.425325, -0.262865, 0.0000000),
    ]
    .iter()
    .map(|v| ball.position + (BALL_COLLISION_RADIUS / 0.45) * v)
    .collect::<Vec<_>>();
    let mut lines = vec![];
    for vertex in vertices.clone().iter() {
        vertices.sort_by(|v1, v2| (vertex - v1).norm().partial_cmp(&(vertex - v2).norm()).unwrap());
        let closest = vertices.iter().skip(1).take(5);
        for v in closest {
            lines.push((
                Point3::new(vertex.x, vertex.y, vertex.z),
                Point3::new(v.x, v.y, v.z),
                Point3::new(1.0, 1.0, 1.0),
            ));
        }
    }
    lines
}

fn hitbox_lines(player: &PlayerState) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
    let mut vertices = [
        Vector3::new(CAR_DIMENSIONS.x, CAR_DIMENSIONS.y, CAR_DIMENSIONS.z),
        Vector3::new(-CAR_DIMENSIONS.x, CAR_DIMENSIONS.y, CAR_DIMENSIONS.z),
        Vector3::new(CAR_DIMENSIONS.x, -CAR_DIMENSIONS.y, CAR_DIMENSIONS.z),
        Vector3::new(-CAR_DIMENSIONS.x, -CAR_DIMENSIONS.y, CAR_DIMENSIONS.z),
        Vector3::new(CAR_DIMENSIONS.x, CAR_DIMENSIONS.y, -CAR_DIMENSIONS.z),
        Vector3::new(-CAR_DIMENSIONS.x, CAR_DIMENSIONS.y, -CAR_DIMENSIONS.z),
        Vector3::new(CAR_DIMENSIONS.x, -CAR_DIMENSIONS.y, -CAR_DIMENSIONS.z),
        Vector3::new(-CAR_DIMENSIONS.x, -CAR_DIMENSIONS.y, -CAR_DIMENSIONS.z),
    ]
    .iter()
    .map(|v| player.hitbox_center() + player.rotation.to_rotation_matrix() * (0.5 * v))
    .collect::<Vec<_>>();
    let mut lines = vec![];
    for vertex in vertices.clone().iter() {
        vertices.sort_by(|v1, v2| (vertex - v1).norm().partial_cmp(&(vertex - v2).norm()).unwrap());
        let closest = vertices.iter().skip(1).take(4);
        for v in closest {
            lines.push((
                Point3::new(vertex.x, vertex.y, vertex.z),
                Point3::new(v.x, v.y, v.z),
                Point3::new(1.0, 1.0, 1.0),
            ));
        }
    }
    lines
}

fn update_in_game_visualization(
    rlbot: &rlbot::RLBot,
    bot: &BotState,
    plan_result: &PlanResult,
) -> Result<(), Box<dyn Error>> {
    let plan = &plan_result.plan;
    let mut chunk_num = 0;

    if let Some(ref plan) = bot.plan {
        // white line showing best planned path
        draw_lines(&rlbot, &plan_lines(&plan, Point3::new(1.0, 1.0, 1.0)), &mut chunk_num)?;

        // visualization of ball at end of planned path
        if let Some(ref planned_ball) = bot.planned_ball {
            draw_lines(&rlbot, &icosahedron_lines(planned_ball), &mut chunk_num)?;
        }

        // visualization of car at end of planned path
        if let Some((player, _, _)) = plan.last() {
            draw_lines(&rlbot, &hitbox_lines(player), &mut chunk_num)?;
        }
    }

    if let Some(plan) = plan {
        // turquoise line showing most recently calculated path
        draw_lines(&rlbot, &plan_lines(&plan, Point3::new(0.0, 1.0, 1.0)), &mut chunk_num)?;
    }

    Ok(())
}

// using the current player state as a starting point, apply the plan
// FIXME unexplode plan first and calculate using those longer durations for more accurate
// values, then explode again
// TODO for even more accuracy:
// - simulate following the plan while not exactly on it
// - simulate turn error correction
fn adjusted_plan(player: &PlayerState, plan: Plan) -> Result<Plan, Box<dyn Error>> {
    let mut adjusted = Vec::with_capacity(plan.len());
    adjusted.push((player.clone(), BrickControllerState::new(), 0.0));
    let mut last_player: PlayerState = player.clone();
    plan.iter().filter(|(_, _, cost)| *cost > 0.0).map(|(_, controller, cost): &(PlayerState, BrickControllerState, f32)| -> Result<(PlayerState, BrickControllerState, f32), Box<dyn Error>> {
        let next_player = predict::player::next_player_state(&last_player, &controller, *cost);
        if next_player.is_err() {
            // print to stderr now since we're swallowing these errors right after this
            eprintln!("Warning: failed to adjust plan: {}", next_player.as_ref().unwrap_err());
        }
        last_player = next_player?;
        Ok((last_player.clone(), controller.clone(), *cost))
    }).filter_map(Result::ok).for_each(|plan_val| {
        adjusted.push(plan_val);
    });

    Ok(adjusted)
}

fn adjusted_plan_or_fallback(game: &GameState, bot: &BotState) -> Option<Plan> {
    if let Some(bot_plan) = &bot.plan {
        if let Ok(adjusted) = adjusted_plan(&game.player, bot_plan.clone()) {
            Some(adjusted)
        } else {
            eprintln!("Failed to adjust plan for logic lag compensation");
            None
        }
    } else {
        // NOTE hard-coded 1 second, if we take longer in bot logic, we have bigger problems
        match forward_plan(&game.player, 1000.0) {
            Ok(fallback_plan) => {
                // handles fallback case where there isn't a plan and we just throttle forward
                if let Ok(exploded) = brain::plan::explode_plan(&Some(fallback_plan)) {
                    exploded.clone()
                } else {
                    eprintln!("Failed to explode fallback plan for logic lag compensation");
                    None
                }
            }
            Err(e) => {
                eprintln!("Failed to calculate fallback plan for logic lag compensation: {}", e);
                None
            }
        }
    }
}

fn send_to_bot_logic(sender: &Sender<(GameState, BotState)>, bot: &BotState, logic_lag: u32) {
    let mut game = (*GAME_STATE.read().unwrap()).clone();

    // logic lag compensation: send player and ball in the future, since our calculation might take
    // a while and we don't want a slow calculation's result to be invalid immediately
    if let Some(plan) = adjusted_plan_or_fallback(&game, bot) {
        let index = brain::play::closest_plan_index(&game.player, &plan) + logic_lag as usize;
        if let Some((player, _, _)) = plan.get(index) {
            game.player = player.clone();
        }
        game.ball = predict::ball::ball_trajectory(&game.ball, logic_lag as f32 * TICK)
            .pop()
            .expect("Missing ball trajectory");
    }

    sender.send((game, bot.clone())).expect("Sending to bot logic failed");
}

#[allow(dead_code)]
fn turn_plan(current: &PlayerState, angle: f32) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let desired_heading = Rotation3::from_euler_angles(0.0, 0.0, angle) * current_heading;
    let mut turn_controller = BrickControllerState::new();
    turn_controller.throttle = Throttle::Forward;
    turn_controller.steer = if angle < 0.0 { Steer::Right } else { Steer::Left };

    let mut straight_controller = BrickControllerState::new();
    straight_controller.throttle = Throttle::Forward;

    const TURN_DURATION: f32 = 16.0 * TICK;
    // straighten out for zero angular velocity at end, hopefully 16 ticks is enough?
    const STRAIGHT_DURATION: f32 = 16.0 * TICK;

    // iterate till dot product is maximized (ie we match the desired heading)
    let mut last_dot = std::f32::MIN;
    let mut player = current.clone();
    loop {
        let turn_player = predict::player::next_player_state(&player, &turn_controller, TURN_DURATION)?;
        let turn_heading = turn_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let turn_dot = na::Matrix::dot(&turn_heading, &desired_heading);

        // straight duration is much longer than turn duration
        let long_turn_player = predict::player::next_player_state(&turn_player, &turn_controller, STRAIGHT_DURATION)?;
        let long_turn_heading = long_turn_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let long_turn_dot = na::Matrix::dot(&long_turn_heading, &desired_heading);

        let turn_then_straight_player =
            predict::player::next_player_state(&turn_player, &straight_controller, STRAIGHT_DURATION)?;
        let turn_then_straight_heading =
            turn_then_straight_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let turn_then_straight_dot = na::Matrix::dot(&turn_then_straight_heading, &desired_heading);

        let straight_player = predict::player::next_player_state(&player, &straight_controller, STRAIGHT_DURATION)?;
        let straight_heading = straight_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let straight_dot = na::Matrix::dot(&straight_heading, &desired_heading);

        // if we aren't overshooting, add a turn
        if turn_dot > last_dot && long_turn_dot > turn_then_straight_dot {
            plan.push((turn_player.clone(), turn_controller, TURN_DURATION));
            player = turn_player;
            last_dot = turn_dot;
        } else if turn_then_straight_dot > last_dot && turn_then_straight_dot > straight_dot {
            plan.push((turn_player, turn_controller, TURN_DURATION));
            plan.push((turn_then_straight_player.clone(), straight_controller, STRAIGHT_DURATION));
            player = turn_then_straight_player;
            last_dot = turn_then_straight_dot;
        } else if straight_dot > last_dot + 0.001 {
            plan.push((straight_player.clone(), straight_controller, STRAIGHT_DURATION));
            player = straight_player;
            last_dot = straight_dot;
        } else {
            break;
        }
    }

    Ok(plan)
}

#[allow(dead_code)]
fn forward_plan(current: &PlayerState, time: f32) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];

    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    let mut player = current.clone();
    let mut time_so_far = 0.0;
    while time_so_far < time {
        let step_duration = 16.0 * TICK;
        player = predict::player::next_player_state(&player, &controller, step_duration)?;
        plan.push((player.clone(), controller, step_duration));
        time_so_far += step_duration;
    }

    Ok(plan)
}

#[allow(dead_code)]
fn square_plan(player: &PlayerState) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];
    plan.push((player.clone(), BrickControllerState::new(), 0.0));
    for _ in 0..4 {
        let mut plan_part = forward_plan(&plan[plan.len() - 1].0, 1000.0)?;
        plan.append(&mut plan_part);
        let mut plan_part = turn_plan(&plan[plan.len() - 1].0, -PI / 2.0)?;
        plan.append(&mut plan_part);
    }

    Ok(plan)
}

#[allow(dead_code)]
fn snek_plan(player: &PlayerState) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];
    plan.push((player.clone(), BrickControllerState::new(), 0.0));

    let mut plan_part = forward_plan(&plan[plan.len() - 1].0, 500.0)?;
    plan.append(&mut plan_part);
    for _ in 0..2 {
        let mut plan_part = turn_plan(&plan[plan.len() - 1].0, PI / 6.0)?;
        plan.append(&mut plan_part);
        let mut plan_part = turn_plan(&plan[plan.len() - 1].0, -PI / 6.0)?;
        plan.append(&mut plan_part);
    }

    Ok(plan)
}

#[allow(dead_code)]
fn snek_plan2(current: &PlayerState) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];
    let mut player = current.clone();
    plan.push((player.clone(), BrickControllerState::new(), 0.0));

    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;
    for _ in 0..4 {
        player = predict::player::next_player_state(&player, &controller, 16.0 * TICK)?;
        plan.push((player.clone(), controller, 16.0 * TICK));
    }

    for _ in 0..2 {
        controller.steer = Steer::Left;
        for _ in 0..4 {
            player = predict::player::next_player_state(&player, &controller, 16.0 * TICK)?;
            plan.push((player.clone(), controller, 16.0 * TICK));
        }

        controller.steer = Steer::Right;
        for _ in 0..4 {
            player = predict::player::next_player_state(&player, &controller, 16.0 * TICK)?;
            plan.push((player.clone(), controller, 16.0 * TICK));
        }
    }

    Ok(plan)
}

#[allow(dead_code)]
fn snek_plan3(current: &PlayerState) -> Result<Plan, Box<dyn Error>> {
    let mut plan = vec![];
    let mut player = current.clone();
    plan.push((player.clone(), BrickControllerState::new(), 0.0));

    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    while player.position.y < 3000.0 {
        let get_steer = |y| -> Steer {
            if y < 400.0 {
                Steer::Straight
            } else if y < 800.0 {
                Steer::Left
            } else if y < 1200.0 {
                Steer::Right
            } else if y < 1600.0 {
                Steer::Left
            } else if y < 2000.0 {
                Steer::Right
            } else if y < 3000.0 {
                Steer::Straight
            } else {
                Steer::Straight
            }
        };

        controller.steer = get_steer(player.position.y);

        let mut next_player = predict::player::next_player_state(&player, &controller, 16.0 * TICK)?;

        if get_steer(next_player.position.y) == controller.steer {
            plan.push((next_player.clone(), controller, 16.0 * TICK));
        } else {
            // simulate till the point steer is supposed to change
            next_player = player.clone();
            loop {
                // TODO 1-tick if we can?
                next_player = predict::player::next_player_state(&next_player, &controller, 2.0 * TICK)?;
                plan.push((next_player.clone(), controller, 2.0 * TICK));
                if get_steer(next_player.position.y) != controller.steer {
                    break;
                }
            }
        }

        player = next_player
    }

    Ok(plan)
}

#[allow(dead_code)]
fn offset_forward_plan(current: &PlayerState) -> Result<Plan, Box<dyn Error>> {
    let mut offset_player = current.clone();
    let heading = offset_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, -PI / 2.0);
    let right = clockwise_90_rotation * heading;
    offset_player.position += 200.0 * right;

    forward_plan(&offset_player, 4000.0)
}

fn simulate_over_time() {
    thread::sleep(Duration::from_millis(5000));
    let initial_game_state: GameState;
    let mut bot = BotState::default();
    let mut model = brain::get_model();

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(120.0); // simulation limited to 120 FPS

    {
        let mut game_state = GAME_STATE.write().unwrap();
        game_state.ball.position = Vector3::new(0.0, 0.0, BALL_COLLISION_RADIUS);
        game_state.ball.velocity = Vector3::new(400.0, 400.0, 0.0);
        game_state.player.position = Vector3::new(0.0, -1000.0, 0.0);
        game_state.player.velocity = Vector3::new(0.0, 0.0, 0.0);

        // left
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        // up
        game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0);
        // down
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        // right
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI);

        initial_game_state = game_state.clone();
    }

    loop {
        loop_helper.loop_start();
        {
            let game_state = GAME_STATE.read().unwrap();
            let plan_result = brain::play::play(&mut model, &game_state, &mut bot);
            update_bot_state(&game_state, &mut bot, &plan_result);
            update_simulation_visualization(&bot, &plan_result);
            // this pauses the simulation forever when no plan is found
            // if plan_result.plan.is_none() {
            //     thread::sleep(Duration::from_millis(5000)));
            //     continue;
            // }
        }

        if let Some(plan) = bot.plan.clone() {
            let mut game_state = GAME_STATE.write().unwrap();
            game_state.ball = predict::ball::next_ball_state(&game_state.ball, TICK);
            let i = brain::play::closest_plan_index(&game_state.player, &plan);
            if plan.len() >= i + 2 {
                game_state.player = plan[i + 1].0.clone();
            } else {
                // we're at the goal, so start over
                *game_state = initial_game_state.clone();
                bot.plan = None;
            }
        } else {
            // no plan, just try again with a slightly different position
            let mut game_state = GAME_STATE.write().unwrap();
            game_state.player.position += Vector3::new(20.0, 20.0, 0.0);
        }

        loop_helper.loop_sleep();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Docopt::new(USAGE).and_then(|dopt| dopt.parse()).unwrap_or_else(|e| e.exit());

    let test_bot = args.get_bool("--bot-test");
    if args.get_bool("--bot") || test_bot {
        thread::spawn(move || loop {
            let t = thread::spawn(move || {
                if test_bot {
                    panic::catch_unwind(run_bot_test).expect("Panic catch unwind failed");
                } else {
                    panic::catch_unwind(run_bot).expect("Panic catch unwind failed");
                }
            });
            t.join()
                .expect_err("The bot thread should only end if panic, but it didn't panic.");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
            println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

            thread::sleep(Duration::from_millis(1000));
        });
    } else if args.get_bool("--simulate") {
        thread::spawn(simulate_over_time);
    } else {
        panic!("Must provide --bot, --bot-test or --simulate");
    }

    if args.get_bool("--simulate") {
        run_visualization();
    } else {
        thread::park();
    }

    Ok(())
}
