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

// ============================================================================
// 9. PSAPI Performance Information (Windows 7+ kernel32.dll / Vista psapi.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn K32GetPerformanceInfo(
    p_performance_information: *mut c_void,
    cb: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        // 1. Try kernel32.dll for K32GetPerformanceInfo (Windows 7+)
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        let mut proc = if !k32.is_null() {
            unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"K32GetPerformanceInfo\0".as_ptr()) }
        } else {
            None
        };
        // 2. If not found in kernel32 (Windows Vista / XP), load psapi.dll and lookup GetPerformanceInfo
        if proc.is_none() {
            let psapi = unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"psapi.dll\0".as_ptr()) };
            if !psapi.is_null() {
                proc = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(psapi, b"GetPerformanceInfo\0".as_ptr()) };
            }
        }
        if let Some(p) = proc {
            FUNC.store(p as *mut c_void, Ordering::Release);
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(p_performance_information, cb) }
    } else {
        unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED); }
        0
    }
}

// ============================================================================
// 10. Thread Error Mode (Windows 7+ kernel32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn SetThreadErrorMode(
    dw_new_mode: u32,
    lp_old_mode: *mut u32,
) -> i32 {
    type Func = unsafe extern "system" fn(u32, *mut u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetThreadErrorMode\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(dw_new_mode, lp_old_mode) }
    } else {
        // Fallback for Windows Vista: SetErrorMode sets process error mode and returns previous mode
        type SetErrorModeFn = unsafe extern "system" fn(u32) -> u32;
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            if let Some(proc) = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetErrorMode\0".as_ptr()) } {
                let f: SetErrorModeFn = unsafe { std::mem::transmute(proc) };
                let prev = unsafe { f(dw_new_mode) };
                if !lp_old_mode.is_null() {
                    unsafe { *lp_old_mode = prev; }
                }
                return 1;
            }
        }
        1
    }
}

// ============================================================================
// 11. Power Availability Requests (Windows 7+ kernel32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerCreateRequest(
    context: *const c_void,
) -> *mut c_void {
    type Func = unsafe extern "system" fn(*const c_void) -> *mut c_void;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"PowerCreateRequest\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(context) }
    } else {
        // Vista fallback: create a valid dummy event handle so callers can pass it to CloseHandle safely
        let h = unsafe { windows_sys::Win32::System::Threading::CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
        h as *mut c_void
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerSetRequest(
    power_request: *mut c_void,
    request_type: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"PowerSetRequest\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(power_request, request_type) }
    } else {
        // Fallback for Windows Vista: SetThreadExecutionState
        type SetThreadExecutionStateFn = unsafe extern "system" fn(u32) -> u32;
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            if let Some(proc) = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetThreadExecutionState\0".as_ptr()) } {
                let f: SetThreadExecutionStateFn = unsafe { std::mem::transmute(proc) };
                const ES_CONTINUOUS: u32 = 0x80000000;
                const ES_SYSTEM_REQUIRED: u32 = 0x00000001;
                const ES_DISPLAY_REQUIRED: u32 = 0x00000002;
                let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
                if request_type == 0 { // PowerRequestDisplayRequired
                    flags |= ES_DISPLAY_REQUIRED;
                }
                unsafe { f(flags) };
            }
        }
        1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn PowerClearRequest(
    power_request: *mut c_void,
    request_type: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"PowerClearRequest\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(power_request, request_type) }
    } else {
        type SetThreadExecutionStateFn = unsafe extern "system" fn(u32) -> u32;
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            if let Some(proc) = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetThreadExecutionState\0".as_ptr()) } {
                let f: SetThreadExecutionStateFn = unsafe { std::mem::transmute(proc) };
                const ES_CONTINUOUS: u32 = 0x80000000;
                unsafe { f(ES_CONTINUOUS) };
            }
        }
        1
    }
}

// ============================================================================
// 12. Window Display Affinity (Windows 7+ user32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn SetWindowDisplayAffinity(
    hwnd: *mut c_void,
    affinity: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"SetWindowDisplayAffinity\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(hwnd, affinity) }
    } else {
        1
    }
}

