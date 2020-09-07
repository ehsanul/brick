extern crate fnv;
extern crate heuristic;
extern crate indexmap;
extern crate itertools;
extern crate nalgebra as na;
pub extern crate predict;
extern crate rlbot;
extern crate state;

#[macro_use]
extern crate lazy_static;

pub mod plan;
pub mod play;

pub use heuristic::*;

pub fn get_model() -> impl HeuristicModel {
    // TODO config file or something
    //let path = "./heuristic/train/nn/simple_throttle_cost_saved_model/1552341051/";
    //NeuralHeuristic::try_new(path).expect("Failed to initialize NeuralHeuristic")

    // TODO config file or something
    let path = "./time.csv";
    KnnHeuristic::try_new(path).expect("Failed to initialize KnnHeuristic")

    //BasicHeuristic::default()
}
