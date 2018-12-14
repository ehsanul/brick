// NOTE using system allocator so that we can interface with a dylib and share heap-allocated
// structures *for live reloading purposes ONLY*
// FIXME remove for final build.
#![cfg_attr(rustc_nightly, feature(test))]
#![feature(alloc_system)]
#![feature(allocator_api)]
extern crate alloc_system;
use alloc_system::System;
#[global_allocator]
static A: System = System;
// FIXME remove above for final build.


const USAGE: &'static str = "
Brick

Usage:
  brick --bot
  brick --bot-test
  brick --simulate
  brick --plan-test

Options:
  -h --help     Show this screen.
  --version     Show version.
  --bot         Run regular bot in a match.
  --bot-test    Run regular bot using test plan.
  --simulate    Simulate game over time.
  --plan-test   Test & visualize a single plan.
";

extern crate kiss3d;
extern crate nalgebra as na;
extern crate dynamic_reload;
extern crate state;
extern crate rlbot;
extern crate passthrough;
extern crate docopt;

#[macro_use]
extern crate lazy_static;


use docopt::Docopt;
use std::panic;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, RwLock, Mutex};
use std::f32;
use std::f32::consts::PI;
use std::path::Path;
use std::time::{Duration, Instant};
use std::error::Error;

use state::*;
use passthrough::{Gilrs, Gamepad, human_input, update_gamepad};

use na::{ Unit, Vector3, Point3, Translation3, UnitQuaternion, Rotation3};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::MeshManager;

use dynamic_reload::{DynamicReload, Lib, Symbol, Search, PlatformName, UpdateState};
pub const TICK: f32 = 1.0 / 120.0; // FIXME import from predict

lazy_static! {
    static ref PLAYER_INDEX: Mutex<Option<usize>> = Mutex::new(None);

    static ref GAME_STATE: RwLock<GameState> = {
        RwLock::new(GameState::default())
    };

    static ref LINES: RwLock<Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>> = {
        RwLock::new(vec![])
    };

    static ref POINTS: RwLock<Vec<(Point3<f32>, Point3<f32>)>> = {
        RwLock::new(vec![])
    };

    static ref RELOAD_HANDLER: Mutex<DynamicReload<'static>> = {
        Mutex::new(
            DynamicReload::new(Some(vec!["target/release"]),
                               Some("target/release"),
                               Search::Default)
        )
    };

    static ref RELOAD_HANDLER2: Mutex<DynamicReload<'static>> = {
        Mutex::new(
            DynamicReload::new(Some(vec!["target/release"]),
                               Some("target/release"),
                               Search::Default)
        )
    };

    static ref BRAIN: Mutex<BrainPlugin> = {
        let lib = match RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER").add_library("brain", PlatformName::Yes) {
            Ok(lib) => lib,
            Err(e) => {
                panic!("Unable to load dynamic lib, err {:?}", e);
            }
        };
        Mutex::new(BrainPlugin { lib: Some(lib) })
    };

    static ref BRAIN2: Mutex<BrainPlugin> = {
        let lib = match RELOAD_HANDLER2.lock().expect("Failed to get lock on RELOAD_HANDLER2").add_library("brain", PlatformName::Yes) {
            Ok(lib) => lib,
            Err(e) => {
                panic!("2Unable to load dynamic lib, err {:?}", e);
            }
        };
        Mutex::new(BrainPlugin { lib: Some(lib) })
    };
}



struct BrainPlugin {
    lib: Option<Arc<Lib>>
}

impl BrainPlugin {
    fn unload_plugin(&mut self, lib: &Arc<Lib>) {
        self.lib = None;
    }

    fn reload_plugin(&mut self, lib: &Arc<Lib>) {
        self.lib = Some(lib.clone());
    }

    fn reload_callback(&mut self, state: UpdateState, lib: Option<&Arc<Lib>>) {
        match state {
            UpdateState::Before => Self::unload_plugin(self, lib.unwrap()),
            UpdateState::After => Self::reload_plugin(self, lib.unwrap()),
            UpdateState::ReloadFailed(_) => println!("Failed to reload"),
        }
    }
}


