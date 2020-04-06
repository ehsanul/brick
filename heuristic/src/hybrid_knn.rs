use knn;
use basic;

use state::{DesiredContact, PlayerState};
use std::error::Error;
use crate::HeuristicModel;

#[derive(Debug)]
pub struct HybridKnnHeuristic {
    knn_heuristic: knn::KnnHeuristic,
    basic_heuristic: basic::BasicHeuristic,
    scale: f32,
}

impl HybridKnnHeuristic {
    pub fn try_new(path: &str) -> Result<Self, Box<dyn Error>> {
        let knn_heuristic = knn::KnnHeuristic::try_new(path)?;
        let basic_heuristic = basic::BasicHeuristic::default();
        Ok(HybridKnnHeuristic {
            knn_heuristic,
            basic_heuristic,
            scale: 1.0,
        })
    }
}

impl HeuristicModel for HybridKnnHeuristic {
    fn unscaled_heuristic(
        &mut self,
        players: &[PlayerState],
        costs: &mut [f32],
    ) -> Result<(), Box<dyn Error>> {
        assert!(players.len() == costs.len());
        for (i, cost) in costs.iter_mut().enumerate() {
            let player = unsafe { players.get_unchecked(i) };
            let point = self.knn_heuristic.to_knn_point(&player);
            let single_nearest = self.knn_heuristic.tree.nearest(&point, 1, &knn::knn_distance).unwrap()[0];
            let distance = single_nearest.0;

            // FIXME
            //println!("distance: {}", distance);

            // if under some empirically determined threshold, we decide knn is accurate
            if distance < 1_500_000.0 {
                *cost = self.knn_heuristic.single_heuristic(player);
            } else {
                *cost = 1.05 * self.basic_heuristic.single_heuristic(player);
            }
        }

        Ok(())
    }

    fn scale(&self) -> f32 { self.scale }

    fn configure(&mut self, desired: &DesiredContact, scale: f32) {
        self.scale = scale;
        self.knn_heuristic.configure(desired, 1.0);
        self.basic_heuristic.configure(desired, 1.0);
    }
}