// ============================================================================
// 13. UIPI Window Message Filter (Windows 7+ user32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ChangeWindowMessageFilterEx(
    hwnd: *mut c_void,
    message: u32,
    action: u32,
    filter_struct: *mut c_void,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32, u32, *mut c_void) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"ChangeWindowMessageFilterEx\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(hwnd, message, action, filter_struct) }
    } else {
        // Fallback for Windows Vista: ChangeWindowMessageFilter(message, action)
        type VistaFunc = unsafe extern "system" fn(u32, u32) -> i32;
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"ChangeWindowMessageFilter\0".as_ptr()) };
            if let Some(proc) = p {
                let f: VistaFunc = unsafe { std::mem::transmute(proc) };
                return unsafe { f(message, action) };
            }
        }
        1
    }
}

// ============================================================================
// 14. Windows Touch APIs (Windows 7+ user32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn RegisterTouchWindow(
    hwnd: *mut c_void,
    flags: u32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"RegisterTouchWindow\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(hwnd, flags) }
    } else {
        unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED); }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn GetTouchInputInfo(
    touch_input: *mut c_void,
    inputs: u32,
    p_inputs: *mut c_void,
    cb_size: i32,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u32, *mut c_void, i32) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"GetTouchInputInfo\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(touch_input, inputs, p_inputs, cb_size) }
    } else {
        unsafe { windows_sys::Win32::Foundation::SetLastError(windows_sys::Win32::Foundation::ERROR_NOT_SUPPORTED); }
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CloseTouchInputHandle(
    touch_input: *mut c_void,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let u32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"user32.dll\0".as_ptr()) };
        if !u32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(u32, b"CloseTouchInputHandle\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(touch_input) }
    } else {
        1
    }
}

// ============================================================================
// 15. Notification Area Geometry (Windows 7+ shell32.dll)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Shell_NotifyIconGetRect(
    identifier: *const c_void,
    icon_location: *mut c_void,
) -> i32 {
    type Func = unsafe extern "system" fn(*const c_void, *mut c_void) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let s32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"shell32.dll\0".as_ptr()) };
        if !s32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(s32, b"Shell_NotifyIconGetRect\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        unsafe { f(identifier, icon_location) }
    } else {
        0x80004001u32 as i32 // E_NOTIMPL on Windows Vista
    }
}

// ============================================================================
// 16. File Completion Notification Modes (Windows Vista+ kernel32.dll / AFD fix)
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "system" fn SetFileCompletionNotificationModes(
    file_handle: *mut c_void,
    flags: u8,
) -> i32 {
    type Func = unsafe extern "system" fn(*mut c_void, u8) -> i32;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"SetFileCompletionNotificationModes\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        let ret = unsafe { f(file_handle, flags) };
        if ret != 0 {
            return ret;
        }
        // On Windows Vista, SetFileCompletionNotificationModes on \Device\Afd handles
        // opened with SYNCHRONIZE fails with ERROR_ACCESS_DENIED (5) or ERROR_INVALID_FUNCTION (1)
        // because Vista strictly requires write access on notification mode changes.
        // FILE_SKIP_SET_EVENT_ON_HANDLE is an optimization to skip handle signaling on completion.
        // Returning success (1) allows mio and Tokio IOCP socket polling to proceed safely.
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        if err == windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED
            || err == windows_sys::Win32::Foundation::ERROR_INVALID_FUNCTION
        {
            unsafe { windows_sys::Win32::Foundation::SetLastError(0); }
            return 1;
        }
        0
    } else {
        1
    }
}

// ============================================================================
// 17. FindFirstFileExW Fallback for Windows Vista (os error 87 fix)
// ============================================================================

