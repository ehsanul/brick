extern crate nalgebra as na;

use na::{Vector3, UnitQuaternion};

pub struct GameState {
    pub ball: BallState,
    pub player: PlayerState,
}

#[derive(Clone)]
pub struct PlayerState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

#[derive(Clone)]
pub struct BallState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

#[derive(PartialEq)]
pub enum Steer {
    Right,
    Left,
    Straight,
}

impl Steer {
    pub fn value(&self) -> f32 {
        match *self {
            Steer::Right => 1.0,
            Steer::Left => -1.0,
            Steer::Straight => 0.0,
        }
    }
}


#[derive(PartialEq)]
pub enum Throttle {
    Forward,
    Reverse,
    Rest,
}

impl Throttle {
    pub fn value(&self) -> f32 {
        match *self {
            Throttle::Forward => 1.0,
            Throttle::Reverse => -1.0,
            Throttle::Rest => 0.0,
        }
    }
}

// XXX this was copied from the grpc generated game_data.rs file, from the ControllerState struct
pub struct BrickControllerState {
    pub throttle: Throttle,
    pub steer: Steer,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub jump: bool,
    pub boost: bool,
    pub handbrake: bool,
}

impl BrickControllerState {
    pub fn new() -> BrickControllerState {
        BrickControllerState {
            throttle: Throttle::Rest,
            steer: Steer::Straight,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            jump: false,
            boost: false,
            handbrake: false,
        }
    }
}
