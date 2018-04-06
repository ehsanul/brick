use na::{Vector3, UnitQuaternion};

pub struct GameState {
    pub ball: BallState,
    pub player: PlayerState,
}

pub struct PlayerState {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

pub struct BallState {
    pub position: Vector3<f32>,
}
