use std::time::{Instant, Duration};

use nvidia_overlay_hijack::Overlay;

fn main() {
    let mut overlay = Overlay::new("Consolas", 18.0);

    // call init
    let init = overlay.init();
    if init.is_err() {
        println!("init failed");
    } else {
        println!("init success");
    }

    // call startup_d2d
    let startup_d2d = overlay.startup_d2d();
    if startup_d2d.is_err() {
        println!("startup_d2d failed");
    } else {
        println!("startup_d2d success");
    }

    println!("Successfully initialized, rendering for 10 seconds now..\n");

    // Show the overlay for 10 seconds
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let _ = overlay.begin_scene();
        let _ = overlay.clear_scene();
        let _ = overlay.draw_text((10.0, 10.0), "Hello World!", (255, 255, 255, 255));
        let _ = overlay.end_scene();
    }

    println!("Done!");
}
