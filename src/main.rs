extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate tls_api;
extern crate kiss3d;
extern crate nalgebra as na;

#[macro_use]
extern crate lazy_static;

pub mod game_data;
pub mod game_data_grpc;
pub mod state;

use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::sync::{RwLock};
use std::f32;

use game_data::*;
use game_data_grpc::*;
use state::*;

use na::{Vector3, Translation3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;

static BALL_RADIUS: f32 = 93.143;


lazy_static! {
    // batmobile
    static ref CAR_DIMENSIONS: Vector3<f32> = Vector3::new(128.82, 84.67, 29.39);
    static ref PIVOT_OFFSET: Vector3<f32> = Vector3::new(9.008, 0.0, 12.094);

    static ref GAME_STATE: RwLock<GameState> = {
        RwLock::new(GameState {
            ball: BallState { position: Vector3::new(0.0, 0.0, 0.0) },
            player: PlayerState { position: Vector3::new(0.0, 0.0, 0.0), rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0) },
        })
    };

    static ref FILE: RwLock<File> = {
        RwLock::new(File::create("foo.csv").unwrap())
    };
}

static mut record: bool = true;


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
        game_state.ball.position = Vector3::new(-bl.x, bl.y, bl.z); // x should be positive towards right, it only makes sense

        let pl = player.get_location();
        let pr = player.get_rotation();
        game_state.player.position = Vector3::new(-pl.x, pl.y, pl.z); // x should be positive towards right, it only makes sense
        game_state.player.rotation = UnitQuaternion::from_euler_angles(-pr.roll, pr.pitch, -pr.yaw);

        let mut file = FILE.write().unwrap();
        unsafe {
            if record {
                file.write_all(format!("{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    packet.get_game_info().seconds_elapsed,
                    -bl.x,
                    bl.y,
                    bl.z,
                    -ball.get_velocity().x,
                    ball.get_velocity().y,
                    ball.get_velocity().z,
                    -ball.get_acceleration().x,
                    ball.get_acceleration().y,
                    ball.get_acceleration().z,
                    -ball.get_angular_velocity().x,
                    ball.get_angular_velocity().y,
                    ball.get_angular_velocity().z,
                ).as_bytes());
                //println!("ball z: {}", bl.z);
            }
        }

        controller_state.throttle = 1.0;
        //controller_state.steer = 1.0;

        grpc::SingleResponse::completed(controller_state)
    }
}

fn main() {
    // visualization
    thread::spawn(move || {
        let mut window = Window::new("Rocket League Visualization");

        // we're dividing everything by 1000 until we can set the camera up to be more zoomed out
        let mut sphere = window.add_sphere(BALL_RADIUS / 1000.0);
        let mut car = window.add_cube(CAR_DIMENSIONS.x/1000.0, CAR_DIMENSIONS.y/1000.0, CAR_DIMENSIONS.z/1000.0);

        sphere.set_color(0.8, 0.8, 0.8);
        car.set_color(0.1, 0.4, 1.0);

        window.set_light(Light::StickToCamera);

        let mut floor = window.add_cube(8.0, 10.0, 0.001);
        floor.set_surface_rendering_activation(false);
        floor.set_points_size(0.1);
        floor.set_lines_width(0.1);
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

    // server
    let mut server = grpc::ServerBuilder::new_plain();
    server.http.set_port(34865);
    server.add_service(BotServer::new_service_def(BotImpl));
    server.http.set_cpu_pool_threads(4);
    let _server = server.build().expect("server");
    loop {
        thread::park();
    }
}
