extern crate dynamic_reload;

#[macro_use]
extern crate lazy_static;

use std::sync::{Arc, Mutex};
use std::f32;

use dynamic_reload::{DynamicReload, Lib, Symbol, Search, PlatformName, UpdateState};


lazy_static! {
    static ref RELOAD_HANDLER: Mutex<DynamicReload<'static>> = {
        Mutex::new(
            DynamicReload::new(Some(vec!["predict/target/debug"]),
                               Some("target/debug"),
                               Search::Default)
        )
    };

    static ref PREDICT: Mutex<PredictPlugin> = {
        let lib = match RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER").add_library("predict", PlatformName::Yes) {
            Ok(lib) => lib,
            Err(e) => {
                panic!("Unable to load dynamic lib, err {:?}", e);
            }
        };
        Mutex::new(PredictPlugin { lib: Some(lib) })
    };
}

struct PredictPlugin {
    lib: Option<Arc<Lib>>
}

impl PredictPlugin {
    fn unload_plugin(&mut self, lib: &Arc<Lib>) {
        self.lib = None;
    }

    fn reload_plugin(&mut self, lib: &Arc<Lib>) {
        self.lib = Some(lib.clone());
    }

    fn reload_callback(&mut self, state: UpdateState, lib: Option<&Arc<Lib>>) {
        match state {
            UpdateState::Before => Self::unload_plugin(self, lib.unwrap()),
            UpdateState::After => Self::reload_plugin(self, lib.unwrap()),
            UpdateState::ReloadFailed(_) => println!("Failed to reload"),
        }
    }
}

fn main() {
    loop {

        // FIXME is there a way to unlock without a made up scope?
        {
            // XXX there must be a reason why this happens, but PREDICT must be locked before
            // RELOAD_HANDLER, otherwise we apparently end up in a deadlock
            let mut p = PREDICT.lock().expect("Failed to get lock on PREDICT");
            let mut rh = RELOAD_HANDLER.lock().expect("Failed to get lock on RELOAD_HANDLER");
            rh.update(PredictPlugin::reload_callback, &mut p);
        }

        if let Some(ref x) = PREDICT.lock().unwrap().lib {
            // TODO cache
            let predict_test: Symbol<extern "C" fn() -> f32> = unsafe {
                x.lib.get(b"predict_test\0").unwrap()
            };
            println!("predict test: {}", predict_test());
        }

    }
}
