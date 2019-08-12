extern crate nalgebra as na;
extern crate rlbot;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use na::{Point3, Quaternion, Rotation3, UnitQuaternion, Vector3};
use std::collections::VecDeque;
use std::f32::consts::PI;

// general constants
pub const FPS: f32 = 120.0;
pub const TICK: f32 = 1.0 / FPS; // matches RL's internal fixed physics tick rate

// arena constants
pub const SIDE_WALL_DISTANCE: f32 = 4096.0;
pub const BACK_WALL_DISTANCE: f32 = 5140.0;
pub const CEILING_DISTANCE: f32 = 2044.0;
pub const GOAL_X: f32 = 892.75;
pub const GOAL_Z: f32 = 640.0;

// car constants
pub const MAX_BOOST_SPEED: f32 = 1000.0; // max speed if boosting FIXME get exact known value from graph, rename to MAX_SPEED
pub const MAX_ANGULAR_SPEED: f32 = 5.5;
pub const MAX_GROUND_ANGULAR_SPEED: f32 = 4.4; // NOTE this is based on the turning sample collection, though we might be able to redo a few samples to move this up

// batmobile
pub const RESTING_Z: f32 = 18.65;
pub const RESTING_Z_VELOCITY: f32 = 8.0;

// XXX must confirm. this might include height of the ball in free play when it first starts
// floating above the ground, which would be no good. 91.25 has been seen in RLBounce
pub static BALL_RADIUS: f32 = 93.143;

lazy_static! {
    // batmobile
    pub static ref CAR_DIMENSIONS: Vector3<f32> = Vector3::new(128.82, 84.67, 29.39);
    pub static ref PIVOT_OFFSET: Vector3<f32> = Vector3::new(9.008, 0.0, 12.094);
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Shoot,
    //Shadow,
    //GoToMid, // XXX not a real action, just a test
}

// re-implementation of rlbot::ControllerState since it's missing some traits. and probably better
// not to use its structs directly.
#[derive(Debug, Default, Clone)]
pub struct FullController {
    pub throttle: f32,
    pub steer: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub jump: bool,
    pub boost: bool,
    pub handbrake: bool,
}

impl From<&rlbot::ControllerState> for FullController {
    fn from(ctrl: &rlbot::ControllerState) -> Self {
        FullController {
            throttle: ctrl.throttle,
            steer: ctrl.steer,
            pitch: ctrl.pitch,
            yaw: ctrl.yaw,
            roll: ctrl.roll,
            jump: ctrl.jump,
            boost: ctrl.boost,
            handbrake: ctrl.handbrake,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BotState {
    pub plan: Option<Plan>,
    /// queue of (frame, controller)
    pub controller_history: VecDeque<(i32, FullController)>,
    pub turn_errors: VecDeque<f32>,
    pub last_action: Option<Action>,
}

#[derive(Debug, Default, Clone)]
pub struct GameState {
    pub ball: BallState,
    pub player: PlayerState,
    pub frame: i32,
    pub input_frame: i32,
}

// FIXME check if this order matches up with team integers we get from rlbot interface
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum Team {
    Blue,
    Orange,
}

// TODO-perf remove Copy
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct PlayerState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>, // FIXME switch to Rotation3!
    pub team: Team,
    //pub rotation: Rotation3<f32>,
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0),
            team: Team::Blue,
        }
    }
}

impl PlayerState {
    //pub fn heading(&self) -> Vector3<f32> {
    //}

