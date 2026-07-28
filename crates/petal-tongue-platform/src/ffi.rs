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
        // SAFETY: caller guarantees config_json is a valid NUL-terminated C string.
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
    // SAFETY: null-checked above; caller guarantees valid handle from pt_create.
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
    // SAFETY: null-checked above; caller guarantees valid handle from pt_create.
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
    // SAFETY: null-checked above; caller guarantees valid handle from pt_create.
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
    // SAFETY: null-checked above; caller guarantees valid handle from pt_create.
    let h = unsafe { &*handle };

    // SAFETY: null-checked above; caller guarantees valid NUL-terminated C strings.
    let Ok(bid) = (unsafe { CStr::from_ptr(builder_id) }).to_str() else {
        return std::ptr::null_mut();
    };
    // SAFETY: same as above.
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
    // SAFETY: null-checked above; caller guarantees valid handle from pt_create.
    let h = unsafe { &mut *handle };

    // SAFETY: null-checked above; caller guarantees valid NUL-terminated C string.
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
        // SAFETY: caller guarantees ptr was returned by a pt_* function (allocated via CString::into_raw).
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
        // SAFETY: null-checked; caller guarantees valid handle from pt_create, not yet destroyed.
        drop(unsafe { Box::from_raw(handle) });
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    #[test]
    fn create_with_null_config_returns_handle() {
        // SAFETY: NULL config triggers default Desktop platform.
        let handle = unsafe { pt_create(std::ptr::null()) };
        assert!(!handle.is_null());
        // SAFETY: handle is valid from pt_create above.
        unsafe { pt_destroy(handle) };
    }

    #[test]
    fn create_with_valid_json_returns_handle() {
        let json = CString::new(r#"{"platform":"desktop"}"#).unwrap();
        // SAFETY: json is a valid NUL-terminated C string.
        let handle = unsafe { pt_create(json.as_ptr()) };
        assert!(!handle.is_null());
        // SAFETY: handle is valid from pt_create above.
        unsafe { pt_destroy(handle) };
    }

    #[test]
    fn create_with_invalid_json_returns_null() {
        let json = CString::new("not valid json {{{").unwrap();
        // SAFETY: json is a valid NUL-terminated C string (content is invalid JSON).
        let handle = unsafe { pt_create(json.as_ptr()) };
        assert!(handle.is_null());
    }

    #[test]
    fn start_null_handle_returns_error() {
        // SAFETY: testing NULL handle path.
        assert_eq!(unsafe { pt_start(std::ptr::null_mut()) }, -1);
    }

    #[test]
    fn pause_null_handle_returns_error() {
        // SAFETY: testing NULL handle path.
        assert_eq!(unsafe { pt_pause(std::ptr::null_mut()) }, -1);
    }

    #[test]
    fn resume_null_handle_returns_error() {
        // SAFETY: testing NULL handle path.
        assert_eq!(unsafe { pt_resume(std::ptr::null_mut()) }, -1);
    }

    #[test]
    fn render_svg_null_handle_returns_null() {
        let bid = CString::new("test").unwrap();
        let sname = CString::new("test").unwrap();
        // SAFETY: testing NULL handle path.
        let result = unsafe { pt_render_svg(std::ptr::null_mut(), bid.as_ptr(), sname.as_ptr()) };
        assert!(result.is_null());
    }

    #[test]
    fn render_svg_null_args_returns_null() {
        // SAFETY: NULL config → default platform.
        let handle = unsafe { pt_create(std::ptr::null()) };
        assert!(!handle.is_null());

        // SAFETY: testing NULL builder_id path with valid handle.
        let sname = CString::new("test").unwrap();
        let r1 = unsafe { pt_render_svg(handle, std::ptr::null(), sname.as_ptr()) };
        assert!(r1.is_null());

        // SAFETY: testing NULL scene_name path with valid handle.
        let bid = CString::new("test").unwrap();
        let r2 = unsafe { pt_render_svg(handle, bid.as_ptr(), std::ptr::null()) };
        assert!(r2.is_null());

        // SAFETY: valid handle from pt_create.
        unsafe { pt_destroy(handle) };
    }

    #[test]
    fn ipc_request_null_handle_returns_null() {
        let json = CString::new("{}").unwrap();
        // SAFETY: testing NULL handle path.
        let result = unsafe { pt_ipc_request(std::ptr::null_mut(), json.as_ptr()) };
        assert!(result.is_null());
    }

    #[test]
    fn ipc_request_null_json_returns_null() {
        // SAFETY: NULL config → default platform.
        let handle = unsafe { pt_create(std::ptr::null()) };
        assert!(!handle.is_null());
        // SAFETY: testing NULL json path with valid handle.
        let result = unsafe { pt_ipc_request(handle, std::ptr::null()) };
        assert!(result.is_null());
        // SAFETY: valid handle from pt_create.
        unsafe { pt_destroy(handle) };
    }

    #[test]
    fn free_string_null_is_noop() {
        // SAFETY: NULL is explicitly allowed and is a no-op.
        unsafe { pt_free_string(std::ptr::null_mut()) };
    }

    #[test]
    fn destroy_null_is_noop() {
        // SAFETY: NULL is explicitly allowed and is a no-op.
        unsafe { pt_destroy(std::ptr::null_mut()) };
    }

    #[test]
    fn full_lifecycle_roundtrip() {
        // SAFETY: NULL config → default platform.
        let handle = unsafe { pt_create(std::ptr::null()) };
        assert!(!handle.is_null(), "create should succeed");

        // SAFETY: valid handle from pt_create.
        let start_result = unsafe { pt_start(handle) };
        assert_eq!(start_result, 0, "start should succeed");

        // SAFETY: valid handle, pause/resume lifecycle.
        assert_eq!(unsafe { pt_pause(handle) }, 0);
        assert_eq!(unsafe { pt_resume(handle) }, 0);

        let req =
            CString::new(r#"{"jsonrpc":"2.0","method":"visualization.list_scenarios","id":1}"#)
                .unwrap();
        // SAFETY: valid handle and valid C string.
        let resp = unsafe { pt_ipc_request(handle, req.as_ptr()) };
        if !resp.is_null() {
            // SAFETY: resp is from pt_ipc_request, valid to read and free.
            let resp_str = unsafe { CStr::from_ptr(resp) }.to_str().unwrap();
            assert!(resp_str.contains("jsonrpc"));
            unsafe { pt_free_string(resp) };
        }

        // SAFETY: valid handle, final cleanup.
        unsafe { pt_destroy(handle) };
    }
}