fn run_visualization(){
    let mut window = Window::new("Rocket League Visualization");

    // we're dividing everything by 1000 until we can set the camera up to be more zoomed out
    let mut sphere = window.add_sphere(BALL_RADIUS / 1000.0);
    let mut car = window.add_cube(CAR_DIMENSIONS.x/1000.0, CAR_DIMENSIONS.y/1000.0, CAR_DIMENSIONS.z/1000.0);

    let arena_mesh = MeshManager::load_obj(
                        Path::new("./assets/arena.obj"),
                        Path::new("./assets/"),
                        "arena"
                    ).expect("Can't load arena obj file")
                    .pop().expect("Missing arena mesh")
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

    while window.render() {
        thread::sleep_ms(100);
        let game_state = &GAME_STATE.read().unwrap();
        let lines = &LINES.read().unwrap();
        let points = &POINTS.read().unwrap();

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        sphere.set_local_translation(Translation3::from_vector(game_state.ball.position.map(|c| c / 1000.0)));

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        let hitbox_position = game_state.player.position.map(|c| c / 1000.0) + PIVOT_OFFSET.map(|c| c / 1000.0);
        car.set_local_translation(Translation3::from_vector(hitbox_position));
        car.set_local_rotation(game_state.player.rotation); // FIXME need to rotate about the pivot, not center

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
            window.draw_line(&Point3::new(l.0.x / 1000.0, l.0.y / 1000.0, l.0.z / 1000.0), &Point3::new(l.1.x / 1000.0, l.1.y / 1000.0, l.1.z / 1000.0), &l.2);
        }

        for p in points.iter() {
            window.draw_point(&Point3::new(p.0.x / 1000.0, p.0.y / 1000.0, p.0.z / 1000.0), &p.1);
        }
    }
}

/// main bot playing loop
/// this is the entry point for custom logic for this specific bot
fn run_bot() {
    let (state_sender, state_receiver): (Sender<(GameState, BotState)>, Receiver<(GameState, BotState)>) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        bot_io_loop(state_sender, plan_receiver);
    });
    bot_logic_loop(plan_sender, state_receiver);
}

fn run_bot_live_test() {
    let (state_sender, state_receiver): (Sender<(GameState, BotState)>, Receiver<(GameState, BotState)>) = mpsc::channel();
    let (plan_sender, plan_receiver): (Sender<PlanResult>, Receiver<PlanResult>) = mpsc::channel();
    thread::spawn(move || {
        bot_io_loop(state_sender, plan_receiver);
    });
    bot_logic_loop_live_test(plan_sender, state_receiver);
}

fn bot_logic_loop(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    loop {
        let (mut game, mut bot) = receiver.recv().expect("Coudln't receive game state");

        // make sure we have the latest, drop earlier states
        while let Ok((g,b)) = receiver.try_recv() {
            game = g;
            bot = b;
        }

        let plan_result = get_plan_result(&game, &bot);
        sender.send(plan_result).expect("Failed to send plan result");
    }
}

fn bot_logic_loop_live_test(sender: Sender<PlanResult>, receiver: Receiver<(GameState, BotState)>) {
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();

    loop {
        let (game, bot) = receiver.recv().expect("Coudln't receive game state");

        update_gamepad(&mut gilrs, &mut gamepad);
        if !gamepad.select_toggled {
            thread::sleep_ms(1000/121);
            continue;
        }

        // TODO configure via args? better yet, move this all out to a separate binary after moving
        // the shared logic to a reusable lib
        let mut plan = if false {
            square_plan(&game.player)
        } else {
            offset_forward_plan(&game.player)
        };

        sender.send(PlanResult {
            plan: Some(plan.clone()),
            desired: DesiredContact::default(),
            visualization_lines: vec![],
            visualization_points: vec![],
        }).expect("Failed to send plan result");

        let mut square_errors = vec![];
        loop {
            let (game, bot) = receiver.recv().expect("Coudln't receive game state");

            let closest_index = closest_plan_index(&game.player, &plan);
            plan = plan.split_off(closest_index);

            let square_error = (plan[0].0.position - &game.player.position).norm().powf(2.0);
            square_errors.push(square_error);

            if plan.len() <= 2 {
                break;
            }

            update_gamepad(&mut gilrs, &mut gamepad);
            if !gamepad.select_toggled {
                break;
            }

            let square_error = (plan[0].0.position - &game.player.position).norm().powf(2.0);
            square_errors.push(square_error);

            thread::sleep_ms(1000/121);
        }
        println!("========================================");
        println!("Steps: {}", square_errors.len());
        println!("RMS Error: {}", (square_errors.iter().sum::<f32>() / (square_errors.len() as f32)).sqrt());
        println!("========================================");
    }
}