    // global_velocity = rotation * local_velocity
    pub fn local_velocity(&self) -> Vector3<f32> {
        // the actual car with no rotation is sideways, pointed towards negative x. so we do an
        // additional rotation to convert to local coords with car pointing towards positive
        // y instead of negative x, since that's a lot more intuitive
        Rotation3::from_euler_angles(0.0, 0.0, -PI / 2.0)
            * na::inverse(&self.rotation.to_rotation_matrix())
            * self.velocity
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
            position: Vector3::new(0.0, 0.0, BALL_RADIUS), // on ground, center of field
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            //rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, -PI/2.0),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
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

pub type Plan = Vec<(PlayerState, BrickControllerState, f32)>;

#[derive(Serialize, Deserialize)]
pub struct SerializablePlan(pub Plan);

#[derive(Clone)]
pub struct PlanResult {
    pub plan: Option<Plan>,
    pub desired: DesiredContact,
    pub visualization_lines: Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>,
    pub visualization_points: Vec<(Point3<f32>, Point3<f32>)>,
}

impl Default for PlanResult {
    fn default() -> PlanResult {
        PlanResult {
            plan: None,
            desired: DesiredContact::default(),
            visualization_lines: vec![],
            visualization_points: vec![],
        }
    }
}

pub struct SearchConfig {
    pub step_duration: f32,
    pub slop: f32,
    pub max_cost: f32,
    pub max_iterations: i32,
    pub scale_heuristic: f32,
}

impl Default for SearchConfig {
    fn default() -> SearchConfig {
        SearchConfig {
            // pretty loose defaults, but makes the search a lot faster. for a bit more consistency
            // and accuracy, can do 8.0 * TICK step duration with 10.0 slop, though that will
            // require a larger max_iterations value to plan some paths
            step_duration: 16.0 * TICK,
            slop: 20.0,
            max_cost: 10.0,
            max_iterations: 50_000,
            scale_heuristic: 1.0,
        }
    }
}

// XXX we may want to use different internal structs, since in some cases we may care about
// position but not velocity, and vice versa
pub struct DesiredState {
    pub player: Option<PlayerState>,
    pub ball: Option<BallState>,
}

#[derive(Debug, Clone)]
pub struct DesiredContact {
    pub position: Vector3<f32>,
    pub heading: Vector3<f32>,
}

impl Default for DesiredContact {
    fn default() -> DesiredContact {
        DesiredContact {
            position: Vector3::new(0.0, 0.0, RESTING_Z),
            heading: Vector3::new(0.0, 1.0, 0.0),
        }
    }
}

const MAX_FRAMES: f32 = 100_000.0;

/// updates our game state, which is a representation of the packet, but with our own data types etc
pub fn update_game_state(
    game_state: &mut GameState,
    tick: &rlbot::flat::RigidBodyTick,
    player_index: usize,
) {
    let ball = tick.ball().expect("Missing ball");
    let ball = ball.state().expect("Missing rigid body ball state");
    let players = tick.players().expect("Missing players");
    let player = players.get(player_index);
    let player = player.state().expect("Missing rigid body player state");

    let bl = ball.location().expect("Missing ball location");
    let bv = ball.velocity().expect("Missing ball velocity");
    let bav = ball
        .angularVelocity()
        .expect("Missing ball angular velocity");
    game_state.frame = ball.frame();
    game_state.ball.position = Vector3::new(-bl.x(), bl.y(), bl.z()); // x should be positive towards right, it only makes sense
    game_state.ball.velocity = Vector3::new(-bv.x(), bv.y(), bv.z()); // x should be positive towards right, it only makes sense
    game_state.ball.angular_velocity = Vector3::new(-bav.x(), bav.y(), bav.z()); // x should be positive towards right, it only makes sense

    let pl = player.location().expect("Missing player location");
    let pv = player.velocity().expect("Missing player velocity");
    let pav = player
        .angularVelocity()
        .expect("Missing player angular velocity");
    let pr = player.rotation().expect("Missing player rotation");
    game_state.player.position = Vector3::new(-pl.x(), pl.y(), pl.z()); // x should be positive towards right, it only makes sense
    game_state.player.velocity = Vector3::new(-pv.x(), pv.y(), pv.z()); // x should be positive towards right, it only makes sense
    game_state.player.angular_velocity = Vector3::new(-pav.x(), pav.y(), pav.z()); // x should be positive towards right, it only makes sense

    // XXX not sure how to flip the x axis direction, but I know flipping the x value isn't right!
    game_state.player.rotation =
        UnitQuaternion::from_quaternion(Quaternion::new(pr.w(), pr.x(), pr.y(), pr.z()));

    let pitch = game_state.player.rotation.euler_angles().1;
    game_state.input_frame = (pitch * MAX_FRAMES).round() as i32;

    // FIXME we don't get team in the physics tick. maybe we need to seed this with a single
    //       GameTickPacket to start
    // game_state.player.team = match player.Team {
    //     0 => Team::Blue,
    //     1 => Team::Orange,
    //     _ => unimplemented!(),
    // };
}

pub fn set_frame_metadata(game_state: &mut GameState, controller: &mut rlbot::ControllerState) {
    controller.pitch = (game_state.frame as f32 % MAX_FRAMES) / MAX_FRAMES;
}
