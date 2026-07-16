// SPDX-License-Identifier: AGPL-3.0-or-later
//! C-FFI entry points for platform embedding.
//!
//! These functions constitute the ABI surface exposed to host applications
//! (Android JNI, iOS Swift interop, C#, etc.). They follow C calling conventions
//! and manage memory via opaque handle pointers.
//!
//! # Safety Contract
//!
//! - All `*mut PetalTongueHandle` arguments must be non-null pointers obtained
//!   from [`pt_create`] and not yet freed by [`pt_destroy`].
//! - All `*const c_char` arguments must be valid, NUL-terminated C strings.
//! - Returned `*mut c_char` strings must be freed by calling [`pt_free_string`].
//! - The handle is NOT thread-safe — the host must serialize calls.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::config::EmbedConfig;
use crate::lifecycle::PlatformLifecycle;
use crate::runtime::EmbeddedRuntime;

/// Opaque handle exposed across the FFI boundary.
///
/// The host application holds a pointer to this and passes it back to each
/// `pt_*` function. Internally it wraps an [`EmbeddedRuntime`].
pub struct PetalTongueHandle {
    runtime: EmbeddedRuntime,
}

/// Create a new petalTongue runtime instance.
///
/// # Arguments
/// * `config_json` — NUL-terminated JSON string of [`EmbedConfig`], or NULL for defaults.
///
/// # Returns
/// An opaque handle pointer, or NULL on failure.
///
/// # Safety
/// `config_json` must be NULL or a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_create(config_json: *const c_char) -> *mut PetalTongueHandle {
    let config = if config_json.is_null() {
        EmbedConfig::new(crate::config::Platform::Desktop)
    } else {
        let c_str = unsafe { CStr::from_ptr(config_json) };
        let Ok(json) = c_str.to_str() else {
            return std::ptr::null_mut();
        };
        let Ok(c) = serde_json::from_str::<EmbedConfig>(json) else {
            return std::ptr::null_mut();
        };
        c
    };

    EmbeddedRuntime::new(config).map_or(std::ptr::null_mut(), |runtime| {
        Box::into_raw(Box::new(PetalTongueHandle { runtime }))
    })
}

/// Start the runtime (connect transport, enable rendering).
///
/// # Returns
/// 0 on success, -1 on error.
///
/// # Safety
/// `handle` must be a valid pointer from [`pt_create`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_start(handle: *mut PetalTongueHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let h = unsafe { &mut *handle };
    if h.runtime.start().is_ok() { 0 } else { -1 }
}

/// Pause the runtime (reduce activity for background state).
///
/// # Returns
/// 0 on success, -1 on error.
///
/// # Safety
/// `handle` must be a valid pointer from [`pt_create`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_pause(handle: *mut PetalTongueHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let h = unsafe { &mut *handle };
    if h.runtime.on_pause().is_ok() { 0 } else { -1 }
}

/// Resume the runtime from paused state.
///
/// # Returns
/// 0 on success, -1 on error.
///
/// # Safety
/// `handle` must be a valid pointer from [`pt_create`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_resume(handle: *mut PetalTongueHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let h = unsafe { &mut *handle };
    if h.runtime.on_resume().is_ok() { 0 } else { -1 }
}

/// Render a scenario scene to SVG.
///
/// # Arguments
/// * `builder_id` — NUL-terminated scenario builder ID (e.g. `"airspring.et0"`)
/// * `scene_name` — NUL-terminated scene name (e.g. `"daily_et0"`)
///
/// # Returns
/// A heap-allocated NUL-terminated SVG string (caller must free with [`pt_free_string`]),
/// or NULL on error.
///
/// # Safety
/// `handle`, `builder_id`, and `scene_name` must be valid non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_render_svg(
    handle: *mut PetalTongueHandle,
    builder_id: *const c_char,
    scene_name: *const c_char,
) -> *mut c_char {
    if handle.is_null() || builder_id.is_null() || scene_name.is_null() {
        return std::ptr::null_mut();
    }
    let h = unsafe { &*handle };

    let Ok(bid) = (unsafe { CStr::from_ptr(builder_id) }).to_str() else {
        return std::ptr::null_mut();
    };
    let Ok(sname) = (unsafe { CStr::from_ptr(scene_name) }).to_str() else {
        return std::ptr::null_mut();
    };

    let Ok(svg) = h.runtime.render_svg(bid, sname) else {
        return std::ptr::null_mut();
    };
    CString::new(svg).map_or(std::ptr::null_mut(), CString::into_raw)
}

/// Process a JSON-RPC request and return the JSON response.
///
/// # Arguments
/// * `json` — NUL-terminated JSON-RPC request string.
///
/// # Returns
/// A heap-allocated NUL-terminated JSON response string (caller must free with
/// [`pt_free_string`]), or NULL on error.
///
/// # Safety
/// `handle` and `json` must be valid non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_ipc_request(
    handle: *mut PetalTongueHandle,
    json: *const c_char,
) -> *mut c_char {
    if handle.is_null() || json.is_null() {
        return std::ptr::null_mut();
    }
    let h = unsafe { &*handle };

    let Ok(request) = (unsafe { CStr::from_ptr(json) }).to_str() else {
        return std::ptr::null_mut();
    };

    let Ok(response) = h.runtime.ipc_request(request) else {
        return std::ptr::null_mut();
    };
    CString::new(response).map_or(std::ptr::null_mut(), CString::into_raw)
}

/// Free a string returned by any `pt_*` function.
///
/// # Safety
/// `ptr` must be a pointer previously returned by a `pt_*` function, or NULL.
/// After this call, the pointer is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}

/// Destroy the runtime and free all resources.
///
/// After this call, the handle is invalid and must not be used.
///
/// # Safety
/// `handle` must be a valid pointer from [`pt_create`], or NULL (no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pt_destroy(handle: *mut PetalTongueHandle) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}
