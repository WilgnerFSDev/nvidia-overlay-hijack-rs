use std::time::{Duration, Instant};

use nvidia_overlay_hijack::core::Overlay;

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
        overlay.begin_scene();
        overlay.clear_scene();
        overlay.draw_text(
            (10.0, 30.0),
            "github.com/WilgnerFSDev/nvidia-overlay-hijack-rs".to_string(),
            (255, 51, 0, 255),
        );
        overlay.draw_rect((10.0, 80.0), (100.0, 100.0), 2.0, (255, 51, 0, 255));
        overlay.end_scene();
    }

    println!("Done!");
}
