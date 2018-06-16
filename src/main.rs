// XXX uncomment below to use system allocator
//
// #![cfg_attr(rustc_nightly, feature(test))]
// #![feature(alloc_system)]
//
// #![feature(alloc_system)]
// #![feature(global_allocator, allocator_api)]
//
// extern crate alloc_system;
//
// use alloc_system::System;
//
// #[global_allocator]
// static A: System = System;

use std::mem;
extern crate libloading as lib;

fn main() {
    if let Ok(lib) = lib::Library::new("predict/target/debug/libpredict.dylib") {
        let test_it: lib::Symbol<extern fn() -> Vec<f32>> = unsafe {
            lib.get(b"test_it\0").unwrap()
        };

        loop {
            let v = test_it();
            println!("test_it: {:?}", v);
            //mem::forget(v); // <-- leaking `v` avoids the segfault
        }
    } else {
        println!("couldn't find libpredict.dylib. Are you on osx and did you compile the predict sub-crate?")
    }
}
