extern crate nalgebra as na;

use na::{Vector3, Translation3, UnitQuaternion};
// TODO figure this out // use state::*;

use std::f32;

#[no_mangle]
pub extern fn predict_test() -> f32 {
    3.0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
