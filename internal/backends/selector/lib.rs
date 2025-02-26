// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-1.1 OR LicenseRef-Slint-commercial

#![doc = include_str!("README.md")]
#![doc(html_logo_url = "https://slint.dev/logo/slint-logo-square-light.svg")]
#![cfg_attr(
    not(any(
        feature = "i-slint-backend-qt",
        feature = "i-slint-backend-winit",
        feature = "i-slint-backend-linuxkms"
    )),
    no_std
)]

extern crate alloc;

use alloc::boxed::Box;
use i_slint_core::platform::Platform;
use i_slint_core::platform::PlatformError;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "i-slint-backend-qt", not(no_qt)))] {
        use i_slint_backend_qt as default_backend;

        fn create_default_backend() -> Result<Box<dyn Platform + 'static>, PlatformError> {
            Ok(Box::new(default_backend::Backend::new()))
        }
    } else if #[cfg(feature = "i-slint-backend-winit")] {
        use i_slint_backend_winit as default_backend;
        fn create_default_backend() -> Result<Box<dyn Platform + 'static>, PlatformError> {
            Ok(Box::new(i_slint_backend_winit::Backend::new()))
        }
    } else if #[cfg(feature = "i-slint-backend-linuxkms")] {
        use i_slint_backend_linuxkms as default_backend;
        fn create_default_backend() -> Result<Box<dyn Platform + 'static>, PlatformError> {
            Ok(Box::new(i_slint_backend_linuxkms::Backend::new()?))
        }
    } else {

    }
}

cfg_if::cfg_if! {
    if #[cfg(any(
            all(feature = "i-slint-backend-qt", not(no_qt)),
            feature = "i-slint-backend-winit",
            feature = "i-slint-backend-linuxkms"
        ))] {
        pub fn create_backend() -> Result<Box<dyn Platform + 'static>, PlatformError>  {

            let backend_config = std::env::var("SLINT_BACKEND").unwrap_or_default();

            let backend_config = backend_config.to_lowercase();
            let (event_loop, _renderer) = backend_config.split_once('-').unwrap_or(match backend_config.as_str() {
                "qt" => ("qt", ""),
                "gl" | "winit" => ("winit", ""),
                "femtovg" => ("winit", "femtovg"),
                "skia" => ("winit", "skia"),
                "sw" | "software" => ("winit", "software"),
                "linuxkms" => ("linuxkms", ""),
                x => (x, ""),
            });

            match event_loop {
                #[cfg(all(feature = "i-slint-backend-qt", not(no_qt)))]
                "qt" => return Ok(Box::new(i_slint_backend_qt::Backend::new())),
                #[cfg(feature = "i-slint-backend-winit")]
                "winit" => return Ok(Box::new(i_slint_backend_winit::Backend::new_with_renderer_by_name((!_renderer.is_empty()).then_some(_renderer)))),
                #[cfg(feature = "i-slint-backend-linuxkms")]
                "linuxkms" => return Ok(Box::new(i_slint_backend_linuxkms::Backend::new_with_renderer_by_name((!_renderer.is_empty()).then(|| _renderer))?)),
                _ => {},
            }

            if !backend_config.is_empty() {
                eprintln!("Could not load rendering backend {}, fallback to default", backend_config)
            }
            create_default_backend()
        }
        pub use default_backend::{
            native_widgets, Backend, NativeGlobals, NativeWidgets, HAS_NATIVE_STYLE,
        };
    } else {
        pub fn create_backend() -> Result<Box<dyn Platform + 'static>, PlatformError> {
            Err(PlatformError::NoPlatform)
        }
        pub mod native_widgets {}
        pub type NativeWidgets = ();
        pub type NativeGlobals = ();
        pub const HAS_NATIVE_STYLE: bool = false;
    }
}

/// Run the callback with the platform abstraction.
/// Create the backend if it does not exist yet
pub fn with_platform<R>(
    f: impl FnOnce(&dyn Platform) -> Result<R, PlatformError>,
) -> Result<R, PlatformError> {
    i_slint_core::with_platform(create_backend, f)
}