fn bot_io_loop(sender: Sender<(GameState, BotState)>, receiver: Receiver<PlanResult>) {
    let mut bot = BotState::default();
    let mut gilrs = Gilrs::new().unwrap();
    let mut gamepad = Gamepad::default();
    let rlbot = rlbot::init().expect("rlbot init failed");
    let mut physicist = rlbot.physicist();

    loop {
        let start = Instant::now();
        let player_index = *PLAYER_INDEX.lock().unwrap();
        //println!("player index: {:?}", player_index);
        let player_index = match player_index {
            Some(i) => i,
            None => {
                thread::sleep_ms(1000);
                continue;
            }
        };

        let tick = physicist.next_flat().expect("Missing physics tick");
        update_game_state(&mut GAME_STATE.write().unwrap(), &tick, player_index);

        send_to_bot_logic(&sender, &bot);
        thread::sleep_ms(1000 / 121); // TODO measure time taken and do a diff
        if let Ok(plan_result) = receiver.try_recv() {
            // FIXME we only want to replace it if it's better! also, what if it can't find a path
            // now, even though it could before and the old one is still ok?
            update_bot_state(&GAME_STATE.read().unwrap(), &mut bot, &plan_result);
            update_visualization(&bot, &plan_result);
        }

        let mut closest_index = 0;
        // remove part of plan that is no longer relevant since we've already passed it
        if let Some(ref mut plan) = bot.plan {
            closest_index = closest_plan_index(&GAME_STATE.read().unwrap().player, &plan);
            *plan = plan.split_off(closest_index);
        }

        update_gamepad(&mut gilrs, &mut gamepad);
        let input = if gamepad.select_toggled {
            next_rlbot_input(&GAME_STATE.read().unwrap().player, &mut bot)
        } else {
            human_input(&gamepad)
        };
        rlbot.update_player_input(input, player_index as i32);

        {
            println!("---------------------------------------------");
            println!("i: {} | steer: {} | ELAPSED: {:?}", closest_index, input.Steer, start.elapsed());
            let player = &GAME_STATE.read().unwrap().player;
            let pos = player.position;
            let v = player.velocity;
            let (roll, pitch, yaw) = player.rotation.to_euler_angles();
            //println!("ang vel: {:?}", packet.GameCars[player_index].Physics.AngularVelocity);
            println!("game: {:?},{:?},{:?},{:?},{:?},{:?},{:?}", pos.x, pos.y, pos.z, v.x, v.y, v.z, yaw);

            if let Some(ref plan) = bot.plan {
                let player = plan[0].0;
                let pos = player.position;
                let v = player.velocity;
                let (roll, pitch, yaw) = player.rotation.to_euler_angles();
                println!("plan[0]: {:?},{:?},{:?},{:?},{:?},{:?},{:?}", pos.x, pos.y, pos.z, v.x, v.y, v.z, yaw);
            }
        }
    }
}

