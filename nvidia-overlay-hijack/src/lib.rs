extern crate winapi;

use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::dxgiformat::DXGI_FORMAT_UNKNOWN;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::S_OK;
use winapi::um::dcommon::{D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED};
use winapi::um::winuser::{FindWindowA, GetWindowLongA, SetWindowLongPtrA, GetClientRect};
use winapi::um::d2d1::{ID2D1Factory, ID2D1HwndRenderTarget, D2D1CreateFactory, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_RENDER_TARGET_PROPERTIES, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_SIZE_U, D2D1_PRESENT_OPTIONS_NONE};
use winapi::um::dwrite::{IDWriteFactory, DWriteCreateFactory, DWRITE_FACTORY_TYPE_SHARED, IDWriteTextFormat, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL};
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
    
    ID2D1FactoryFailed,
    StartupD2DFailed,
    IDWriteFactoryFailed,
    IDWriteTextFormatFailed,

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

            if hresult == S_OK {
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

            if hresult == S_OK {
                ComPtr::from_raw(write_factory)
            } else {
                return Err(OverlayError::IDWriteFactoryFailed);
            }
        };

        let font_wide: Vec<u16> = OsStr::new(&self.font).encode_wide().chain(Some(0).into_iter()).collect();
        let locale_name: Vec<u16> = OsStr::new("en-us\0").encode_wide().chain(Some(0).into_iter()).collect();
        let _format: ComPtr<IDWriteTextFormat> = unsafe {
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

            if hresult == S_OK {
                ComPtr::from_raw(format)
            } else {
                return Err(OverlayError::IDWriteTextFormatFailed);
            }
        };
        
        unsafe { GetClientRect(self.window, &mut rc); }

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

            let hresult = (*d2d_factory).CreateHwndRenderTarget(
                &render_target_properties,
                &hwnd_target_properties,
                &mut tar,
            );

            if hresult == S_OK {
                ComPtr::from_raw(tar)
            } else {
                return Err(OverlayError::StartupD2DFailed);
            }
        };

        self.d2d_factory = Some(d2d_factory);
        self.write_factory = Some(write_factory);
        self.tar = Some(tar);

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
