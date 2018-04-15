extern crate nalgebra as na;
extern crate state;

use na::{Vector3, Translation3, UnitQuaternion};
use state::*;
use std::f32;

#[no_mangle]
pub extern fn predict_test() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
