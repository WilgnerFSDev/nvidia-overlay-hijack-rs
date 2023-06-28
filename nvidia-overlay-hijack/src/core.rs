use winapi::{shared::windef::HWND, um::{d2d1::{ID2D1Factory, ID2D1HwndRenderTarget}, dwrite::{IDWriteFactory, IDWriteTextFormat}}};
use wio::com::ComPtr;

pub struct Overlay {
    pub window: HWND,
    pub d2d_factory: Option<ComPtr<ID2D1Factory>>,
    pub tar: Option<ComPtr<ID2D1HwndRenderTarget>>,
    pub write_factory: Option<ComPtr<IDWriteFactory>>,
    pub format: Option<ComPtr<IDWriteTextFormat>>,
    // ... other fields...
    pub font: String,
    pub font_size: f32,
}

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