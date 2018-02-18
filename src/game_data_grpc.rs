// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Bot {
    fn get_controller_state(&self, o: ::grpc::RequestOptions, p: super::game_data::GameTickPacket) -> ::grpc::SingleResponse<super::game_data::ControllerState>;
}

// client

pub struct BotClient {
    grpc_client: ::grpc::Client,
    method_GetControllerState: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::game_data::GameTickPacket, super::game_data::ControllerState>>,
}

impl BotClient {
    pub fn with_client(grpc_client: ::grpc::Client) -> Self {
        BotClient {
            grpc_client: grpc_client,
            method_GetControllerState: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/rlbot.api.Bot/GetControllerState".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new_plain(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_plain(host, port, conf).map(|c| {
            BotClient::with_client(c)
        })
    }
    pub fn new_tls<C : ::tls_api::TlsConnector>(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_tls::<C>(host, port, conf).map(|c| {
            BotClient::with_client(c)
        })
    }
}

impl Bot for BotClient {
    fn get_controller_state(&self, o: ::grpc::RequestOptions, p: super::game_data::GameTickPacket) -> ::grpc::SingleResponse<super::game_data::ControllerState> {
        self.grpc_client.call_unary(o, p, self.method_GetControllerState.clone())
    }
}

// server

pub struct BotServer;


impl BotServer {
    pub fn new_service_def<H : Bot + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/rlbot.api.Bot",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/rlbot.api.Bot/GetControllerState".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_controller_state(o, p))
                    },
                ),
            ],
        )
    }
}
