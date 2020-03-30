extern crate kdtree;

use self::kdtree::KdTree;
use crate::{get_ball_position, get_normalization_rotation, HeuristicModel};
use na::{Rotation3, Vector3};
use ord_subset::OrdSubsetIterExt;
use state::{DesiredContact, PlayerState};
use std::error::Error;
use std::f32::consts::PI;
use std::fs::File;

pub(crate) const KNN_DIMENSIONS: usize = 5; // x, y, vx, vy, yaw

#[derive(Debug)]
pub struct KnnHeuristic {
    pub(crate) tree: KdTree<f32, f32, [f32; KNN_DIMENSIONS]>,
    pub(crate) ball_position: Vector3<f32>,
    pub(crate) normalization_rotation: Rotation3<f32>,
    pub(crate) scale: f32,
}

// so that yaw distance is in the same ballpark as positional distance
const SCALE_CIRCULAR_DISTANCE: f32 = 200.0;

// so that velocity differences are bigger than positional differences, which will bias us to
// finding states with similar velocity
const SCALE_VELOCITY_DISTANCE: f32 = 2.0;

// +PI and -PI are the same angle, so the distance needs to take that into account!
fn scaled_circular_distance(a: f32, b: f32) -> f32 {
    SCALE_CIRCULAR_DISTANCE * (a - b).abs().min(2.0 * PI + a - b).min(2.0 * PI + b - a)
}

fn squared_distance(a: f32, b: f32) -> f32 {
    (b - a).powf(2.0)
}

pub(crate) fn knn_distance(a: &[f32], b: &[f32]) -> f32 {
    squared_distance(a[0], b[0]) // x
        + squared_distance(a[1], b[1]) // y
        // FIXME we don't set these in generate-data // + SCALE_VELOCITY_DISTANCE * squared_distance(a[2], b[2]) // vx
        + SCALE_VELOCITY_DISTANCE * squared_distance(a[3], b[3]) // vy
        + scaled_circular_distance(a[4], b[4]).powf(2.0) // yaw
}

impl KnnHeuristic {
    pub fn try_new(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut tree = KdTree::new(KNN_DIMENSIONS);

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(File::open(path)?);

        for record in rdr.records() {
            let record = record?;
            let cost = record.get(0).expect("Invalid row?").parse()?;
            let x = record.get(1).expect("Invalid row?").parse()?;
            let y = record.get(2).expect("Invalid row?").parse()?;

            let lvx = record.get(4).expect("Invalid row?").parse()?;
            let lvy = record.get(5).expect("Invalid row?").parse()?;

            let yaw = record.get(12).expect("Invalid row?").parse()?;
            tree.add([x, y, lvx, lvy, yaw], cost)?;
        }

        Ok(KnnHeuristic {
            tree,
            // set the rest in configure step
            ..Default::default()
        })
    }

    pub(crate) fn to_knn_point(&self, player: &PlayerState) -> [f32; KNN_DIMENSIONS] {
        let pos = self.normalization_rotation * (player.position - self.ball_position);
        let lvel = player.local_velocity();
        let (_roll, _pitch, yaw) = (self.normalization_rotation * player.rotation).euler_angles();

        [pos.x, pos.y, lvel.x, lvel.y, yaw]
    }

    pub(crate) fn single_heuristic(&self, player: &PlayerState) -> f32 {
        let point = self.to_knn_point(&player);
        let nearest = self.tree.nearest(&point, 3, &knn_distance).unwrap();

        let max_distance: f32 = *nearest.iter().map(|(d, _)| d).ord_subset_max().unwrap();
        let total_weights: f32 = nearest.iter().map(|(d, _)| max_distance / d).sum();
        let weighted_average_cost: f32 = nearest
            .iter()
            .map(|(distance, &cost)| {
                let weight = max_distance / distance;
                weight * cost
            })
            .sum::<f32>()
            / total_weights;

        weighted_average_cost
    }
}

impl Default for KnnHeuristic {
    fn default() -> KnnHeuristic {
        KnnHeuristic {
            tree: KdTree::new(KNN_DIMENSIONS),
            ball_position: Vector3::new(0.0, 0.0, 0.0),
            normalization_rotation: Rotation3::from_euler_angles(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl HeuristicModel for KnnHeuristic {
    fn heuristic(
        &mut self,
        players: &[PlayerState],
        costs: &mut [f32],
    ) -> Result<(), Box<dyn Error>> {
        assert!(players.len() == costs.len());
        for (i, cost) in costs.iter_mut().enumerate() {
            let player = unsafe { players.get_unchecked(i) };
            *cost = self.single_heuristic(player) * self.scale;
        }

        Ok(())
    }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.normalization_rotation = get_normalization_rotation(desired);
        self.ball_position = get_ball_position(desired);
        self.scale = scale;
    }
}
