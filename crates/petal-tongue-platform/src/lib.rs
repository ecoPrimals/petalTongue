// SPDX-License-Identifier: AGPL-3.0-or-later
#![deny(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Platform embedding layer for petalTongue.
//!
//! This crate provides the **embedding contract** that enables petalTongue to be
//! loaded as a dynamic library (`.so`, `.dylib`) by host applications on Android,
//! iOS, and future targets. The host drives the OS lifecycle; petalTongue owns the
//! rendering pipeline and transport layer.
//!
//! # Architecture
//!
//! ```text
//! Host App (Kotlin/Swift/C#)
//!   │
//!   ├─ Android Activity ──► JNI ──┐
//!   ├─ iOS AppDelegate  ──► @_cdecl ──┤
//!   └─ Desktop main()   ──► direct ──┤
//!                                     ▼
//!                          ┌─────────────────────┐
//!                          │ petal-tongue-platform│
//!                          │  PlatformLifecycle   │
//!                          │  EmbeddedRuntime     │
//!                          │  C-FFI surface       │
//!                          └─────────┬───────────┘
//!                                    │
//!                    ┌───────────────┼───────────────┐
//!                    ▼               ▼               ▼
//!              petal-tongue-core  petal-tongue-scene  petal-tongue-ui-core
//! ```
//!
//! # Usage from Rust (Desktop embedding)
//!
//! ```no_run
//! use petal_tongue_platform::{EmbedConfig, EmbeddedRuntime, Platform};
//!
//! let config = EmbedConfig::new(Platform::Desktop);
//! let mut runtime = EmbeddedRuntime::new(config).unwrap();
//! runtime.start().unwrap();
//!
//! let svg = runtime.render_svg("airspring.et0", "daily_et0").unwrap();
//! println!("{svg}");
//!
//! runtime.stop().unwrap();
//! ```
//!
//! # Usage from C / JNI / Swift (via FFI)
//!
//! ```c
//! PetalTongueHandle* h = pt_create("{\"platform\":\"android\"}");
//! pt_start(h);
//! char* svg = pt_render_svg(h, "airspring.et0", "daily_et0");
//! // ... use svg ...
//! pt_free_string(svg);
//! pt_destroy(h);
//! ```

pub mod config;
pub mod lifecycle;
pub mod runtime;

#[allow(unsafe_code)]
pub mod ffi;

pub use config::{EmbedConfig, Platform, PlatformConfig};
pub use lifecycle::{PlatformEvent, PlatformLifecycle};
pub use runtime::EmbeddedRuntime;