#[allow(non_snake_case)]
pub unsafe extern "system" fn hooked_FindFirstFileExW(
    file_name: *const u16,
    info_level_id: i32,
    find_file_data: *mut c_void,
    search_op: i32,
    search_filter: *mut c_void,
    additional_flags: u32,
) -> *mut c_void {
    type Func = unsafe extern "system" fn(
        *const u16,
        i32,
        *mut c_void,
        i32,
        *mut c_void,
        u32,
    ) -> *mut c_void;
    static FUNC: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    static INIT: AtomicBool = AtomicBool::new(false);
    if !INIT.load(Ordering::Acquire) {
        let k32 = unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
        if !k32.is_null() {
            let p = unsafe { windows_sys::Win32::System::LibraryLoader::GetProcAddress(k32, b"FindFirstFileExW\0".as_ptr()) };
            if let Some(proc) = p {
                FUNC.store(proc as *mut c_void, Ordering::Release);
            }
        }
        INIT.store(true, Ordering::Release);
    }
    let p = FUNC.load(Ordering::Acquire);
    if !p.is_null() {
        let f: Func = unsafe { std::mem::transmute(p) };
        let ret = unsafe {
            f(
                file_name,
                info_level_id,
                find_file_data,
                search_op,
                search_filter,
                additional_flags,
            )
        };
        let invalid = -1isize as *mut c_void;
        if ret != invalid {
            return ret;
        }
        // If it failed with ERROR_INVALID_PARAMETER (87):
        // On Windows Vista, FindExInfoBasic (1) and FIND_FIRST_EX_LARGE_FETCH (2)
        // are not supported and return ERROR_INVALID_PARAMETER.
        // Fall back to FindExInfoStandard (0) and clear the large fetch flag.
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        if err == windows_sys::Win32::Foundation::ERROR_INVALID_PARAMETER {
            let fallback_level = if info_level_id == 1 { 0 } else { info_level_id };
            let fallback_flags = additional_flags & !2;
            let retry = unsafe {
                f(
                    file_name,
                    fallback_level,
                    find_file_data,
                    search_op,
                    search_filter,
                    fallback_flags,
                )
            };
            return retry;
        }
        ret
    } else {
        -1isize as *mut c_void
    }
}

// ============================================================================
// 18. In-Memory IAT Hooking for Windows Vista Compatibility
// ============================================================================

