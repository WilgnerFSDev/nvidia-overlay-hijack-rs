**<h1 align="center">🚀 NVIDIA Overlay Hijacker in Rust</h1>**

<p align="center">
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&amp;logo=rust&amp;logoColor=white" alt="Rust">
</p>

<p align="center">
    <a href="https://github.com/wilgnerfsdev/nvidia-overlay-hijack-rs/stargazers">
        <img src="https://img.shields.io/github/stars/wilgnerfsdev/nvidia-overlay-hijack-rs?colorA=363a4f&colorB=b7bdf8&style=for-the-badge">
    </a>
    <a href="https://github.com/wilgnerfsdev/nvidia-overlay-hijack-rs/issues">
        <img src="https://img.shields.io/github/issues/wilgnerfsdev/nvidia-overlay-hijack-rs?colorA=363a4f&colorB=f5a97f&style=for-the-badge"></a>
    <a href="https://github.com/wilgnerfsdev/nvidia-overlay-hijack-rs/contributors">
        <img src="https://img.shields.io/github/contributors/wilgnerfsdev/nvidia-overlay-hijack-rs?colorA=363a4f&colorB=a6da95&style=for-the-badge"></a>
</p>

This project is a robust and efficient implementation of NVIDIA Overlay Hijacker using Rust and WinAPI. It leverages Rust's powerful safety features to ensure a high degree of modularity, security, and speed. The codebase is well-structured, allowing for ease of modification and extensibility.

## 🎯 **Features**

- 🛡️ Built with Rust's standard safety features
- 🚀 Highly modular and efficient
- 🎨 Basic drawing functions: **`draw_text`** and **`draw_rect`**
- 📦 Helper functions: **`create_brush`**, **`create_text_layout`**, **`draw_element`**, **`create_brush_properties`**, **`color_u8_to_f32`**

## 🏗️ **Project Structure**
```
.
├── examples
│   └── basic.rs   // A simple usage example
└── src
    ├── core.rs   // Contains the Overlay struct and OverlayError enum
    ├── overlay_helper.rs   // Contains the OverlayHelper trait and its implementation
    └── lib.rs   // Contains the main functions, Drop implementation and drawing functions
```

## 🎨 **Extensibility**
New drawing functions can be easily implemented using the draw_element function provided by OverlayHelper. This function provides all the necessary resources, specific properties and differents drawing logic can be defined within the function scope or inside the draw_element function if necessary.<br>
Here is an example with the draw_rect function:
``` rust
pub fn draw_rect(&mut self, (x, y): (f32, f32), (width, height): (f32, f32), stroke_width: f32, color: (u8, u8, u8, u8)) {
        let draw_rect = D2D1_RECT_F {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        self.draw_element(color, |tar, brush| {
            unsafe { (*tar).DrawRectangle(&draw_rect, brush, stroke_width, std::ptr::null_mut()) };
        });
    }
}
```

## 💻 **Usage**
You can find a simple usage example below:
``` rust
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
```

## 🛠️ **Contribution**

This project is open for improvements and maintenance.<br>
Note that further implementations like as **`draw_text`** or **`draw_rect`** will not be added to the project by me. I left as an exercise to you =).<br>
The project provides ample resources and structures for you to create your own implementations.

## 🙏 **Acknowledgements**
Thanks to <a href="https://github.com/iraizo/nvidia-overlay-hijack">iraizo/nvidia-overlay-hijack</a> for being the inspiration for this Rust implementation.

## 📝 **License**
This project is licensed under the MIT License - see the <a href="https://github.com/WilgnerFSDev/nvidia-overlay-hijack-rs/blob/main/LICENSE.md">LICENSE.md</a> file for details.