fn run_test() {
    use std::f32::consts::PI;

    let mut game_state = GameState::default();
    //packet.GameCars[0].Physics.Rotation.Yaw = PI/2.0; // XXX opposite of the yaw in our models
    //packet.GameCars[0].Physics.Location.X = 25.0; //0.0;
    //packet.GameCars[0].Physics.Location.Y = -5567.9844; //0.0;
    //packet.GameCars[0].Physics.Location.Z = 27.106;
    //packet.GameCars[0].Physics.Velocity.Y = 382.1;
    //packet.GameCars[0].Physics.Velocity.Z = -6.956;

    //packet.GameBall.Physics.Location.X = -50.0;
    //packet.GameBall.Physics.Location.Y = -2656.1914; //0.0;
    //packet.GameBall.Physics.Location.Z = 92.0; //0.0;
    //packet.GameBall.Physics.Velocity.Y = 1418.8107;

    loop {
        //use std::time::{SystemTime, UNIX_EPOCH};
        //let start = SystemTime::now();
        //packet.GameCars[0].Physics.Rotation.Yaw = PI * (start.duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / 100000000.0).sin();
        //packet.GameCars[0].Physics.Location.Y = 4000.0 * (start.duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / 110000000.0).sin();
        //packet.GameCars[0].Physics.Location.X = 3000.0 * (start.duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / 70000000.0).sin();
        let player_index = 0;
        let input = get_test_bot_input(&game_state, player_index);
        //thread::sleep_ms(1000 / 120); // TODO measure time taken by bot and do diff
        thread::sleep_ms(1000); // FIXME testing
    }
}

fn update_bot_state(game: &GameState, bot: &mut BotState, plan_result: &PlanResult) {
    // TODO also check if existing plan is invalid, if so replace regardless
    if let Some(ref new_plan) = plan_result.plan {
        if bot.plan.is_some() {
            let new_plan_steps = new_plan.len() - 1;
            let existing_plan_steps = bot.plan.as_ref().unwrap().len() - 1 - closest_plan_index(&game.player, &bot.plan.as_ref().unwrap());
            // bail, we got a worse plan!
            if new_plan_steps > existing_plan_steps {
                return;
            }
        }

        bot.plan = Some(new_plan.clone());
        bot.turn_errors.clear();
    }
}

fn plan_lines(plan: &Plan, color: Point3<f32>) -> Vec<(Point3<f32>, Point3<f32>, Point3<f32>)> {
    let mut lines = Vec::with_capacity(plan.len());
    let pos = plan.get(0).map(|(p, _)| p.position).unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));
    let mut last_point = Point3::new(pos.x, pos.y, pos.z);
    let mut last_position = pos;
    for (ps, _) in plan {
        last_position = ps.position;
        let point = Point3::new(ps.position.x, ps.position.y, ps.position.z + 0.1);
        lines.push((last_point.clone(), point.clone(), color));
        last_point = point;
    }
    lines
}

fn update_visualization(bot: &BotState, plan_result: &PlanResult) {
    let game_state = GAME_STATE.read().unwrap();
    let PlanResult { plan, desired, visualization_lines: lines, visualization_points: points } = plan_result;

    let mut visualize_lines = LINES.write().unwrap();
    visualize_lines.clear();

    // lines directly from plan result
    visualize_lines.append(&mut lines.clone());

    // red line from player center to contact point
    let pos = game_state.player.position;
    let dpos = desired.position;
    visualize_lines.push((Point3::new(pos.x, pos.y, pos.z), Point3::new(dpos.x, dpos.y, dpos.z), Point3::new(1.0, 0.0, 0.0)));

    // white line showing best planned path
    if let Some(ref plan) = bot.plan {
        visualize_lines.append(&mut plan_lines(&plan, Point3::new(1.0, 1.0, 1.0)));
    }

    // yellow line showing most recently calculated path
    if let Some(plan) = plan {
        visualize_lines.append(&mut plan_lines(&plan, Point3::new(0.0, 1.0, 1.0)));
    }

    let mut visualize_points = POINTS.write().unwrap();
    visualize_points.clear();
    visualize_points.append(&mut points.clone());
}

fn send_to_bot_logic(sender: &Sender<(GameState, BotState)>, bot: &BotState) {
    let game = (*GAME_STATE.read().unwrap()).clone();
    let bot = bot.clone();
    sender.send((game, bot)).expect("Sending to bot logic failed");
}

