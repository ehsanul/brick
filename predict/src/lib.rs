extern crate nalgebra as na;
extern crate ncollide;
extern crate obj;
extern crate csv;
extern crate state;

#[macro_use]
extern crate lazy_static;

pub mod arena;
pub mod player;
pub mod ball;
pub mod sample;

static TICK: f32 = 1.0 / 120.0; // matches RL's internal fixed physics tick rate