/// Patches the Import Address Table of the current process module to redirect
/// Windows Vista incompatible API calls to our shims.
pub unsafe fn patch_iat_for_vista() {
    unsafe {
        let module = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        if module.is_null() {
            return;
        }
        let base = module as usize;
        let dos_header = base as *const u8;
        if *dos_header != b'M' || *(dos_header.add(1)) != b'Z' {
            return;
        }
        let e_lfanew = *(base.wrapping_add(0x3C) as *const i32) as usize;
        let nt_headers = (base + e_lfanew) as *const u8;
        if *nt_headers != b'P' || *(nt_headers.add(1)) != b'E' {
            return;
        }
        let optional_header = nt_headers.add(24);
        let magic = *(optional_header as *const u16);
        // 0x020B is PE32+ (64-bit)
        if magic != 0x020B {
            return;
        }
        // Import Directory is DataDirectory[1] at offset 112 from OptionalHeader
        let import_dir_rva = *(optional_header.add(112) as *const u32) as usize;
        if import_dir_rva == 0 {
            return;
        }

        #[repr(C)]
        struct ImportDescriptor {
            original_first_thunk: u32,
            time_date_stamp: u32,
            forwarder_chain: u32,
            name: u32,
            first_thunk: u32,
        }

        let mut desc = (base + import_dir_rva) as *const ImportDescriptor;
        while (*desc).name != 0 {
            let dll_name_ptr = (base + (*desc).name as usize) as *const u8;
            let mut len = 0;
            while *(dll_name_ptr.add(len)) != 0 {
                len += 1;
            }
            let dll_name = std::slice::from_raw_parts(dll_name_ptr, len);
            if dll_name.eq_ignore_ascii_case(b"kernel32.dll") {
                let thunk_rva = if (*desc).original_first_thunk != 0 {
                    (*desc).original_first_thunk as usize
                } else {
                    (*desc).first_thunk as usize
                };
                let mut orig_thunk = (base + thunk_rva) as *const usize;
                let mut iat_thunk = (base + (*desc).first_thunk as usize) as *mut usize;

                while *orig_thunk != 0 {
                    // Not an ordinal import
                    if (*orig_thunk & 0x8000_0000_0000_0000) == 0 {
                        let import_by_name = (base + (*orig_thunk & 0x7FFF_FFFF)) as *const u8;
                        // Skip hint (u16 at offset 0), function name begins at offset 2
                        let func_name_ptr = import_by_name.add(2);
                        let mut fn_len = 0;
                        while *(func_name_ptr.add(fn_len)) != 0 {
                            fn_len += 1;
                        }
                        let func_name = std::slice::from_raw_parts(func_name_ptr, fn_len);

                        let replacement: Option<usize> = if func_name == b"SetFileCompletionNotificationModes" {
                            Some(SetFileCompletionNotificationModes as *const () as usize)
                        } else if func_name == b"FindFirstFileExW" {
                            Some(hooked_FindFirstFileExW as *const () as usize)
                        } else {
                            None
                        };

                        if let Some(target_fn) = replacement {
                            use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};
                            unsafe extern "system" {
                                fn FlushInstructionCache(
                                    process: *mut c_void,
                                    base_address: *const c_void,
                                    size: usize,
                                ) -> i32;
                            }
                            let mut old_protect = 0;
                            if VirtualProtect(iat_thunk as *mut _, 8, PAGE_EXECUTE_READWRITE, &mut old_protect) != 0 {
                                *iat_thunk = target_fn;
                                VirtualProtect(iat_thunk as *mut _, 8, old_protect, &mut old_protect);
                                FlushInstructionCache(
                                    windows_sys::Win32::System::Threading::GetCurrentProcess() as *mut c_void,
                                    iat_thunk as *const _,
                                    8,
                                );
                            }
                        }
                    }
                    orig_thunk = orig_thunk.add(1);
                    iat_thunk = iat_thunk.add(1);
                }
            }
            desc = desc.add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_prng_fills_buffer() {
        let mut buf = [0u8; 32];
        let res = unsafe { ProcessPrng(buf.as_mut_ptr(), buf.len()) };
        assert_eq!(res, 1);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn k32_get_performance_info_executes_safely() {
        let mut perf_info = [0usize; 13];
        let cb = std::mem::size_of_val(&perf_info) as u32;
        let res = unsafe { K32GetPerformanceInfo(perf_info.as_mut_ptr() as *mut c_void, cb) };
        assert_eq!(res, 1);
    }

    #[test]
    fn set_thread_error_mode_executes_safely() {
        let mut old_mode: u32 = 0;
        let res = unsafe { SetThreadErrorMode(1, &mut old_mode) };
        assert_eq!(res, 1);
        let mut dummy: u32 = 0;
        let res2 = unsafe { SetThreadErrorMode(old_mode, &mut dummy) };
        assert_eq!(res2, 1);
    }

    #[test]
    fn power_request_shims_execute_safely() {
        let req = unsafe { PowerCreateRequest(std::ptr::null()) };
        assert!(!req.is_null());
        let set_res = unsafe { PowerSetRequest(req, 1) };
        assert_eq!(set_res, 1);
        let clear_res = unsafe { PowerClearRequest(req, 1) };
        assert_eq!(clear_res, 1);
        unsafe { windows_sys::Win32::Foundation::CloseHandle(req as _) };
    }

    #[test]
    fn touch_and_filter_shims_execute_safely() {
        let filter_res = unsafe { ChangeWindowMessageFilterEx(std::ptr::null_mut(), 0, 1, std::ptr::null_mut()) };
        assert_eq!(filter_res, 1);
        let affinity_res = unsafe { SetWindowDisplayAffinity(std::ptr::null_mut(), 0) };
        assert_eq!(affinity_res, 1);
    }

    #[test]
    fn set_file_completion_notification_modes_executes_safely() {
        // Invalid handle: real API fails with ERROR_INVALID_HANDLE (6) or similar, which is preserved
        let res = unsafe { SetFileCompletionNotificationModes(std::ptr::null_mut(), 2) };
        // Valid execution: either succeeds or handles the call without crashing
        assert!(res == 0 || res == 1);
    }

    #[test]
    fn find_first_file_ex_w_executes_safely() {
        let path = [b'C' as u16, b':' as u16, b'\\' as u16, b'*' as u16, 0];
        let mut data = [0u8; 600];
        let handle = unsafe {
            hooked_FindFirstFileExW(
                path.as_ptr(),
                1, // FindExInfoBasic
                data.as_mut_ptr() as *mut c_void,
                0,
                std::ptr::null_mut(),
                2, // FIND_FIRST_EX_LARGE_FETCH
            )
        };
        if handle != -1isize as *mut c_void {
            unsafe { windows_sys::Win32::Storage::FileSystem::FindClose(handle as _) };
        }
    }

    #[test]
    fn patch_iat_for_vista_runs_safely() {
        unsafe { patch_iat_for_vista(); }
    }
}
