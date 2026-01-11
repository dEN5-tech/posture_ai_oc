/// Windows API blur overlay functionality for posture detection

use anyhow::Result;
use windows::core::s;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{MAX_ALPHA, FADE_SPEED};

pub struct BlurOverlay {
    hwnd: HWND,
    current_alpha: u32,
    target_alpha: u32,
}

impl BlurOverlay {
    pub fn new() -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            let class_name = s!("PostureBlurClass");

            let wc = WNDCLASSA {
                hInstance: instance.into(),
                lpszClassName: class_name,
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };
            RegisterClassA(&wc);

            // Create window: Topmost, Transparent (Click-through), ToolWindow (No Taskbar)
            let hwnd = CreateWindowExA(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TRANSPARENT,
                class_name,
                s!(""),
                WS_POPUP,
                0, 0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                None,
                None,
                instance,
                None,
            );

            Ok(Self {
                hwnd,
                current_alpha: 0,
                target_alpha: 0
            })
        }
    }

    pub fn set_target_visible(&mut self, visible: bool) {
        self.target_alpha = if visible { MAX_ALPHA } else { 0 };
    }

    // Runs every frame to smooth out the alpha transition
    pub fn update(&mut self) {
        if self.current_alpha == self.target_alpha {
            // Optimization: Hide window if fully transparent
            if self.current_alpha == 0 {
                unsafe { ShowWindow(self.hwnd, SW_HIDE) };
            }
            return;
        }

        // Show window if we are starting to fade in
        if self.current_alpha == 0 && self.target_alpha > 0 {
            unsafe { ShowWindow(self.hwnd, SW_SHOW) };
        }

        // Interpolate Alpha
        if self.current_alpha < self.target_alpha {
            self.current_alpha = (self.current_alpha + FADE_SPEED).min(self.target_alpha);
        } else {
            self.current_alpha = self.current_alpha.saturating_sub(FADE_SPEED).max(self.target_alpha);
        }

        // Apply Neutral Acrylic Blur (more effective visual punishment)
        // Color Format: ABGR -> 0xAA000000 (AA=Alpha, BB=Blue=00, GG=Green=00, RR=Red=00)
        // Using neutral color instead of red for better readability preservation
        let color = (self.current_alpha << 24) | 0x00000000;
        self.set_acrylic(color);
    }

    fn set_acrylic(&self, color: u32) {
        unsafe {
            let user32 = LoadLibraryA(s!("user32.dll")).unwrap();
            type SetWindowCompositionAttribute = unsafe extern "system" fn(HWND, *mut WindowCompositionAttributeData) -> i32;
            let func_ptr = GetProcAddress(user32, s!("SetWindowCompositionAttribute"));

            if let Some(func) = func_ptr {
                let func: SetWindowCompositionAttribute = std::mem::transmute(func);

                let mut policy = AccentPolicy {
                    AccentState: 4, // ACCENT_ENABLE_ACRYLICBLURBEHIND
                    AccentFlags: 0,
                    GradientColor: color,
                    AnimationId: 0,
                };

                let mut data = WindowCompositionAttributeData {
                    Attribute: 19, // WCA_ACCENT_POLICY
                    Data: &mut policy as *mut _ as *mut std::ffi::c_void,
                    SizeOfData: std::mem::size_of::<AccentPolicy>() as u32,
                };

                func(self.hwnd, &mut data);
            }
        }
    }

    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) }
    }
}

// Windows Structures
#[repr(C)]
#[allow(non_snake_case)]
struct AccentPolicy {
    AccentState: i32,
    AccentFlags: i32,
    GradientColor: u32,
    AnimationId: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct WindowCompositionAttributeData {
    Attribute: i32,
    Data: *mut std::ffi::c_void,
    SizeOfData: u32,
}
