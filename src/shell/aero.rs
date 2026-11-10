//! Desktop Window Manager (DWM) Aero Glass integration for Windows Vista SP2 and Windows 7 SP1.

use std::sync::atomic::{AtomicBool, Ordering};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Margins {
    pub cx_left_width: i32,
    pub cx_right_width: i32,
    pub cy_top_height: i32,
    pub cy_bottom_height: i32,
}

impl Margins {
    /// Margin extending glass over the entire window client area.
    pub const FULL_WINDOW: Self = Self {
        cx_left_width: -1,
        cx_right_width: -1,
        cy_top_height: -1,
        cy_bottom_height: -1,
    };

    /// Custom glass margins for header and footer.
    pub fn top_and_bottom(top: i32, bottom: i32) -> Self {
        Self {
            cx_left_width: 0,
            cx_right_width: 0,
            cy_top_height: top,
            cy_bottom_height: bottom,
        }
    }
}

pub trait DwmProvider: Send + Sync {
    /// Checks whether DWM desktop composition (Aero) is currently enabled.
    fn is_composition_enabled(&self) -> bool;

    /// Extends the window frame glass into the client area.
    fn extend_frame_into_client_area(&self, hwnd: isize, margins: Margins) -> bool;
}

#[cfg(windows)]
type DwmIsCompositionEnabledFn = unsafe extern "system" fn(*mut i32) -> i32;

#[cfg(windows)]
type DwmExtendFrameIntoClientAreaFn = unsafe extern "system" fn(isize, *const Margins) -> i32;

pub struct LiveDwmProvider {
    #[cfg(windows)]
    dwm_is_composition_enabled: Option<DwmIsCompositionEnabledFn>,
    #[cfg(windows)]
    dwm_extend_frame_into_client_area: Option<DwmExtendFrameIntoClientAreaFn>,
    last_known_enabled: AtomicBool,
}

impl LiveDwmProvider {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};

            let wide_name: Vec<u16> = "dwmapi.dll\0".encode_utf16().collect();
            let mut module = unsafe { GetModuleHandleW(wide_name.as_ptr()) };
            if module.is_null() {
                module = unsafe { LoadLibraryW(wide_name.as_ptr()) };
            }

            let dwm_is_composition_enabled = if !module.is_null() {
                let proc = unsafe { GetProcAddress(module, b"DwmIsCompositionEnabled\0".as_ptr()) };
                proc.map(|p| unsafe { std::mem::transmute::<_, DwmIsCompositionEnabledFn>(p) })
            } else {
                None
            };

            let dwm_extend_frame_into_client_area = if !module.is_null() {
                let proc = unsafe { GetProcAddress(module, b"DwmExtendFrameIntoClientArea\0".as_ptr()) };
                proc.map(|p| unsafe { std::mem::transmute::<_, DwmExtendFrameIntoClientAreaFn>(p) })
            } else {
                None
            };

            let provider = Self {
                dwm_is_composition_enabled,
                dwm_extend_frame_into_client_area,
                last_known_enabled: AtomicBool::new(false),
            };
            let enabled = provider.check_composition_enabled();
            provider.last_known_enabled.store(enabled, Ordering::Relaxed);
            provider
        }
        #[cfg(not(windows))]
        {
            Self {
                last_known_enabled: AtomicBool::new(false),
            }
        }
    }

    #[cfg(windows)]
    fn check_composition_enabled(&self) -> bool {
        if let Some(func) = self.dwm_is_composition_enabled {
            let mut enabled: i32 = 0;
            let hr = unsafe { func(&mut enabled) };
            hr >= 0 && enabled != 0
        } else {
            false
        }
    }

    #[cfg(not(windows))]
    fn check_composition_enabled(&self) -> bool {
        false
    }

    /// Refresh the cached composition state (e.g. after WM_DWMCOMPOSITIONCHANGED).
    pub fn on_composition_changed(&self) -> bool {
        let enabled = self.check_composition_enabled();
        self.last_known_enabled.store(enabled, Ordering::Relaxed);
        enabled
    }
}

impl Default for LiveDwmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DwmProvider for LiveDwmProvider {
    fn is_composition_enabled(&self) -> bool {
        #[cfg(windows)]
        {
            self.check_composition_enabled()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    fn extend_frame_into_client_area(&self, hwnd: isize, margins: Margins) -> bool {
        #[cfg(windows)]
        {
            if hwnd == 0 {
                return false;
            }
            if let Some(func) = self.dwm_extend_frame_into_client_area {
                let hr = unsafe { func(hwnd, &margins) };
                hr >= 0
            } else {
                false
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (hwnd, margins);
            false
        }
    }
}

#[cfg(test)]
pub struct MockDwmProvider {
    pub enabled: bool,
    pub last_margins: std::sync::Mutex<Option<Margins>>,
}

#[cfg(test)]
impl MockDwmProvider {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_margins: std::sync::Mutex::new(None),
        }
    }
}

#[cfg(test)]
impl DwmProvider for MockDwmProvider {
    fn is_composition_enabled(&self) -> bool {
        self.enabled
    }

    fn extend_frame_into_client_area(&self, _hwnd: isize, margins: Margins) -> bool {
        if self.enabled {
            *self.last_margins.lock().unwrap() = Some(margins);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_dwm_tracks_margins() {
        let mock = MockDwmProvider::new(true);
        assert!(mock.is_composition_enabled());
        let full = Margins::FULL_WINDOW;
        assert!(mock.extend_frame_into_client_area(1234, full));
        assert_eq!(*mock.last_margins.lock().unwrap(), Some(full));
    }

    #[test]
    fn mock_dwm_disabled_fails_gracefully() {
        let mock = MockDwmProvider::new(false);
        assert!(!mock.is_composition_enabled());
        assert!(!mock.extend_frame_into_client_area(1234, Margins::FULL_WINDOW));
        assert_eq!(*mock.last_margins.lock().unwrap(), None);
    }
}
