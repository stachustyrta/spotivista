//! Win32 Shell integration module for SpotiVista on Windows Vista SP2 and Windows 7 SP1.

pub mod aero;
pub mod compat;
pub mod media_keys;
pub mod taskbar;
pub mod tray;

use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};

pub use aero::{DwmProvider, LiveDwmProvider, Margins};
pub use media_keys::MediaAction;
pub use taskbar::TaskbarButtonCommand;
pub use tray::TrayAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellEvent {
    Media(MediaAction),
    Tray(TrayAction),
    Taskbar(TaskbarButtonCommand),
    DwmCompositionChanged(bool),
    RestoreWindow,
}

pub struct ShellManager {
    pub aero: Arc<LiveDwmProvider>,
    pub tray: Arc<tray::TrayController>,
    pub taskbar: Arc<taskbar::TaskbarController>,
    sender: Sender<ShellEvent>,
    receiver: Receiver<ShellEvent>,
    hwnd: std::sync::atomic::AtomicIsize,
    current_track: std::sync::Mutex<Option<String>>,
    is_playing: std::sync::atomic::AtomicBool,
}

impl ShellManager {
    pub fn new() -> Arc<Self> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Arc::new(Self {
            aero: Arc::new(LiveDwmProvider::new()),
            tray: Arc::new(tray::TrayController::new()),
            taskbar: Arc::new(taskbar::TaskbarController::new()),
            sender,
            receiver,
            hwnd: std::sync::atomic::AtomicIsize::new(0),
            current_track: std::sync::Mutex::new(None),
            is_playing: std::sync::atomic::AtomicBool::new(false),
        })
    }

    pub fn receiver(&self) -> &Receiver<ShellEvent> {
        &self.receiver
    }

    pub fn sender(&self) -> Sender<ShellEvent> {
        self.sender.clone()
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Updates the playback state used for the tray context menu and tooltips.
    pub fn set_playback_state(&self, track_info: Option<&str>, playing: bool) {
        *self.current_track.lock().unwrap() = track_info.map(|s| s.to_string());
        self.is_playing.store(playing, std::sync::atomic::Ordering::Relaxed);
    }

    /// Extends DWM Aero Glass into the client area of the managed window.
    pub fn extend_glass(&self, margins: Margins) -> bool {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            self.aero.extend_frame_into_client_area(hwnd, margins)
        } else {
            false
        }
    }

    /// Attaches and subclasses the given window handle.
    pub fn attach(self: &Arc<Self>, hwnd: isize) {
        if hwnd == 0 {
            return;
        }
        self.hwnd.store(hwnd, std::sync::atomic::Ordering::Relaxed);

        // Apply initial DWM glass frame
        self.extend_glass(Margins::FULL_WINDOW);

        // Install tray icon
        self.tray.install(hwnd, "SpotiVista");

        #[cfg(windows)]
        {
            subclass_window(hwnd, Arc::clone(self));
        }
    }

    /// Detaches and cleans up the shell resources.
    pub fn detach(&self) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            self.tray.remove(hwnd);
            #[cfg(windows)]
            {
                use windows_sys::Win32::Foundation::HWND;
                use windows_sys::Win32::UI::WindowsAndMessaging::{GWLP_WNDPROC, SetWindowLongPtrW};
                let prev = PREV_WNDPROC.swap(0, std::sync::atomic::Ordering::Relaxed);
                if prev != 0 {
                    unsafe {
                        SetWindowLongPtrW(hwnd as HWND, GWLP_WNDPROC, prev);
                    }
                }
                let old = MANAGER_PTR.swap(std::ptr::null_mut(), std::sync::atomic::Ordering::SeqCst);
                if !old.is_null() {
                    unsafe { drop(Arc::from_raw(old)); }
                }
            }
            self.hwnd.store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl Drop for ShellManager {
    fn drop(&mut self) {
        self.detach();
    }
}

