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
pub const LAG_FRAMES: usize = 0; // turns out there isn't actually much lag

// arena constants
pub const SIDE_WALL_DISTANCE: f32 = 4096.0;
pub const SIDE_CURVE_DISTANCE: f32 = 3838.0;
pub const BACK_WALL_DISTANCE: f32 = 5140.0;
pub const BACK_CURVE_DISTANCE: f32 = 4960.0;
pub const CEILING_DISTANCE: f32 = 2044.0;
pub const GOAL_X: f32 = 892.75;
pub const GOAL_Z: f32 = 640.0;

// car constants
pub const MAX_BOOST_SPEED: f32 = 2300.0; // TODO rename to MAX_SPEED
pub const MAX_ANGULAR_SPEED: f32 = 5.5;
pub const MAX_GROUND_ANGULAR_SPEED: f32 = 4.4; // NOTE this is based on the turning sample collection, though we might be able to redo a few samples to move this up

//pub const RESTING_Z: f32 = 18.65; // batmobile
pub const RESTING_Z: f32 = 17.01; // fennec
pub const RESTING_Z_VELOCITY: f32 = 8.0; // TODO double check

// source: https://github.com/samuelpmish/RLUtilities/blob/master/src/simulation/ball.cc#L17
// TODO handle hoops/dropshot radii
pub const BALL_INERTIAL_RADIUS: f32 = 91.25;
pub const BALL_COLLISION_RADIUS: f32 = 93.15;

pub const CAR_MASS: f32 = 180.0;

