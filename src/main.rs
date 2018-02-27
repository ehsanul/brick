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

use std::thread;
use std::sync::{RwLock};
use std::f32;

use game_data::*;
use game_data_grpc::*;

use na::{Vector3, Translation3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;

struct GameState {
    ball: BallState,
    player: PlayerState,
}

struct PlayerState {
    coordinates: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
}

struct BallState {
    coordinates: Vector3<f32>,
}


lazy_static! {
    static ref GAME_STATE: RwLock<GameState> = {
        RwLock::new(GameState {
            ball: BallState { coordinates: Vector3::new(0.0, 0.0, 0.0) },
            player: PlayerState { coordinates: Vector3::new(0.0, 0.0, 0.0), rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0) },
        })
    };
}

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
        game_state.ball.coordinates = Vector3::new(-bl.x, bl.y, bl.z); // x should be positive towards right, it only makes sense

        let pl = player.get_location();
        let pr = player.get_rotation();
        game_state.player.coordinates = Vector3::new(-pl.x, pl.y, pl.z); // x should be positive towards right, it only makes sense
        game_state.player.rotation = UnitQuaternion::from_euler_angles(-pr.roll, pr.pitch, -pr.yaw);

        // slightly modified atba, but broken when ball is towards blue side relative to player :/
        /*
        controller_state.throttle = 1.0;

        // note x-axis is flipped when blue facing orange (left is +x), hence negate
        let mut xd = -(ball.get_location().x - player.get_location().x);
        let yd = ball.get_location().y - player.get_location().y;

        // naieve, but: if ball is to left of goal, go more left
        if ball.get_location().x > 0.0 { xd -= 40.0 }
        // naieve, but: if ball is to right of goal, go more right
        if ball.get_location().x < 0.0 { xd += 40.0 }

        let mut theta = yd.atan2(xd);
        let angle = f32::consts::PI - theta;
        //println!("yd: {}, xd: {}, angle: {}; theta: {}, yaw: {}", yd, xd, angle, theta, player.get_rotation().yaw);
        //if player.get_rotation().yaw < angle {
            controller_state.steer = 1.0;
        //} else {
        //    controller_state.steer = -1.0;
        //}
        */

        grpc::SingleResponse::completed(controller_state)
    }
}

fn main() {
    // visualization
    thread::spawn(move || {
        let mut window = Window::new("Rocket League Visualization");
        let mut sphere = window.add_sphere(0.1);
        let mut car = window.add_cube(0.2, 0.1, 0.05);

        sphere.set_color(0.8, 0.8, 0.8);
        car.set_color(0.1, 0.4, 1.0);

        window.set_light(Light::StickToCamera);

        let mut floor = window.add_cube(8.0, 10.0, 0.01);
        floor.set_surface_rendering_activation(false);
        floor.set_points_size(0.1);
        floor.set_lines_width(0.1);
        while window.render() {
            let game_state = &GAME_STATE.read().unwrap();

            // we're dividing coordinates by 1000 until we can set the camera up to be more zoomed out
            sphere.set_local_translation(Translation3::from_vector(game_state.ball.coordinates.map(|c| c / 1000.0)));

            // we're dividing coordinates by 1000 until we can set the camera up to be more zoomed out
            car.set_local_translation(Translation3::from_vector(game_state.player.coordinates.map(|c| c / 1000.0)));
            car.set_local_rotation(game_state.player.rotation);
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