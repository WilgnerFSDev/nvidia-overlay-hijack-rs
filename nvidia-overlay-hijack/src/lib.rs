extern crate winapi;

use std::ptr::null_mut;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{FindWindowA, GetWindowLongA, SetWindowLongPtrA};
use winapi::um::d2d1::{ID2D1Factory, ID2D1HwndRenderTarget, D2D1CreateFactory, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_RENDER_TARGET_PROPERTIES, D2D1_HWND_RENDER_TARGET_PROPERTIES};
use winapi::um::dwrite::{IDWriteFactory, DWriteCreateFactory, DWRITE_FACTORY_TYPE_SHARED};
use winapi::um::dwmapi::{DwmExtendFrameIntoClientArea};
use winapi::um::uxtheme::MARGINS;
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
    
}

pub struct Overlay {
    window: HWND,
    d2d_factory: Option<ComPtr<ID2D1Factory>>,
    tar: Option<ComPtr<ID2D1HwndRenderTarget>>,
    write_factory: Option<ComPtr<IDWriteFactory>>,
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
            
            font: font.to_string(),
            font_size: size,
        }
    }

    pub fn init(&mut self) -> Result<(), OverlayError> {
        self.window = unsafe { FindWindowA(format!("{}\0", "CEF-OSC-WIDGET").as_ptr() as _, format!("{}\0", "NVIDIA GeForce Overlay").as_ptr() as _) };
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

        let set_layered_window_attributes: bool = unsafe { winapi::um::winuser::SetLayeredWindowAttributes(self.window, 0x000000, 0xFF, 0x02) != 0 };
        if !set_layered_window_attributes {
            return Err(OverlayError::FailedSetLayeredWindowAttributes);
        }

        let set_windows_pos: bool = unsafe { winapi::um::winuser::SetWindowPos(self.window, winapi::um::winuser::HWND_TOPMOST, 0, 0, 0, 0, winapi::um::winuser::SWP_NOMOVE | winapi::um::winuser::SWP_NOSIZE) != 0 };
        if !set_windows_pos {
            return Err(OverlayError::FailedToSetWindowPos);
        }

        let show_window: bool = unsafe { winapi::um::winuser::ShowWindow(self.window, winapi::um::winuser::SW_SHOW) != 0 };
        if !show_window {
            return Err(OverlayError::ShowWindowFailed);
        }

        Ok(())
    }

    

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
       
    }
}
