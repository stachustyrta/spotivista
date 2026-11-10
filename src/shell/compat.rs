//! Windows Vista SP2 and Windows 7 SP1 backward-compatibility shims
//!
//! Windows 8 and later introduced APIs that are not available in Windows Vista SP2
//! and Windows 7 SP1.
//!
//! Providing these symbols here with `#[unsafe(no_mangle)]` enables the MSVC linker
//! to resolve them internally rather than creating hard PE import table entries
//! against Windows 8+ kernel32 functions or api-ms-win-* mincore DLLs.
//!
//! On Windows 8 and newer, these implementations dynamically forward to the native
//! OS implementation. On Windows Vista / 7, they provide safe fallbacks.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

// ============================================================================
// 1. Process Power Throttling / Information (Windows 8+)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn SetProcessInformation(
    h_process: *mut c_void,
    class: u32,
    info: *mut c_void,
    size: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetProcessInformation\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(h_process, class, info, size) }
    } else {
        unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED); }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn GetProcessInformation(
    h_process: *mut c_void,
    class: u32,
    info: *mut c_void,
    size: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32, *mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"GetProcessInformation\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(h_process, class, info, size) }
    } else {
        unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED); }
        0
    }
}

// ============================================================================
// 2. High-Precision Time (Windows 8+)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn GetSystemTimePreciseAsFileTime(file_time: *mut windows_sys::Win32::Foundation::FILETIME) {
    type Func = unsafe extern "system" fn(*mut windows_sys::Win32::Foundation::FILETIME);
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"GetSystemTimePreciseAsFileTime\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(file_time) }
    } else {
        // Vista / 7 fallback
        unsafe { windows_sys::Win32::System::SystemInformation::GetSystemTimeAsFileTime(file_time) }
    }
}

// ============================================================================
// 3. Extended Architecture Query (Windows 10 RS3+)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn IsWow64Process2(
    h_process: *mut c_void,
    process_machine: *mut u16,
    native_machine: *mut u16,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, *mut u16, *mut u16) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"IsWow64Process2\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(h_process, process_machine, native_machine) }
    } else {
        // Fallback using Vista/7 IsWow64Process
        let mut is_wow64: i32 = 0;
        let ok = unsafe { windows_sys::Win32::System::Threading::IsWow64Process(h_process as _, &mut is_wow64) };
        if ok != 0 {
            if !native_machine.is_null() {
                unsafe { *native_machine = 0x8664 }; // IMAGE_FILE_MACHINE_AMD64
            }
            if !process_machine.is_null() {
                unsafe { *process_machine = if is_wow64 != 0 { 0x014c } else { 0 } }; // IMAGE_FILE_MACHINE_I386 or UNKNOWN
            }
            1
        } else {
            0
        }
    }
}

// ============================================================================
// 4. WinRT Error Reporting (Windows 8+)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RoOriginateErrorW(_error: i32, _cch_max: u32, _message: *const u16) -> i32 {
    0
}

// ============================================================================
// 5. Address Synchronization (Windows 8+ / api-ms-win-core-synch-l1-2-0.dll)
// ============================================================================

unsafe fn memory_matches(a: *const u8, b: *const u8, size: usize) -> bool {
    unsafe { std::slice::from_raw_parts(a, size) == std::slice::from_raw_parts(b, size) }
}

fn fallback_wait_on_address(
    address: *mut c_void,
    compare_address: *mut c_void,
    address_size: usize,
    dw_milliseconds: u32,
) -> i32 {
    let a = address as *const u8;
    let b = compare_address as *const u8;
    if !unsafe { memory_matches(a, b, address_size) } {
        return 1;
    }
    let start = std::time::Instant::now();
    let timeout = if dw_milliseconds == 0xFFFFFFFF {
        None
    } else {
        Some(std::time::Duration::from_millis(dw_milliseconds as u64))
    };
    loop {
        if !unsafe { memory_matches(a, b, address_size) } {
            return 1;
        }
        if let Some(to) = timeout {
            if start.elapsed() >= to {
                unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_TIMEOUT); }
                return 0;
            }
        }
        unsafe { windows_sys::Win32::System::Threading::Sleep(1); }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WaitOnAddress(
    address: *mut c_void,
    compare_address: *mut c_void,
    address_size: usize,
    dw_milliseconds: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, *mut c_void, usize, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let synch = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"api-ms-win-core-synch-l1-2-0.dll\0".as_ptr()) };
        if !synch.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(synch, b"WaitOnAddress\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(address, compare_address, address_size, dw_milliseconds) }
    } else {
        fallback_wait_on_address(address, compare_address, address_size, dw_milliseconds)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WakeByAddressSingle(address: *mut c_void) {
    type Func = unsafe extern "system" fn(*mut c_void);
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let synch = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"api-ms-win-core-synch-l1-2-0.dll\0".as_ptr()) };
        if !synch.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(synch, b"WakeByAddressSingle\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(address) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WakeByAddressAll(address: *mut c_void) {
    type Func = unsafe extern "system" fn(*mut c_void);
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let synch = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"api-ms-win-core-synch-l1-2-0.dll\0".as_ptr()) };
        if !synch.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(synch, b"WakeByAddressAll\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(address) };
    }
}
