//! Windows 7 Taskbar Thumbnail Toolbar (`ITaskbarList3`) with graceful Windows Vista fallback.

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskbarButtonCommand {
    Previous,
    PlayPause,
    Next,
}

pub const ID_TB_PREV: u32 = 101;
pub const ID_TB_PLAY_PAUSE: u32 = 102;
pub const ID_TB_NEXT: u32 = 103;

// Button IDs matching spotifast::thumbbar
pub const THUMB_PREV: u32 = 2;
pub const THUMB_PLAY_PAUSE: u32 = 3;
pub const THUMB_NEXT: u32 = 4;

pub struct TaskbarController {
    taskbar_button_created_msg: u32,
    supported: AtomicBool,
    buttons_added: AtomicBool,
}

impl TaskbarController {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::RegisterWindowMessageW;

            let wide: Vec<u16> = "TaskbarButtonCreated\0".encode_utf16().collect();
            let msg = unsafe { RegisterWindowMessageW(wide.as_ptr()) };

            Self {
                taskbar_button_created_msg: msg,
                supported: AtomicBool::new(false),
                buttons_added: AtomicBool::new(false),
            }
        }
        #[cfg(not(windows))]
        {
            Self {
                taskbar_button_created_msg: 0,
                supported: AtomicBool::new(false),
                buttons_added: AtomicBool::new(false),
            }
        }
    }

    pub fn taskbar_button_created_message(&self) -> u32 {
        self.taskbar_button_created_msg
    }

    pub fn is_supported(&self) -> bool {
        self.supported.load(Ordering::Relaxed)
    }

    /// Called when `WM_TASKBARBUTTONCREATED` is received.
    /// Tests for `ITaskbarList3` support (available on Windows 7+, absent on Windows Vista).
    pub fn on_taskbar_button_created(&self, _hwnd: isize) -> bool {
        #[cfg(windows)]
        {
            use std::ffi::c_void;
            use windows_sys::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
            use windows_sys::Win32::UI::Shell::TaskbarList;
            use windows_sys::core::GUID;

            // IID_ITaskbarList3 = {ea1afb91-9e28-4b86-90e9-9e9f8a5eefaf}
            const IID_ITASKBARLIST3: GUID = GUID::from_u128(0xea1afb91_9e28_4b86_90e9_9e9f8a5eefaf);

            let mut ptr: *mut c_void = std::ptr::null_mut();
            let hr = unsafe {
                CoCreateInstance(
                    &TaskbarList,
                    std::ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    &IID_ITASKBARLIST3,
                    &mut ptr,
                )
            };

            if hr >= 0 && !ptr.is_null() {
                // Windows 7+ with ITaskbarList3 supported
                self.supported.store(true, Ordering::Relaxed);
                log::info!("Windows 7 ITaskbarList3 detected and initialized");
                // Release the probe instance
                unsafe {
                    let vtable = *(ptr as *mut *mut usize);
                    let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                        std::mem::transmute(*vtable.add(2));
                    release(ptr);
                }
                true
            } else {
                // Windows Vista returns E_NOINTERFACE (0x80004002)
                self.supported.store(false, Ordering::Relaxed);
                log::info!("ITaskbarList3 not available (Windows Vista SP2 mode active)");
                false
            }
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    pub const THBN_CLICKED: u32 = 0x1800;

    /// Handles WM_COMMAND button presses from thumbnail toolbar buttons.
    /// Strictly verifies HIWORD(wparam) == THBN_CLICKED (0x1800) to avoid
    /// colliding with standard Win32 dialog button IDs (e.g. IDCANCEL=2).
    pub fn parse_command(&self, wparam: usize) -> Option<TaskbarButtonCommand> {
        let notification = ((wparam >> 16) & 0xFFFF) as u32;
        if notification != Self::THBN_CLICKED {
            return None;
        }
        let id = (wparam & 0xFFFF) as u32;
        match id {
            ID_TB_PREV | THUMB_PREV => Some(TaskbarButtonCommand::Previous),
            ID_TB_PLAY_PAUSE | THUMB_PLAY_PAUSE => Some(TaskbarButtonCommand::PlayPause),
            ID_TB_NEXT | THUMB_NEXT => Some(TaskbarButtonCommand::Next),
            _ => None,
        }
    }
}

impl Default for TaskbarController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_taskbar_buttons() {
        let ctrl = TaskbarController::new();
        let make_wparam = |id: u32| ((TaskbarController::THBN_CLICKED as usize) << 16) | (id as usize);

        assert_eq!(ctrl.parse_command(make_wparam(ID_TB_PREV)), Some(TaskbarButtonCommand::Previous));
        assert_eq!(ctrl.parse_command(make_wparam(ID_TB_PLAY_PAUSE)), Some(TaskbarButtonCommand::PlayPause));
        assert_eq!(ctrl.parse_command(make_wparam(ID_TB_NEXT)), Some(TaskbarButtonCommand::Next));
        assert_eq!(ctrl.parse_command(make_wparam(THUMB_PREV)), Some(TaskbarButtonCommand::Previous));
        assert_eq!(ctrl.parse_command(make_wparam(999)), None);
        // Ensure standard dialog button notification (0) does not match
        assert_eq!(ctrl.parse_command(THUMB_PREV as usize), None);
    }
}