fn turn_plan(current: &PlayerState, angle: f32) -> Vec<(PlayerState, BrickControllerState)> {
    let mut plan = vec![];
    let current_heading = current.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let desired_heading = Rotation3::from_euler_angles(0.0, 0.0, angle) * current_heading;
    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;
    controller.steer = if angle < 0.0 { Steer::Right } else { Steer::Left };

    // iterate till dot product is minimized (ie we match the desired heading)
    let mut last_dot = std::f32::MIN;
    let mut player = current.clone();
    loop {
        let new_player = next_player_state(&player, &controller, TICK);
        let heading = player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
        let dot = na::dot(&heading, &desired_heading);
        if dot > last_dot {
            plan.push((new_player, controller));
            player = new_player;
            last_dot = dot;
        } else {
            break
        }
    }

    plan
}

fn forward_plan(current: &PlayerState, distance: f32) -> Vec<(PlayerState, BrickControllerState)> {
    let mut plan = vec![];

    let mut controller = BrickControllerState::new();
    controller.throttle = Throttle::Forward;

    let mut player = current.clone();
    while (player.position - current.position).norm() < distance {
        player = next_player_state(&player, &controller, TICK);
        plan.push((player, controller));
    }
    plan
}

fn square_plan(current: &PlayerState) -> Vec<(PlayerState, BrickControllerState)> {
    let mut plan = vec![];
    let mut player = current.clone();
    let max_throttle_speed = 1545.0; // FIXME put in common lib
    player.velocity = max_throttle_speed * Unit::new_normalize(player.velocity).unwrap();
    plan.push((player, BrickControllerState::new()));
    for _ in 0..4 {
        let mut plan_part = forward_plan(&plan[plan.len() - 1].0, 1000.0);
        plan.append(&mut plan_part);
        let mut plan_part = turn_plan(&plan[plan.len() - 1].0, -PI/2.0);
        plan.append(&mut plan_part);
    }
    plan
}

fn offset_forward_plan(current: &PlayerState) -> Vec<(PlayerState, BrickControllerState)> {
    let mut offset_player = current.clone();
    let heading = offset_player.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0);
    let clockwise_90_rotation = Rotation3::from_euler_angles(0.0, 0.0, -PI/2.0);
    let right = clockwise_90_rotation * heading;
    offset_player.position += 200.0 * right;

    forward_plan(&offset_player, 4000.0)
}


fn simulate_over_time() {
    thread::sleep_ms(5000);
    let initial_game_state: GameState;
    let mut bot = BotState::default();
    {
        let mut game_state = GAME_STATE.write().unwrap();
        game_state.ball.position = Vector3::new(2000.0, 1000.0, 89.0);
        game_state.ball.velocity = Vector3::new(0.0, 0.0, 0.0);

        game_state.player.position = Vector3::new(0.0, 0.0, 0.0);
        game_state.player.velocity = Vector3::new(0.0, 0.0, 0.0);
        game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0);
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI/2.0);
        //game_state.player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI);

        initial_game_state = game_state.clone();
    }

    let mut last_plan = vec![];
    loop {
        {
            let mut game_state = GAME_STATE.read().unwrap();
            let plan_result = get_plan_result(&game_state, &bot);
            update_bot_state(&game_state, &mut bot, &plan_result);
            update_visualization(&bot, &plan_result);
            if plan_result.plan.is_none() {
                thread::sleep_ms(5000);
                continue;
            }
        }

        if let Some(plan) = bot.plan.clone() {
            let mut game_state = GAME_STATE.write().unwrap();
            let i = closest_plan_index(&game_state.player, &plan);
            if plan.len() >= i + 2 {
                game_state.player = plan[i + 1].0;
                last_plan = plan.clone();
                // TODO move the ball. but ball velocity is zero for now
            } else {
                // we're at the goal, so start over
                *game_state = initial_game_state.clone();
                bot.plan = None;
            }
            if plan.len() - i < 20 {
                thread::sleep_ms(1000/4);
            } else {
                thread::sleep_ms(1000/121);
            }
        } else {
            // let mut game_state = GAME_STATE.write().unwrap();
            // let i = closest_plan_index(&game_state.player, &last_plan);
            // game_state.player = last_plan[i + 1].0;
            unimplemented!("go forward 2")
        }
        //thread::sleep_ms(1000/1);
    }
}

