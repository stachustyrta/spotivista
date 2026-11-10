//! Windows Notification Area (System Tray) integration.

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    PlayPause,
    Next,
    Previous,
    RestoreWindow,
    Exit,
}

pub const WM_TRAYICON: u32 = 0x0400 + 101; // WM_USER + 101
pub const TRAY_ICON_UID: u32 = 1;

pub const CMD_NOW_PLAYING: usize = 1000;
pub const CMD_PLAY_PAUSE: usize = 1001;
pub const CMD_NEXT: usize = 1002;
pub const CMD_PREV: usize = 1003;
pub const CMD_RESTORE: usize = 1004;
pub const CMD_EXIT: usize = 1005;

pub struct TrayController {
    installed: AtomicBool,
}

impl TrayController {
    pub fn new() -> Self {
        Self {
            installed: AtomicBool::new(false),
        }
    }

    pub fn is_installed(&self) -> bool {
        self.installed.load(Ordering::Relaxed)
    }

    /// Adds the SpotiVista icon to the Windows Notification Area.
    pub fn install(&self, hwnd: isize, tip: &str) -> bool {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HWND;
            use windows_sys::Win32::UI::Shell::{
                NOTIFYICONDATAW, NIM_ADD, NIF_ICON, NIF_MESSAGE, NIF_TIP, Shell_NotifyIconW,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassLongPtrW, GCLP_HICON, LoadIconW, IDI_APPLICATION};

            if hwnd == 0 {
                return false;
            }

            let hinstance = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null()) };
            let mut hicon = unsafe { LoadIconW(hinstance, 1 as _) as isize };
            if hicon == 0 {
                hicon = unsafe { GetClassLongPtrW(hwnd as HWND, GCLP_HICON) as isize };
            }
            if hicon == 0 {
                hicon = unsafe { LoadIconW(0 as _, IDI_APPLICATION) as isize };
            }

            let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd as HWND;
            nid.uID = TRAY_ICON_UID;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon as _;

            Self::encode_tip(&mut nid.szTip, tip);

            let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) } != 0;
            if ok {
                self.installed.store(true, Ordering::Relaxed);
            }
            ok
        }
        #[cfg(not(windows))]
        {
            let _ = (hwnd, tip);
            false
        }
    }

    /// Updates the tooltip of the SpotiVista tray icon.
    pub fn update_tooltip(&self, hwnd: isize, tip: &str) -> bool {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HWND;
            use windows_sys::Win32::UI::Shell::{
                NOTIFYICONDATAW, NIM_MODIFY, NIF_TIP, Shell_NotifyIconW,
            };

            if hwnd == 0 || !self.is_installed() {
                return false;
            }

            let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd as HWND;
            nid.uID = TRAY_ICON_UID;
            nid.uFlags = NIF_TIP;

            Self::encode_tip(&mut nid.szTip, tip);

            unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) != 0 }
        }
        #[cfg(not(windows))]
        {
            let _ = (hwnd, tip);
            false
        }
    }

    /// Removes the SpotiVista icon from the Windows Notification Area.
    pub fn remove(&self, hwnd: isize) -> bool {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HWND;
            use windows_sys::Win32::UI::Shell::{
                NOTIFYICONDATAW, NIM_DELETE, Shell_NotifyIconW,
            };

            if hwnd == 0 || !self.is_installed() {
                return false;
            }

            let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd as HWND;
            nid.uID = TRAY_ICON_UID;

            let ok = unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) } != 0;
            self.installed.store(false, Ordering::Relaxed);
            ok
        }
        #[cfg(not(windows))]
        {
            let _ = hwnd;
            false
        }
    }

    /// Displays the tray context popup menu at the current cursor position.
    pub fn show_context_menu(&self, hwnd: isize, track_info: Option<&str>, is_playing: bool) -> Option<TrayAction> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{HWND, POINT};
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos,
                MF_DISABLED, MF_GRAYED, MF_SEPARATOR, MF_STRING, PostMessageW,
                SetForegroundWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_NONOTIFY,
                TPM_RETURNCMD, TrackPopupMenuEx, WM_NULL,
            };

            if hwnd == 0 {
                return None;
            }

            let hmenu = unsafe { CreatePopupMenu() };
            if hmenu.is_null() {
                return None;
            }

            let to_wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };

            // Title / Now playing info
            let title = track_info.unwrap_or("SpotiVista");
            let title_wide = to_wide(title);
            unsafe {
                AppendMenuW(hmenu, MF_STRING | MF_GRAYED | MF_DISABLED, CMD_NOW_PLAYING, title_wide.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
            }

            // Playback controls
            let play_pause_label = if is_playing { "Pause\0" } else { "Play\0" };
            let play_pause_wide: Vec<u16> = play_pause_label.encode_utf16().collect();
            let next_wide = to_wide("Next");
            let prev_wide = to_wide("Previous");

            unsafe {
                AppendMenuW(hmenu, MF_STRING, CMD_PLAY_PAUSE, play_pause_wide.as_ptr());
                AppendMenuW(hmenu, MF_STRING, CMD_NEXT, next_wide.as_ptr());
                AppendMenuW(hmenu, MF_STRING, CMD_PREV, prev_wide.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
            }

            // Window restore & Exit
            let restore_wide = to_wide("Open SpotiVista");
            let exit_wide = to_wide("Exit");

            unsafe {
                AppendMenuW(hmenu, MF_STRING, CMD_RESTORE, restore_wide.as_ptr());
                AppendMenuW(hmenu, MF_STRING, CMD_EXIT, exit_wide.as_ptr());
            }

            let mut pt = POINT { x: 0, y: 0 };
            unsafe { GetCursorPos(&mut pt) };

            // Standard Win32 workaround for popup menus on notify icons
            unsafe {
                SetForegroundWindow(hwnd as HWND);
            }

            let cmd = unsafe {
                TrackPopupMenuEx(
                    hmenu,
                    TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY,
                    pt.x,
                    pt.y,
                    hwnd as HWND,
                    std::ptr::null(),
                )
            };

            unsafe {
                PostMessageW(hwnd as HWND, WM_NULL, 0, 0);
                DestroyMenu(hmenu);
            }

            match cmd as usize {
                CMD_PLAY_PAUSE => Some(TrayAction::PlayPause),
                CMD_NEXT => Some(TrayAction::Next),
                CMD_PREV => Some(TrayAction::Previous),
                CMD_RESTORE => Some(TrayAction::RestoreWindow),
                CMD_EXIT => Some(TrayAction::Exit),
                _ => None,
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (hwnd, track_info, is_playing);
            None
        }
    }

    fn encode_tip(buf: &mut [u16; 128], tip: &str) {
        let mut idx = 0;
        for c in tip.encode_utf16() {
            if idx >= 127 {
                break;
            }
            buf[idx] = c;
            idx += 1;
        }
        buf[idx] = 0;
    }
}

impl Default for TrayController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_tip_properly() {
        let mut buf = [0u16; 128];
        TrayController::encode_tip(&mut buf, "Artist - Track");
        let wide: Vec<u16> = "Artist - Track\0".encode_utf16().collect();
        assert_eq!(&buf[..wide.len()], &wide[..]);
    }

    #[test]
    fn truncates_long_tip() {
        let mut buf = [0u16; 128];
        let long_str = "a".repeat(200);
        TrayController::encode_tip(&mut buf, &long_str);
        assert_eq!(buf[127], 0);
    }
}
