extern crate winapi;

use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::{
    D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, ID2D1SolidColorBrush, D2D1_BRUSH_PROPERTIES,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_MATRIX_3X2_F, D2D1_POINT_2F, D2D1_PRESENT_OPTIONS_NONE, D2D1_RECT_F, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_SIZE_U,
};
use winapi::um::d2d1::{ID2D1Brush, D2D1_COLOR_F, D2D1_DRAW_TEXT_OPTIONS_NONE};
use winapi::um::dcommon::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT};
use winapi::um::dwmapi::DwmExtendFrameIntoClientArea;
use winapi::um::dwrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_REGULAR,
};
use winapi::um::uxtheme::MARGINS;
use winapi::um::winuser::{FindWindowA, GetClientRect, GetWindowLongA, GetWindowRect, SetWindowLongPtrA};
use winapi::Interface;
use wio::com::ComPtr;

#[derive(Debug)]
pub enum OverlayError {
    WindowNotFound,
    FailedToGetWindowLong,
    FailedToSetWindowLong,
    FailedToExtendFrame,
    FailedSetLayeredWindowAttributes,
    FailedToSetWindowPos,
    ShowWindowFailed,

    ID2D1FactoryFailed,
    StartupD2DFailed,
    IDWriteFactoryFailed,
    IDWriteTextFormatFailed,

    NoRenderTarget,
    DrawFailed,
    GetWindowRectFailed,
    GetWriteTextFormatFailed,
    DrawTextFailed(i32),
    CreateBrushFailed(i32),
    CreateSolidColorBrushFailed,
    ID2D1BrushCastFailed,
}

pub struct Overlay {
    window: HWND,
    d2d_factory: Option<ComPtr<ID2D1Factory>>,
    tar: Option<ComPtr<ID2D1HwndRenderTarget>>,
    write_factory: Option<ComPtr<IDWriteFactory>>,
    format: Option<ComPtr<IDWriteTextFormat>>,
    // ... other fields...
    font: String,
    font_size: f32,
}

impl Overlay {
    pub fn new(font: &str, size: f32) -> Self {
        Self {
            window: null_mut(),
            d2d_factory: None,
            tar: None,
            write_factory: None,
            format: None,

            font: font.to_string(),
            font_size: size,
        }
    }

    pub fn init(&mut self) -> Result<(), OverlayError> {
        self.window = unsafe {
            FindWindowA(
                format!("{}\0", "CEF-OSC-WIDGET").as_ptr() as _,
                format!("{}\0", "NVIDIA GeForce Overlay").as_ptr() as _,
            )
        };
        if self.window == null_mut() {
            return Err(OverlayError::WindowNotFound);
        }

        let info = unsafe { GetWindowLongA(self.window, -20) };
        if info == 0 {
            return Err(OverlayError::FailedToGetWindowLong);
        }

        let attrchange = unsafe { SetWindowLongPtrA(self.window, -20, (info | 0x20) as isize) };
        if attrchange == 0 {
            return Err(OverlayError::FailedToSetWindowLong);
        }

        let mut margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };

        let extendframeintoclientare = unsafe { DwmExtendFrameIntoClientArea(self.window, &mut margins) };
        if extendframeintoclientare != 0 {
            return Err(OverlayError::FailedToExtendFrame);
        }

        let set_layered_window_attributes: bool =
            unsafe { winapi::um::winuser::SetLayeredWindowAttributes(self.window, 0x000000, 0xFF, 0x02) != 0 };
        if !set_layered_window_attributes {
            return Err(OverlayError::FailedSetLayeredWindowAttributes);
        }

        let set_windows_pos: bool = unsafe {
            winapi::um::winuser::SetWindowPos(
                self.window,
                winapi::um::winuser::HWND_TOPMOST,
                0,
                0,
                0,
                0,
                winapi::um::winuser::SWP_NOMOVE | winapi::um::winuser::SWP_NOSIZE,
            ) != 0
        };
        if !set_windows_pos {
            return Err(OverlayError::FailedToSetWindowPos);
        }

        let show_window: bool =
            unsafe { winapi::um::winuser::ShowWindow(self.window, winapi::um::winuser::SW_SHOW) != 0 };
        if !show_window {
            return Err(OverlayError::ShowWindowFailed);
        }

