extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate tls_api;

pub mod game_data;
pub mod game_data_grpc;

use std::thread;

use game_data::*;
use game_data_grpc::*;

struct BotImpl;

impl Bot for BotImpl {
    fn get_controller_state(&self, _m: grpc::RequestOptions, packet: GameTickPacket) -> grpc::SingleResponse<ControllerState> {
        let mut controller_state = ControllerState::new();
        controller_state.throttle = -1.0;
        controller_state.steer = -1.0;
        grpc::SingleResponse::completed(controller_state)
    }
}

fn main() {
    let mut server = grpc::ServerBuilder::new_plain();
    server.http.set_port(34865);
    server.add_service(BotServer::new_service_def(BotImpl));
    server.http.set_cpu_pool_threads(4);
    let _server = server.build().expect("server");

    loop {
        thread::park();
    }
}