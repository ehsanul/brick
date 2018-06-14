//use std::f32;
extern crate libloading as lib;

fn main() {
    if let Ok(lib) = lib::Library::new("predict/target/debug/libpredict.dylib") {
        let test_it: lib::Symbol<extern fn() -> Vec<f32>> = unsafe {
            lib.get(b"test_it\0").unwrap()
        };

        loop {
            let v = test_it();
            println!("test_it: {:?}", v);
        }
    } else {
        println!("couldn't find libpredict.dylib. Are you on osx and did you compile the predict sub-crate?")
    }
}