// using lazy static for now due to const restrictions:
// https://github.com/rustsim/nalgebra/issues/521
lazy_static! {
    // source: https://github.com/samuelpmish/RLUtilities/blob/master/src/simulation/car.cc#L369
    pub static ref CAR_INERTIA: na::Matrix3<f32> = CAR_MASS * na::Matrix3::new(
        751.0,    0.0,    0.0,
        0.0  , 1334.0,    0.0,
        0.0  ,    0.0, 1836.0,
    );
    pub static ref CAR_INVERSE_INERTIA: na::Matrix3<f32> = CAR_INERTIA.try_inverse().expect("Inverse car inertia failed");

    // batmobile
    //pub static ref CAR_DIMENSIONS: Vector3<f32> = Vector3::new(128.8198, 84.67036, 29.3944);
    //pub static ref CAR_OFFSET: Vector3<f32> = Vector3::new(-9.008572, 0.0, 12.0942);
    // TODO switch to octane after building new driving model
    // octane/fennec
    pub static ref CAR_DIMENSIONS: Vector3<f32> = Vector3::new(118.0074, 84.19941, 36.15907);
    pub static ref CAR_OFFSET: Vector3<f32> = Vector3::new(-13.87566, 0.0, 20.75499);
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

impl From<&rlbot::ControllerState> for BrickControllerState {
    /// NOTE this is approximate
    fn from(ctrl: &rlbot::ControllerState) -> Self {
        let throttle = if ctrl.throttle.abs() < 0.2 {
            Throttle::Idle
        } else if ctrl.throttle >= 0.0 {
            Throttle::Forward
        } else {
            Throttle::Reverse
        };

        let steer = if ctrl.steer.abs() < 0.25 {
            Steer::Straight
        } else if ctrl.steer >= 0.0 {
            Steer::Right
        } else {
            Steer::Left
        };

        BrickControllerState {
            throttle,
            steer,
            pitch: ctrl.pitch,
            yaw: ctrl.yaw,
            roll: ctrl.roll,
            jump: ctrl.jump,
            boost: ctrl.boost,
            handbrake: ctrl.handbrake,
        }
    }
}

impl From<BrickControllerState> for rlbot::ControllerState {
    fn from(controller: BrickControllerState) -> Self {
        rlbot::ControllerState {
            throttle: match controller.throttle {
                Throttle::Idle => 0.0,
                Throttle::Forward => 1.0,
                Throttle::Reverse => -1.0,
            },
            steer: match controller.steer {
                Steer::Straight => 0.0,
                Steer::Left => -1.0,
                Steer::Right => 1.0,
            },
            pitch: controller.pitch,
            yaw: controller.yaw,
            roll: controller.roll,
            jump: controller.jump,
            boost: controller.boost,
            handbrake: controller.handbrake,
            use_item: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BotState {
    pub plan: Option<Plan>,
    pub planned_ball: Option<BallState>,
    pub plan_source_frame: u32,
    pub cost_diff: f32,
    pub controller_history: VecDeque<BrickControllerState>,
    pub turn_errors: VecDeque<f32>,
    pub last_action: Option<Action>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameState {
    pub ball: BallState,
    pub player: PlayerState,
    pub frame: u32,
}

// FIXME check if this order matches up with team integers we get from rlbot interface
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum Team {
    Blue,
    Orange,
}

// TODO-perf remove Copy
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    pub fn hitbox_center(&self) -> Vector3<f32> {
        self.position + self.rotation.to_rotation_matrix() * (*CAR_OFFSET)
    }

    pub fn heading(&self) -> Vector3<f32> {
        // the actual car with no rotation is sideways, pointed towards negative x
        self.rotation.to_rotation_matrix() * Vector3::new(-1.0, 0.0, 0.0)
    }

    // global_velocity = rotation * local_velocity
    pub fn local_velocity(&self) -> Vector3<f32> {
        // the actual car with no rotation is sideways, pointed towards negative x. so we do an
        // additional rotation to convert to local coords with car pointing towards positive
        // y instead of negative x, since that's a lot more intuitive
        Rotation3::from_euler_angles(0.0, 0.0, -PI / 2.0)
            * self.rotation.to_rotation_matrix().inverse()
            * self.velocity
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct BallState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
}

impl Default for BallState {
    fn default() -> BallState {
        BallState {
            position: Vector3::new(0.0, 0.0, BALL_COLLISION_RADIUS), // on ground, center of field
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
            throttle: Throttle::Forward,
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
    pub planned_ball: Option<BallState>,
    pub source_frame: u32,
    pub cost_diff: f32,
    pub ball_trajectory: Vec<BallState>,
    pub visualization_lines: Vec<(Point3<f32>, Point3<f32>, Point3<f32>)>,
    pub visualization_points: Vec<(Point3<f32>, Point3<f32>)>,
}

impl Default for PlanResult {
    fn default() -> PlanResult {
        PlanResult {
            plan: None,
            planned_ball: None,
            source_frame: 0,
            cost_diff: std::f32::MAX,
            ball_trajectory: vec![],
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
    pub custom_filter: Option<fn(&PlayerState) -> bool>,
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
            max_iterations: 300_000, // 50_000 or lower is more appropriate when using knn heuristic
            scale_heuristic: 1.0,
            custom_filter: None,
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

/// updates our game state, which is a representation of the packet/ticket, but with our own data
/// types etc
pub fn update_game_state(
    game_state: &mut GameState,
    tick: &rlbot::GameTickPacket,
    player_index: usize,
    frame: u32,
) {
    let ball = tick.ball.as_ref().expect("Missing ball");
    let players = &tick.players;
    let player = players.get(player_index).expect("Missing player");

    let bp = &ball.physics;
    let bl = &bp.location;
    let bv = &bp.velocity;
    let bav = &bp.angular_velocity;
    game_state.ball.position = Vector3::new(-bl.x, bl.y, bl.z); // x should be positive towards right, it only makes sense
    game_state.ball.velocity = Vector3::new(-bv.x, bv.y, bv.z); // x should be positive towards right, it only makes sense
    game_state.ball.angular_velocity = Vector3::new(-bav.x, bav.y, bav.z); // x should be positive towards right, it only makes sense

    let pp = &player.physics;
    let pl = &pp.location;
    let pv = &pp.velocity;
    let pav = &pp.angular_velocity;
    let pr = &pp.rotation;
    game_state.player.position = Vector3::new(-pl.x, pl.y, pl.z); // x should be positive towards right, it only makes sense
    game_state.player.velocity = Vector3::new(-pv.x, pv.y, pv.z); // x should be positive towards right, it only makes sense
    game_state.player.angular_velocity = Vector3::new(-pav.x, pav.y, pav.z); // x should be positive towards right, it only makes sense

    let uq = UnitQuaternion::from_euler_angles(pr.roll, pr.pitch, pr.yaw);
    let q = uq.quaternion();
    // converting from right handed to left handed coordinate system (goes with the x axis flip above)
    // https://stackoverflow.com/a/34366144/127219
    game_state.player.rotation = UnitQuaternion::from_quaternion(
        Quaternion::new(q.scalar(), -q.vector()[0], q.vector()[1], -q.vector()[2])
    );

    game_state.frame = frame;

    game_state.player.team = match player.team {
        0 => Team::Blue,
        1 => Team::Orange,
        _ => unimplemented!(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_local_velocity() {
        let mut player = PlayerState::default();
        player.velocity.y = 1000.0;

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI / 2.0);
        assert!((player.local_velocity().y - 1000.0).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        assert!((player.local_velocity().y - (-1000.0)).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        assert!((player.local_velocity().x - 1000.0).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, PI);
        assert!((player.local_velocity().x - (-1000.0)).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, -PI);
        assert!((player.local_velocity().x - (-1000.0)).abs() < 0.1);
    }

    #[test]
    fn test_local_velocity_quadrants() {
        let mut player = PlayerState::default();
        player.velocity.y = 1000.0;

        player.rotation =  UnitQuaternion::from_euler_angles(0.0, 0.0, 3.0 * -PI / 4.0);
        assert!((player.local_velocity().y - 707.1).abs() < 0.1);
        assert!((player.local_velocity().x - -707.1).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 3.0 * PI / 4.0);
        assert!((player.local_velocity().y - -707.1).abs() < 0.1);
        assert!((player.local_velocity().x - -707.1).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 1.0 * -PI / 4.0);
        assert!((player.local_velocity().y - 707.1).abs() < 0.1);
        assert!((player.local_velocity().x - 707.1).abs() < 0.1);

        player.rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 1.0 * PI / 4.0);
        assert!((player.local_velocity().y - -707.1).abs() < 0.1);
        assert!((player.local_velocity().x - 707.1).abs() < 0.1);
    }
}
