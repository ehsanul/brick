extern crate nalgebra as na;
extern crate rlbot;

use na::{Vector3, UnitQuaternion};
use std::f32::consts::PI;

// XXX must confirm. this might include height of the ball in free play when it first starts
// floating above the ground, which would be no good. 91.25 has been seen in RLBounce
pub static BALL_RADIUS: f32 = 93.143;

#[derive(Debug, Default)]
pub struct GameState {
    pub ball: BallState,
    pub player: PlayerState,
}

// FIXME check if this order matches up with team integers we get from rlbot interface
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Team {
    Blue,
    Orange,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>, // FIXME switch to Rotation3!
    pub team: Team,
    //pub rotation: Rotation3<f32>,
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0),
            team: Team::Blue,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BallState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
}

impl Default for BallState {
    fn default() -> BallState {
        BallState {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            //rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Throttle {
    Forward,
    Reverse,
    Idle,
}

impl Throttle {
    pub fn value(&self) -> f32 {
        match *self {
            Throttle::Forward => 1.0,
            Throttle::Reverse => -1.0,
            Throttle::Idle => 0.0,
        }
    }
}

// XXX this was copied from the grpc generated game_data.rs file, from the ControllerState struct
#[derive(Copy, Clone, Debug)]
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
            throttle: Throttle::Idle,
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

// XXX we may want to use different internal structs, since in some cases we may care about
// position but not velocity, and vice versa
pub struct DesiredState {
    pub player: Option<PlayerState>,
    pub ball: Option<BallState>,
}

/// updates our game state, which is a representation of the packet, but with our own data types etc
pub fn update_game_state(game_state: &mut GameState, packet: &rlbot::LiveDataPacket, player_index: usize) {
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