#[cfg(windows)]
static PREV_WNDPROC: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
#[cfg(windows)]
static MANAGER_PTR: std::sync::atomic::AtomicPtr<ShellManager> = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
#[cfg(windows)]
static TASKBAR_CREATED_MSG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(windows)]
fn subclass_window(hwnd: isize, manager: Arc<ShellManager>) {
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, GWLP_WNDPROC, RegisterWindowMessageW, SetWindowLongPtrW, WNDPROC,
        WM_APPCOMMAND, WM_COMMAND, WM_DWMCOMPOSITIONCHANGED, WM_NCDESTROY,
    };
    use crate::shell::tray::WM_TRAYICON;

    // Register Explorer restart notification message
    let created_wide: Vec<u16> = "TaskbarCreated\0".encode_utf16().collect();
    let taskbar_created = unsafe { RegisterWindowMessageW(created_wide.as_ptr()) };
    TASKBAR_CREATED_MSG.store(taskbar_created, std::sync::atomic::Ordering::Relaxed);

    let raw = Arc::into_raw(Arc::clone(&manager)) as *mut ShellManager;
    let old = MANAGER_PTR.swap(raw, std::sync::atomic::Ordering::SeqCst);
    if !old.is_null() {
        unsafe { drop(Arc::from_raw(old)); }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let prev = PREV_WNDPROC.load(std::sync::atomic::Ordering::Relaxed);
        let orig_proc: WNDPROC = if prev != 0 {
            Some(unsafe { std::mem::transmute(prev) })
        } else {
            None
        };

        let ptr = MANAGER_PTR.load(std::sync::atomic::Ordering::Relaxed);
        if !ptr.is_null() {
            let mgr = unsafe { &*ptr };
            // Explorer restart recovery
            let taskbar_created = TASKBAR_CREATED_MSG.load(std::sync::atomic::Ordering::Relaxed);
            if msg == taskbar_created && taskbar_created != 0 {
                let track = mgr.current_track.lock().unwrap().clone();
                let tip = track.as_deref().unwrap_or("SpotiVista");
                mgr.tray.install(hwnd as isize, tip);
            }

            // Taskbar button creation (Win7)
            if msg == mgr.taskbar.taskbar_button_created_message() && msg != 0 {
                mgr.taskbar.on_taskbar_button_created(hwnd as isize);
            }

            match msg {
                WM_APPCOMMAND => {
                    if let Some(action) = media_keys::parse_appcommand(lparam as isize) {
                        let _ = mgr.sender.send(ShellEvent::Media(action));
                        return 1;
                    }
                }
                WM_COMMAND => {
                    if let Some(cmd) = mgr.taskbar.parse_command(wparam) {
                        let _ = mgr.sender.send(ShellEvent::Taskbar(cmd));
                        return 0;
                    }
                }
                WM_TRAYICON => {
                    let event = (lparam & 0xFFFF) as u32;
                    const WM_LBUTTONUP: u32 = 0x0202;
                    const WM_LBUTTONDBLCLK: u32 = 0x0203;
                    const WM_RBUTTONUP: u32 = 0x0205;
                    const WM_CONTEXTMENU: u32 = 0x007B;

                    if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                        let _ = mgr.sender.send(ShellEvent::RestoreWindow);
                        return 0;
                    } else if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                        let track = mgr.current_track.lock().unwrap().clone();
                        let playing = mgr.is_playing.load(std::sync::atomic::Ordering::Relaxed);
                        if let Some(action) = mgr.tray.show_context_menu(hwnd as isize, track.as_deref(), playing) {
                            let _ = mgr.sender.send(ShellEvent::Tray(action));
                        }
                        return 0;
                    }
                }
                WM_DWMCOMPOSITIONCHANGED => {
                    let enabled = mgr.aero.on_composition_changed();
                    if enabled {
                        mgr.extend_glass(Margins::FULL_WINDOW);
                    }
                    let _ = mgr.sender.send(ShellEvent::DwmCompositionChanged(enabled));
                }
                WM_NCDESTROY => {
                    // Restore original window proc on destruction
                    if prev != 0 {
                        unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, prev); }
                        PREV_WNDPROC.store(0, std::sync::atomic::Ordering::Relaxed);
                    }
                    let old = MANAGER_PTR.swap(std::ptr::null_mut(), std::sync::atomic::Ordering::SeqCst);
                    if !old.is_null() {
                        unsafe { drop(Arc::from_raw(old)); }
                    }
                }
                _ => {}
            }
        }

        if let Some(proc) = orig_proc {
            unsafe { CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam) }
        } else {
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    let prev = unsafe {
        SetWindowLongPtrW(
            hwnd as HWND,
            GWLP_WNDPROC,
            window_proc as *const () as usize as isize,
        )
    };
    PREV_WNDPROC.store(prev, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_manager_creates_and_routes_events() {
        let mgr = ShellManager::new();
        assert_eq!(mgr.hwnd(), 0);

        mgr.set_playback_state(Some("Artist - Title"), true);
        assert_eq!(*mgr.current_track.lock().unwrap(), Some("Artist - Title".to_string()));
        assert!(mgr.is_playing.load(std::sync::atomic::Ordering::Relaxed));

        let sender = mgr.sender();
        sender.send(ShellEvent::Media(MediaAction::PlayPause)).unwrap();
        sender.send(ShellEvent::RestoreWindow).unwrap();

        assert_eq!(mgr.receiver().recv().unwrap(), ShellEvent::Media(MediaAction::PlayPause));
        assert_eq!(mgr.receiver().recv().unwrap(), ShellEvent::RestoreWindow);
    }
}
