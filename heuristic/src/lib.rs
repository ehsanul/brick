extern crate csv;
extern crate nalgebra as na;
extern crate ord_subset;
extern crate state;

use na::{Rotation3, Vector3};
use state::{DesiredContact, PlayerState, BALL_COLLISION_RADIUS};
use std::error::Error;

mod basic;
pub use basic::BasicHeuristic;

mod knn;
pub use knn::KnnHeuristic;

mod hybrid_knn;
pub use hybrid_knn::HybridKnnHeuristic;

#[cfg(feature = "neural")]
mod neural;
#[cfg(feature = "neural")]
pub use neural::NeuralHeuristic;

pub trait HeuristicModel {
    fn unscaled_heuristic(
        &mut self,
        players: &[PlayerState],
        costs: &mut [f32],
    ) -> Result<(), Box<dyn Error>>;

    fn heuristic(
        &mut self,
        players: &[PlayerState],
        costs: &mut [f32],
    ) -> Result<(), Box<dyn Error>> {
        self.unscaled_heuristic(&players, costs)?;

        for c in costs.iter_mut() {
            *c *= self.scale()
        }

        Ok(())
    }

    fn scale(&self) -> f32;

    // NOTE scale is a fudge factor to make the heuristic over-estimate, which gives up
    // accuracy/optimality in exchange for speed
    fn configure(&mut self, desired: &DesiredContact, scale: f32);
}

pub(crate) fn get_normalization_rotation(desired: &DesiredContact) -> Rotation3<f32> {
    // the training data is based on the ball positioned at 0, 0, and the desired heading being
    // directly in the positive y axis. given the current heading, we want to find
    // a transformation matrix that would tranform it into the standard heading, which we can
    // apply to the car in order to align with how we trained.
    let standard_heading = Vector3::new(0.0, 1.0, 0.0);
    let heading = desired.heading / desired.heading.norm();

    let mut angle = na::Matrix::dot(&standard_heading, &heading).acos();

    // if standard is to the right, we need to rotate clockwise
    // https://math.stackexchange.com/a/555243
    let delta = heading.x * standard_heading.y - heading.y * standard_heading.x;
    if delta < 0.0 {
        angle *= -1.0;
    }

    Rotation3::from_euler_angles(0.0, 0.0, angle)
}

pub(crate) fn get_ball_position(desired: &DesiredContact) -> Vector3<f32> {
    desired.position + BALL_COLLISION_RADIUS * desired.heading
}