        Ok(())
    }

    pub fn startup_d2d(&mut self) -> Result<(), OverlayError> {
        let mut rc: RECT = unsafe { std::mem::zeroed() };

        let d2d_factory: ComPtr<ID2D1Factory> = unsafe {
            let mut d2d_factory: *mut ID2D1Factory = std::ptr::null_mut();
            let hresult = D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &ID2D1Factory::uuidof(),
                std::ptr::null(),
                &mut d2d_factory as *mut _ as _,
            );

            if SUCCEEDED(hresult) {
                ComPtr::from_raw(d2d_factory)
            } else {
                return Err(OverlayError::ID2D1FactoryFailed);
            }
        };

        let write_factory: ComPtr<IDWriteFactory> = unsafe {
            let mut write_factory: *mut IDWriteFactory = std::ptr::null_mut();
            let hresult = DWriteCreateFactory(
                DWRITE_FACTORY_TYPE_SHARED,
                &IDWriteFactory::uuidof(),
                &mut write_factory as *mut _ as _,
            );

            if SUCCEEDED(hresult) {
                ComPtr::from_raw(write_factory)
            } else {
                return Err(OverlayError::IDWriteFactoryFailed);
            }
        };

        let font_wide: Vec<u16> = OsStr::new(&self.font)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();
        let locale_name: Vec<u16> = OsStr::new("en-us\0").encode_wide().chain(Some(0).into_iter()).collect();
        let format: ComPtr<IDWriteTextFormat> = unsafe {
            let mut format: *mut IDWriteTextFormat = std::ptr::null_mut();
            let hresult = (*write_factory).CreateTextFormat(
                font_wide.as_ptr(),
                std::ptr::null_mut(),
                DWRITE_FONT_WEIGHT_REGULAR,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                self.font_size,
                locale_name.as_ptr(),
                &mut format,
            );

            if SUCCEEDED(hresult) {
                ComPtr::from_raw(format)
            } else {
                return Err(OverlayError::IDWriteTextFormatFailed);
            }
        };

        unsafe {
            GetClientRect(self.window, &mut rc);
        }

        let tar: ComPtr<ID2D1HwndRenderTarget> = unsafe {
            let mut tar: *mut ID2D1HwndRenderTarget = std::ptr::null_mut();
            let render_target_properties = D2D1_RENDER_TARGET_PROPERTIES {
                _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let hwnd_target_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: self.window,
                pixelSize: D2D1_SIZE_U {
                    width: (rc.right - rc.left) as u32,
                    height: (rc.bottom - rc.top) as u32,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let hresult =
                (*d2d_factory).CreateHwndRenderTarget(&render_target_properties, &hwnd_target_properties, &mut tar);

            if SUCCEEDED(hresult) {
                ComPtr::from_raw(tar)
            } else {
                return Err(OverlayError::StartupD2DFailed);
            }
        };

        self.d2d_factory = Some(d2d_factory);
        self.write_factory = Some(write_factory);
        self.tar = Some(tar);
        self.format = Some(format);

        Ok(())
    }

    pub fn begin_scene(&mut self) {
        if let Some(tar) = &self.tar {
            unsafe {
                (*tar).BeginDraw();
            }
        }
    }

    pub fn end_scene(&mut self) {
        let tar = self.tar.as_ref().expect("No render target available");
        unsafe {
            (*tar).EndDraw(std::ptr::null_mut(), std::ptr::null_mut());
        }
    }

    pub fn clear_scene(&mut self) {
        let tar = self.tar.as_ref().expect("No render target available");
        unsafe {
            (*tar).Clear(std::ptr::null());
        }
    }

    pub fn draw_element<F: Fn(&ComPtr<ID2D1HwndRenderTarget>, *mut ID2D1Brush)>(
        &mut self, color: (u8, u8, u8, u8), draw: F,
    ) {
        let brush_color_ptr = self.create_brush(color);
        let tar = self.tar.as_ref().expect("No render target available");
        let brush: *mut ID2D1Brush = brush_color_ptr as *mut ID2D1Brush;
        draw(tar, brush);
        unsafe {
            (*brush_color_ptr).Release();
        }
    }
    
    pub fn draw_text(&mut self, (x, y): (f32, f32), text: String, color: (u8, u8, u8, u8)) {
        let text_layout = self.create_text_layout(&text);
        
        self.draw_element(color, |tar, brush| unsafe {
            (*tar).DrawTextLayout(
                D2D1_POINT_2F { x, y },
                text_layout.as_raw(),
                brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        });
    }

    pub fn draw_rect(&mut self, (x, y): (f32, f32), (width, height): (f32, f32), color: (u8, u8, u8, u8)) {
        self.draw_element(color, |tar, brush| {
            let draw_rect = D2D1_RECT_F {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            };
            unsafe { (*tar).DrawRectangle(&draw_rect, brush, 1.0f32, std::ptr::null_mut()) };
        });
    }

    fn create_brush(&mut self, (r, g, b, a): (u8, u8, u8, u8)) -> *mut ID2D1SolidColorBrush {
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
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.begin_scene();
        self.clear_scene();
        self.end_scene();
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

fn create_brush_properties() -> D2D1_BRUSH_PROPERTIES {
    D2D1_BRUSH_PROPERTIES {
        opacity: 1.0f32,
        transform: D2D1_MATRIX_3X2_F {
            matrix: [[1.0f32, 0.0f32], [0.0f32, 1.0f32], [0.0f32, 0.0f32]],
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn it_works() {
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
            overlay.draw_rect((10.0, 80.0), (100.0, 100.0), (255, 51, 0, 255));
            overlay.end_scene();
        }

        println!("Done!");
    }
}
