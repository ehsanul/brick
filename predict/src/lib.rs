extern crate nalgebra as na;
extern crate ncollide;
extern crate fnv;
extern crate obj;
extern crate csv;
extern crate state;

#[macro_use]
extern crate lazy_static;

pub mod arena;
pub mod player;
pub mod ball;
pub mod sample;

pub const FPS:f32 = 120.0;
pub const TICK: f32 = 1.0 / FPS; // matches RL's internal fixed physics tick rate
