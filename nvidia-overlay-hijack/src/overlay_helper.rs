use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;

use winapi::shared::windef::RECT;
use winapi::um::d2d1::{ID2D1SolidColorBrush, ID2D1HwndRenderTarget, D2D1_BRUSH_PROPERTIES, D2D1_COLOR_F, D2D1_MATRIX_3X2_F, ID2D1Brush};
use winapi::shared::winerror::SUCCEEDED;
use winapi::Interface;
use winapi::um::dwrite::IDWriteTextLayout;
use winapi::um::winuser::GetWindowRect;
use wio::com::ComPtr;
use crate::Overlay;

pub trait OverlayHelper {
    fn create_brush(&self, color: (u8, u8, u8, u8)) -> *mut ID2D1SolidColorBrush;
    fn create_text_layout(&self, text: &str) -> ComPtr<IDWriteTextLayout>;
    
    fn draw_element<F: Fn(&ComPtr<ID2D1HwndRenderTarget>, *mut ID2D1Brush)>(
        &self, color: (u8, u8, u8, u8), draw: F
    );
}

impl OverlayHelper for Overlay {
    fn create_brush(&self, (r, g, b, a): (u8, u8, u8, u8)) -> *mut ID2D1SolidColorBrush {
        let tar = self.tar.as_ref().expect("No render target available");
        let brush_properties = create_brush_properties();

        let color = color_u8_to_f32((r, g, b, a));

        let mut _brush_color_ptr: *mut ID2D1SolidColorBrush = std::ptr::null_mut();
        _brush_color_ptr = unsafe {
            let mut brush_color: *mut ID2D1SolidColorBrush = std::ptr::null_mut();
            let hresult = (*tar).CreateSolidColorBrush(&color, &brush_properties, &mut brush_color);

            if SUCCEEDED(hresult) {
                brush_color
            } else {
                panic!("Failed to create solid color brush");
            }
        };
        _brush_color_ptr
    }

    fn create_text_layout(&self, text: &str) -> ComPtr<IDWriteTextLayout> {
        let format = self.format.as_ref().expect("No text format available");

        let mut rc: RECT = RECT {left: 0, top: 0, right: 0, bottom: 0,};
        unsafe {
            GetWindowRect(self.window, &mut rc);
        }
    
        let text_wide: Vec<u16> = OsStr::new(text).encode_wide().chain(Some(0).into_iter()).collect();
        unsafe {
            let mut text_layout: *mut IDWriteTextLayout = std::ptr::null_mut();
            let hresult = (*self.write_factory.as_ref().unwrap()).CreateTextLayout(
                text_wide.as_ptr(),
                text_wide.len() as u32,
                format.as_raw(),
                (rc.right - rc.left) as f32,
                (rc.bottom - rc.top) as f32,
                &mut text_layout,
            );
    
            if SUCCEEDED(hresult) {
                ComPtr::from_raw(text_layout)
            } else {
                panic!("Failed to create text layout");
            }
        }
    }

    fn draw_element<F: Fn(&ComPtr<ID2D1HwndRenderTarget>, *mut ID2D1Brush)>( &self, color: (u8, u8, u8, u8), draw: F, ) {
        let brush_color_ptr = self.create_brush(color);
        let tar = self.tar.as_ref().expect("No render target available");
        let brush: *mut ID2D1Brush = brush_color_ptr as *mut ID2D1Brush;
        draw(tar, brush);
        unsafe {
            (*brush_color_ptr).Release();
        }
    }
}

// Auxiliary functions
fn create_brush_properties() -> D2D1_BRUSH_PROPERTIES {
    D2D1_BRUSH_PROPERTIES {
        opacity: 1.0f32,
        transform: D2D1_MATRIX_3X2_F {
            matrix: [[1.0f32, 0.0f32], [0.0f32, 1.0f32], [0.0f32, 0.0f32]],
        },
    }
}

fn color_u8_to_f32((r, g, b, a): (u8, u8, u8, u8)) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: r as f32 / 255.0f32,
        g: g as f32 / 255.0f32,
        b: b as f32 / 255.0f32,
        a: a as f32 / 255.0f32,
    }
}