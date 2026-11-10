//! Windows Vista SP2 and Windows 7 SP1 backward-compatibility shims
//!
//! Windows 8 and later introduced APIs that are not available in Windows Vista SP2
//! and Windows 7 SP1.
//!
//! Providing these symbols here with `#[unsafe(no_mangle)]` enables the MSVC linker
//! to resolve them internally rather than creating hard PE import table entries
//! against Windows 8+ functions or api-ms-win-* mincore DLLs.
//!
//! On Windows 8 and newer, these implementations dynamically forward to the native
//! OS implementation via GetProcAddress. On Windows Vista / 7, they provide safe fallbacks.

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
        let mut is_wow64: i32 = 0;
        let ok = unsafe { windows_sys::Win32::System::Threading::IsWow64Process(h_process as _, &mut is_wow64) };
        if ok != 0 {
            if !native_machine.is_null() {
                unsafe { *native_machine = 0x8664 }; // IMAGE_FILE_MACHINE_AMD64
            }
            if !process_machine.is_null() {
                unsafe { *process_machine = if is_wow64 != 0 { 0x014c } else { 0 } };
            }
            1
        } else {
            0
        }
    }
}

// ============================================================================
// 4. Power Notifications (Windows 8+ powrprof.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerRegisterSuspendResumeNotification(
    flags: u32,
    recipient: *mut c_void,
    registration_handle: *mut *mut c_void,
) -> u32 {
    type Func = unsafe extern "system" fn(u32, *mut c_void, *mut *mut c_void) -> u32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let pwr = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"powrprof.dll\0".as_ptr()) };
        if !pwr.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(pwr, b"PowerRegisterSuspendResumeNotification\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(flags, recipient, registration_handle) }
    } else {
        windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerUnregisterSuspendResumeNotification(
    handle: *mut c_void,
) -> u32 {
    type Func = unsafe extern "system" fn(*mut c_void) -> u32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let pwr = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"powrprof.dll\0".as_ptr()) };
        if !pwr.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(pwr, b"PowerUnregisterSuspendResumeNotification\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(handle) }
    } else {
        windows_sys::Win32::Foundation::ERROR_SUCCESS
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerSettingRegisterNotification(
    setting_guid: *const c_void,
    flags: u32,
    recipient: *mut c_void,
    registration_handle: *mut *mut c_void,
) -> u32 {
    type Func = unsafe extern "system" fn(*const c_void, u32, *mut c_void, *mut *mut c_void) -> u32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let pwr = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"powrprof.dll\0".as_ptr()) };
        if !pwr.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(pwr, b"PowerSettingRegisterNotification\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(setting_guid, flags, recipient, registration_handle) }
    } else {
        windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerSettingUnregisterNotification(
    handle: *mut c_void,
) -> u32 {
    type Func = unsafe extern "system" fn(*mut c_void) -> u32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let pwr = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"powrprof.dll\0".as_ptr()) };
        if !pwr.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(pwr, b"PowerSettingUnregisterNotification\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(handle) }
    } else {
        windows_sys::Win32::Foundation::ERROR_SUCCESS
    }
}

// ============================================================================
// 5. Audio Interface Async (Windows 8+ mmdevapi.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ActivateAudioInterfaceAsync(
    device_interface_path: *const u16,
    riid: *const c_void,
    activation_params: *const c_void,
    completion_handler: *mut c_void,
    activation_operation: *mut *mut c_void,
) -> i32 {
    type Func = unsafe extern "system" fn(*const u16, *const c_void, *const c_void, *mut c_void, *mut *mut c_void) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let mm = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"mmdevapi.dll\0".as_ptr()) };
        if !mm.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(mm, b"ActivateAudioInterfaceAsync\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(device_interface_path, riid, activation_params, completion_handler, activation_operation) }
    } else {
        // E_NOTIMPL on Vista / 7
        0x80004001u32 as i32
    }
}

// ============================================================================
// 6. WinRT Error Reporting (Windows 8+)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RoOriginateErrorW(_error: i32, _cch_max: u32, _message: *const u16) -> i32 {
    0
}

// ============================================================================
// 7. Address Synchronization (Windows 8+ / api-ms-win-core-synch-l1-2-0.dll)
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

// ============================================================================
// 8. Random Bytes (Windows 10+ bcryptprimitives.dll ProcessPrng)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ProcessPrng(data: *mut u8, len: usize) -> i32 {
    type Func = unsafe extern "system" fn(*mut u8, usize) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let bp = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"bcryptprimitives.dll\0".as_ptr()) };
        if !bp.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(bp, b"ProcessPrng\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        return unsafe { f(data, len) };
    }
    // Windows Vista / 7: RtlGenRandom (SystemFunction036) from advapi32.dll
    type RtlGenRandom = unsafe extern "system" fn(*mut u8, u32) -> u8;
    let adv = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"advapi32.dll\0".as_ptr()) };
    if adv.is_null() {
        return 0;
    }
    let Some(gen_proc) = (unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(adv, b"SystemFunction036\0".as_ptr()) }) else {
        return 0;
    };
    let gen_fn: RtlGenRandom = unsafe { std::mem::transmute(gen_proc) };
    let mut offset = 0usize;
    while offset < len {
        let chunk = (len - offset).min(u32::MAX as usize) as u32;
        if unsafe { gen_fn(data.add(offset), chunk) } == 0 {
            return 0;
        }
        offset += chunk as usize;
    }
    1
}
