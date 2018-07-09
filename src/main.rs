extern crate kiss3d;
extern crate nalgebra as na;
extern crate dynamic_reload;
extern crate state;

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::net::TcpListener;
use std::io::prelude::*;
use std::thread;
use std::sync::{Arc, RwLock, Mutex};
use std::f32;
use std::path::Path;

use state::*;

use na::{Vector3, Translation3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::MeshManager;

use dynamic_reload::{DynamicReload, Lib, Symbol, Search, PlatformName, UpdateState};


lazy_static! {
    // batmobile
    static ref CAR_DIMENSIONS: Vector3<f32> = Vector3::new(128.82, 84.67, 29.39);
    static ref PIVOT_OFFSET: Vector3<f32> = Vector3::new(9.008, 0.0, 12.094);

    static ref PLAYER_INDEX: Mutex<Option<usize>> = Mutex::new(None);

    static ref GAME_STATE: RwLock<GameState> = {
        RwLock::new(GameState {
            ball: BallState {
                position: Vector3::new(0.0, 0.0, 0.0),
                velocity: Vector3::new(0.0, 0.0, 0.0),
                angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            },
            player: PlayerState {
                position: Vector3::new(0.0, 0.0, 0.0),
                velocity: Vector3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0)
            },
        })
    };

    static ref RELOAD_HANDLER: Mutex<DynamicReload<'static>> = {
        Mutex::new(
            DynamicReload::new(Some(vec!["predict/target/debug"]),
                               Some("target/debug"),
                               Search::Default)
        )
    };

    static ref PREDICT: Mutex<PredictPlugin> = {
        let lib = match RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER").add_library("predict", PlatformName::Yes) {
            Ok(lib) => lib,
            Err(e) => {
                panic!("Unable to load dynamic lib, err {:?}", e);
            }
        };
        Mutex::new(PredictPlugin { lib: Some(lib) })
    };
}


struct PredictPlugin {
    lib: Option<Arc<Lib>>
}

impl PredictPlugin {
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

        let ball = packet.get_ball();
        let player = &packet.players[packet.player_index as usize];

        let bl = ball.get_location();
        let bv = ball.get_velocity();
        game_state.ball.position = Vector3::new(-bl.x, bl.y, bl.z); // x should be positive towards right, it only makes sense
        game_state.ball.velocity = Vector3::new(-bv.x, bv.y, bv.z); // x should be positive towards right, it only makes sense

        let pl = player.get_location();
        let pv = player.get_velocity();
        let pr = player.get_rotation();
        game_state.player.position = Vector3::new(-pl.x, pl.y, pl.z); // x should be positive towards right, it only makes sense
        game_state.player.velocity = Vector3::new(-pv.x, pv.y, pv.z); // x should be positive towards right, it only makes sense
        game_state.player.rotation = UnitQuaternion::from_euler_angles(-pr.roll, pr.pitch, -pr.yaw);

        // FIXME is there a way to unlock without a made up scope?
        {
            // XXX there must be a reason why this happens, but PREDICT must be locked before
            // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
            let mut p = PREDICT.lock().expect("Failed to get lock on PREDICT");
            let mut rh = RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER");
            rh.update(PredictPlugin::reload_callback, &mut p);
        }

        // TODO get extra visualization data and desired controller state from PREDICT
        if let Some(ref x) = PREDICT.lock().unwrap().lib {
            // TODO cache
            let predict_test: Symbol<extern "C" fn() -> Vector3<f32>> = unsafe {
                x.lib.get(b"predict_test\0").unwrap()
            };
            println!("predict test: {}", predict_test());
        }

        grpc::SingleResponse::completed(controller_state)
    }
}
*/

fn main() {
    // visualization
    thread::spawn(move || {
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

            // we're dividing position by 1000 until we can set the camera up to be more zoomed out
            sphere.set_local_translation(Translation3::from_vector(game_state.ball.position.map(|c| c / 1000.0)));

            // we're dividing position by 1000 until we can set the camera up to be more zoomed out
            let hitbox_position = game_state.player.position.map(|c| c / 1000.0) + PIVOT_OFFSET.map(|c| c / 1000.0);
            car.set_local_translation(Translation3::from_vector(hitbox_position));
            car.set_local_rotation(game_state.player.rotation); // FIXME need to rotate about the pivot, not center
        }
    });


    thread::spawn(|| {
        loop {
            println!("player index: {:?}", *PLAYER_INDEX.lock().unwrap());
            thread::sleep_ms(1000); // TODO measure time taken by bot and do diff
        }
    });

    // obtain port to communicate with python agent. must match the port the python agent is configured to send to!
    let mut port_file = File::open("port.cfg").expect("port.cfg file not found");
    let mut contents = String::new();
    port_file.read_to_string(&mut contents).expect("something went wrong reading the port.cfg file");
    let port = contents.trim().parse::<u16>().expect(&format!("couldn't parse port: {}", contents));

    // super basic tcp server. only used to get the right index from the python agent for now.
    let listener = TcpListener::bind(("127.0.0.1", port)).expect(&format!("Failed to bind port {}", port));
    let mut message = String::new();
    for stream in listener.incoming() {
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
    }
}
