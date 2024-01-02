#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use ookami::prelude::*;

fn main() {
    log::init_max_debug();
    log::lib_info();

    AppBuilder::new()
        .with_title("Ookami Renderer")
        .with_size(800, 450)
        .run();
}

// TODO: Wrap up window class into separate library for reuse.
//       Keep it simple with the iterator method from Piston or trait method from Koyote.