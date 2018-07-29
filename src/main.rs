extern crate kiss3d;
extern crate nalgebra as na;
extern crate dynamic_reload;
extern crate state;
extern crate rlbot;

#[macro_use]
extern crate lazy_static;

use std::panic;
use std::fs::File;
use std::net::TcpListener;
use std::io::prelude::*;
use std::thread;
use std::sync::{Arc, RwLock, Mutex};
use std::f32;
use std::f32::consts::PI;
use std::path::Path;

use state::*;

use na::{ Unit, Vector3, Point3, Translation3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::MeshManager;

use dynamic_reload::{DynamicReload, Lib, Symbol, Search, PlatformName, UpdateState};


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
            DynamicReload::new(Some(vec!["brain/target/release"]),
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

// TODO refactor for no reliance on grpc
/*
struct BotImpl;

impl Bot for BotImpl {
    fn get_controller_state(&self, _m: grpc::RequestOptions, packet: GameTickPacket) -> grpc::SingleResponse<ControllerState> {
        let mut game_state = GAME_STATE.write().unwrap();
        let mut controller_state = ControllerState::new();

        // no players, ie game hasn't started or has ended, or just in menus, etc
        if packet.players.len() == 0usize {
            return grpc::SingleResponse::completed(controller_state);
        }


        grpc::SingleResponse::completed(controller_state)
    }
}
*/

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
        let game_state = &GAME_STATE.read().unwrap();
        let lines = &LINES.read().unwrap();
        let points = &POINTS.read().unwrap();

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        sphere.set_local_translation(Translation3::from_vector(game_state.ball.position.map(|c| c / 1000.0)));

        // we're dividing position by 1000 until we can set the camera up to be more zoomed out
        let hitbox_position = game_state.player.position.map(|c| c / 1000.0) + PIVOT_OFFSET.map(|c| c / 1000.0);
        car.set_local_translation(Translation3::from_vector(hitbox_position));
        car.set_local_rotation(game_state.player.rotation); // FIXME need to rotate about the pivot, not center

        for l in lines.iter() {
            window.draw_line(&Point3::new(l.0.x / 1000.0, l.0.y / 1000.0, l.0.z / 1000.0), &Point3::new(l.1.x / 1000.0, l.1.y / 1000.0, l.1.z / 1000.0), &l.2);
        }

        for p in points.iter() {
            window.draw_point(&Point3::new(p.0.x / 1000.0, p.0.y / 1000.0, p.0.z / 1000.0), &p.1);
        }
    }
}

/// main bot playing loop
fn run_bot() {
    let mut packet = rlbot::LiveDataPacket::default();
    loop {
        let player_index = *PLAYER_INDEX.lock().unwrap();
        //println!("player index: {:?}", player_index);
        let player_index = match player_index {
            Some(i) => i,
            None => {
                thread::sleep_ms(1000);
                continue;
            }
        };

        rlbot::update_live_data_packet(&mut packet);
        //println!("{:?}", packet.GameBall);

        let input = get_bot_input(&packet, player_index);
        rlbot::update_player_input(input, player_index as i32);

        thread::sleep_ms(1000 / 120); // TODO measure time taken by bot and do diff
    }
}

fn run_test() {
    use std::f32::consts::PI;
    let mut packet = rlbot::LiveDataPacket::default();
    packet.GameCars[0].Physics.Rotation.Yaw = -PI/2.0;
    packet.GameCars[0].Physics.Location.Y = -3000.0;
    packet.GameCars[0].Physics.Location.X = -2000.0;
    loop {
        println!("packet player2 location: {:?}", packet.GameCars[0].Physics.Location);
        let player_index = 0;
        let input = get_bot_input(&packet, player_index);
        //thread::sleep_ms(1000 / 120); // TODO measure time taken by bot and do diff
        thread::sleep_ms(1000); // FIXME testing
    }
}


/// updates our game state, which is a representation of the packet, but with our own data types etc
fn update_game_state(game_state: &mut GameState, packet: &rlbot::LiveDataPacket, player_index: usize) {
    let ball = packet.GameBall;
    let player = packet.GameCars[player_index];

    let bl = ball.Physics.Location;
    let bv = ball.Physics.Velocity;
    game_state.ball.position = Vector3::new(-bl.X, bl.Y, bl.Z); // x should be positive towards right, it only makes sense
    game_state.ball.velocity = Vector3::new(-bv.X, bv.Y, bv.Z); // x should be positive towards right, it only makes sense

    let pl = player.Physics.Location;
    let pv = player.Physics.Velocity;
    let pr = player.Physics.Rotation;
    game_state.player.position = Vector3::new(-pl.X, pl.Y, pl.Z); // x should be positive towards right, it only makes sense
    game_state.player.velocity = Vector3::new(-pv.X, pv.Y, pv.Z); // x should be positive towards right, it only makes sense
    game_state.player.rotation = UnitQuaternion::from_euler_angles(-pr.Roll, pr.Pitch, -pr.Yaw);
    game_state.player.team = match player.Team {
        0 => Team::Blue,
        1 => Team::Orange,
        _ => unimplemented!(),
    };
}