fn next_rlbot_input(current_player: &PlayerState, bot: &mut BotState) -> rlbot::ffi::PlayerInput {
    {
        // XXX there must be a reason why this happens, but BRAIN must be locked before
        // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
        let mut p = BRAIN2.lock().expect("Failed to get lock on BRAIN");
        let mut rh = RELOAD_HANDLER2.lock().expect("Failed to get lock on RELOAD_HANDLER");
        rh.update(BrainPlugin::reload_callback, &mut p);
    }

    if let Some(ref x) = BRAIN2.lock().unwrap().lib {
        // TODO cache
        let next_input: Symbol<NextInputFunc> = unsafe {
            x.lib.get(b"next_input\0").unwrap()
        };

        next_input(&current_player, bot)
    } else {
        panic!("We need the brain dynamic library!");
    }
}


type PlayFunc = extern fn (game: &GameState, bot: &BotState) -> PlanResult;
type HybridAStarFunc = extern fn (current: &PlayerState, desired: &DesiredContact, config: &SearchConfig) -> PlanResult;
type SSPSFunc = extern fn (ball: &BallState, desired_ball_position: &Vector3<f32>) -> DesiredContact;
type NextInputFunc = extern fn (current_player: &PlayerState, bot: &mut BotState) -> rlbot::ffi::PlayerInput;
type ClosestPlanIndexFunc = extern fn (current_player: &PlayerState, plan: &Plan) -> usize;
type NextPlayerStateFunc = fn (current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState;

fn closest_plan_index(current_player: &PlayerState, plan: &Plan) -> usize {
    {
        // XXX there must be a reason why this happens, but BRAIN must be locked before
        // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
        let mut p = BRAIN2.lock().expect("Failed to get lock on BRAIN");
        let mut rh = RELOAD_HANDLER2.lock().expect("Failed to get lock on RELOAD_HANDLER");
        rh.update(BrainPlugin::reload_callback, &mut p);
    }

    if let Some(ref x) = BRAIN2.lock().unwrap().lib {
        // TODO cache
        let closest_plan_index: Symbol<ClosestPlanIndexFunc> = unsafe {
            x.lib.get(b"closest_plan_index\0").unwrap()
        };

        closest_plan_index(&current_player, &plan)
    } else {
        panic!("We need the brain dynamic library!");
    }
}

fn next_player_state(current: &PlayerState, controller: &BrickControllerState, time_step: f32) -> PlayerState {
    // FIXME is there a way to unlock without a made up scope?
    {
        // XXX there must be a reason why this happens, but BRAIN must be locked before
        // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
        let mut p = BRAIN.lock().expect("Failed to get lock on BRAIN");
        let mut rh = RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER");
        rh.update(BrainPlugin::reload_callback, &mut p);
    }

    if let Some(ref x) = BRAIN.lock().unwrap().lib {
        // TODO cache
        let next_player_state: Symbol<NextPlayerStateFunc> = unsafe {
            x.lib.get(b"next_player_state\0").unwrap()
        };

        next_player_state(&current, &controller, time_step)
    } else {
        panic!("We need the brain dynamic library!");
    }
}

fn get_plan_result(game_state: &GameState, bot: &BotState) -> PlanResult {
    // FIXME is there a way to unlock without a made up scope?
    {
        // XXX there must be a reason why this happens, but BRAIN must be locked before
        // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
        let mut p = BRAIN.lock().expect("Failed to get lock on BRAIN");
        let mut rh = RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER");
        rh.update(BrainPlugin::reload_callback, &mut p);
    }

    if let Some(ref x) = BRAIN.lock().unwrap().lib {
        // TODO cache
        let play: Symbol<PlayFunc> = unsafe {
            x.lib.get(b"play\0").unwrap()
        };
        let hybrid_a_star: Symbol<HybridAStarFunc> = unsafe {
            x.lib.get(b"hybrid_a_star\0").unwrap()
        };

        play(&game_state, &bot)
    } else {
        panic!("We need the brain dynamic library!");
        //PlanResult::default()
    }
}

fn get_test_bot_input(game_state: &GameState, player_index: usize) -> rlbot::ffi::PlayerInput {
    let mut bot = BotState::default();
    let mut input = rlbot::ffi::PlayerInput::default();

    // FIXME is there a way to unlock without a made up scope?
    {
        // XXX there must be a reason why this happens, but BRAIN must be locked before
        // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
        let mut p = BRAIN.lock().expect("Failed to get lock on BRAIN");
        let mut rh = RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER");
        rh.update(BrainPlugin::reload_callback, &mut p);
    }

    // TODO get extra visualization data and desired controller state from BRAIN
    if let Some(ref x) = BRAIN.lock().unwrap().lib {
        // TODO cache
        let play: Symbol<PlayFunc> = unsafe {
            x.lib.get(b"play\0").unwrap()
        };
        let hybrid_a_star: Symbol<HybridAStarFunc> = unsafe {
            x.lib.get(b"hybrid_a_star\0").unwrap()
        };
        let simple_desired_contact: Symbol<SSPSFunc> = unsafe {
            x.lib.get(b"simple_desired_contact\0").unwrap()
        };


        let start = Instant::now();
        println!("PLAN DURATION: {:?}", start.elapsed());

        let manual = true;
        let mut extra_lines = vec![];
        let result = if manual {
            let desired_ball_position = Vector3::new(0.0, 5140.0, 320.0);
            let dc = simple_desired_contact(&game_state.ball, &desired_ball_position);
            let dh = 1000.0 * dc.heading;
            let bp = game_state.ball.position;
            // velocity delta line
            extra_lines.push((
                Point3::new(bp.x       , bp.y       , bp.z       ),
                Point3::new(bp.x + dh.x, bp.y + dh.y, bp.z + dh.z),
                Point3::new(1.0, 1.0, 0.0)
            ));


            // let now = SystemTime::now();
            // use std::time::{SystemTime, UNIX_EPOCH};
            let mut desired_contact = DesiredContact::default();
            desired_contact.position.x = 52.550236; //101.0; //300.0 * (now.duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / 10000000.0).sin();
            desired_contact.position.y = -2563.3354; //1000.0 + 300.0 * (now.duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f32 / 7000000.0).sin();
            desired_contact.position.z = 90.99978;
            desired_contact.heading = dc.heading;
            let mut config = SearchConfig::default();
            config.step_duration = 20.0/120.0;
            hybrid_a_star(&game_state.player, &desired_contact, &config)
        } else {
            play(&game_state, &bot)
        };
        println!("TOOK: {:?}", start.elapsed());

        update_bot_state(&game_state, &mut bot, &result);
        update_visualization(&bot, &result);
        println!("desired contact position: {:?}", result.desired.position);
    }

    input
}


fn main() -> Result<(), Box<Error>> {
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.parse())
                      .unwrap_or_else(|e| e.exit());

    let test_bot = args.get_bool("--bot-test");
    if args.get_bool("--bot") || test_bot {
        thread::spawn(move || {
            loop {
                let t = thread::spawn(move || {
                    if test_bot {
                        panic::catch_unwind(run_bot_live_test);
                    } else {
                        panic::catch_unwind(run_bot);
                    }
                });
                t.join();
                println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
                println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
                println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
                println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
                println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

                thread::sleep_ms(1000);
            }
        });
    } else if args.get_bool("--simulate") {
        thread::spawn(simulate_over_time);
    } else if args.get_bool("--test") {
        thread::spawn(run_test);
    }

    run_visualization();

    Ok(())
}