type PlayFunc = extern fn (game: &GameState) -> PlanResult;

/// this is the entry point for custom logic for this specific bot
fn get_bot_input(packet: &rlbot::LiveDataPacket, player_index: usize) -> rlbot::PlayerInput {
    let mut input = rlbot::PlayerInput::default();

    //println!("packet player location: {:?}", packet.GameCars[0].Physics.Location);
    update_game_state(&mut GAME_STATE.write().unwrap(), &packet, player_index);

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

        let game_state = &GAME_STATE.read().unwrap();
        //println!("player: {:?}", game_state.player);
        let PlanResult {
            plan: mut path, desired,
            visualization_lines: mut lines,
            visualization_points: mut points,
        } = play(&game_state);

        let mut visualize_lines = LINES.write().unwrap();
        visualize_lines.clear();

        // red line from player center to contact point
        let pos = game_state.player.position;
        let dpos = desired.player.unwrap().position; 
        visualize_lines.push((Point3::new(pos.x, pos.y, pos.z), Point3::new(dpos.x, dpos.y, dpos.z), Point3::new(1.0, 0.0, 0.0)));

        // white line showing planned path
        if let Some(path) = path {
            // first item in path is initial position, so we go to second index. may be missing if we are already there!
            if let Some((_, controller)) = path.get(1) {
                input = convert_controller_to_rlbot_input(&controller);
            }
            let pos = game_state.player.position;
            let mut last_point = Point3::new(pos.x, pos.y, pos.z);
            let mut last_position = pos;
            for (ps, _) in &path {
                last_position = ps.position;
                let point = Point3::new(ps.position.x, ps.position.y, ps.position.z + 0.1);
                visualize_lines.push((last_point.clone(), point.clone(), Point3::new(1.0, 1.0, 1.0)));
                last_point = point;
            }
        }
        visualize_lines.append(&mut lines);


        let mut visualize_points = POINTS.write().unwrap();
        visualize_points.clear();
        visualize_points.append(&mut points);
    }

    input
}

fn convert_controller_to_rlbot_input(controller: &BrickControllerState) -> rlbot::PlayerInput {
    rlbot::PlayerInput {
        Throttle: match controller.throttle {
            Throttle::Idle => 0.0,
            Throttle::Forward => 1.0,
            Throttle::Reverse => -1.0,
        },
        Steer: match controller.steer {
            Steer::Straight => 0.0,
            Steer::Left => -1.0,
            Steer::Right => 1.0,
        },
        Pitch: 0.0, // brick is a brick
        Yaw: 0.0, // brick is a brick
        Roll: 0.0, // brick is a brick
        Jump: false, // brick is a brick
        Boost: false, // brick is a brick
        Handbrake: false, // brick is a brick
    }
}

// obtain port to communicate with python agent. must match the port the python agent is configured to send to!
fn get_port(filename: &str) -> u16 {
    let mut port_file = File::open(filename).expect(&format!("{} file not found", filename));
    let mut contents = String::new();
    port_file.read_to_string(&mut contents).expect("something went wrong reading the port.cfg file");
    contents.trim().parse::<u16>().expect(&format!("couldn't parse port: {}", contents))
}

/// super basic tcp server. only used to get the right index from the python agent for now.
fn run_server() {
    let port = get_port("port.cfg");
    let listener = TcpListener::bind(("127.0.0.1", port)).expect(&format!("Failed to bind port {}", port));
    let mut message = String::new();
    for stream in listener.incoming() {
        message.clear();
        let mut stream = stream.expect("Failed to accept connection");
        stream.read_to_string(&mut message).expect("Couldn't read tcp message to utf8 string");
        let mut split_message = message.split_whitespace();

        let cmd = split_message.next().expect("Missing cmd");
        let _name = split_message.next().expect("Missing name");
        let _team = split_message.next().expect("Missing team");
        let index = split_message.next().expect("Missing index");
        let index = index.trim().parse::<usize>().expect(&format!("Couldn't parse index {}", index));

        // TODO use this
        let dllPath = split_message.next().expect("Missing dllPath");

        match cmd {
            "add" => *PLAYER_INDEX.lock().unwrap() = Some(index),
            "remove" => *PLAYER_INDEX.lock().unwrap() = None,
            _ => unimplemented!(),
        };
        println!("---------------------------------------------");
        println!("message: {}", message);
        println!("cmd: {}, player_index: {:?}", cmd, *PLAYER_INDEX.lock().unwrap());
    }
}

fn main() {
    thread::spawn(|| {
        loop {
            let t = thread::spawn(|| {
                panic::catch_unwind(run_bot);
                //panic::catch_unwind(run_test);
            });
            t.join();
            thread::sleep_ms(1000);
        }
    });

    //run_visualization();
    thread::spawn(run_visualization);
    run_server();
}
