#![allow(
    non_snake_case,
    clippy::missing_safety_doc,
    improper_ctypes_definitions
)]

use once_cell::sync::OnceCell;
use std::os::raw::{c_char, c_void};
use steamworks_sys::*;

macro_rules! reexport {
    (fn $name:ident($( $arg:ident : $type:ty ),*) $(-> $ret:ty)?) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name($( $arg : $type),*) $(-> $ret)? {
            static CELL: OnceCell<libloading::Symbol<unsafe extern "C" fn($($type),*) $(-> $ret)?>> = OnceCell::new();
            let sym = CELL.get_or_init(|| {
                sym(stringify!($name)).expect(&format!("failed to load symbol: {}", stringify!($name)))
            });
            sym($( $arg ),*)
        }
    };
    ($link:literal, fn $name:ident($( $arg:ident : $type:ty ),*) $(-> $ret:ty)?) => {
        #[export_name = $link]
        #[no_mangle]
        pub unsafe extern "C" fn $name($( $arg : $type),*) $(-> $ret)? {
            static CELL: OnceCell<libloading::Symbol<unsafe extern "C" fn($($type),*) $(-> $ret)?>> = OnceCell::new();
            let sym = CELL.get_or_init(|| {
                sym($link)
                    .or_else(|_| sym(&$link[1..]))
                    .or_else(|_| sym(stringify!($name)))
                    .or_else(|_| sym(Box::leak(Box::new("\x01".to_owned() + $link)).as_str()))
                    .expect(&format!("failed to load symbol: {}", $link))
            });
            sym($( $arg ),*)
        }
    };
}
macro_rules! import {
    (fn $name:ident($( $arg:ident : $type:ty ),*) $(-> $ret:ty)?) => {
        pub(crate) unsafe extern "C" fn $name($( $arg : $type),*) $(-> $ret)? {
            static CELL: OnceCell<libloading::Symbol<unsafe extern "C" fn($($type),*) $(-> $ret)?>> = OnceCell::new();
            let sym = CELL.get_or_init(|| {
                sym(stringify!($name)).expect(&format!("failed to load symbol: {}", stringify!($name)))
            });
            sym($( $arg ),*)
        }
    };
}

unsafe fn lib() -> &'static libloading::Library {
    static CELL: OnceCell<libloading::Library> = OnceCell::new();
    CELL.get_or_init(|| {
        unsafe {
            #[cfg(target_os = "macos")]
            {
                libloading::Library::new("./libsteam_api.orig.dylib")
                    .or_else(|_| libloading::Library::new("./libsteam_api_orig.dylib"))
                    .or_else(|_| libloading::Library::new("libsteam_api.orig.dylib"))
                    .or_else(|_| libloading::Library::new("libsteam_api_orig.dylib"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.dylib"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.dylib"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
            #[cfg(all(target_os = "windows", not(target_pointer_width = "64")))]
            {
                libloading::Library::new("./steam_api.orig.dll")
                    .or_else(|_| libloading::Library::new("./steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
            #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
            {
                libloading::Library::new("./steam_api64.orig.dll")
                    .or_else(|_| libloading::Library::new("./steam_api64_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api64.orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api64_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api64.orig"))
                    .or_else(|_| libloading::Library::new("steam_api64_orig"))
                    .or_else(|_| libloading::Library::new("./steam_api.orig.dll"))
                    .or_else(|_| libloading::Library::new("./steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.dll"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
            #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
            {
                libloading::Library::new("./libsteam_api.orig.so")
                    .or_else(|_| libloading::Library::new("./libsteam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("libsteam_api.orig.so"))
                    .or_else(|_| libloading::Library::new("libsteam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api.orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api_orig.so"))
                    .or_else(|_| libloading::Library::new("steam_api.orig"))
                    .or_else(|_| libloading::Library::new("steam_api_orig"))
            }
        }
        .expect("failed to load steam api lib")
    })
}

unsafe fn sym<T>(name: &str) -> Result<libloading::Symbol<T>, libloading::Error> {
    lib().get(name.as_bytes())
}

import!(fn SteamInternal_FindOrCreateUserInterface(hSteamUser: HSteamUser, pszVersion: *const c_char) -> *mut c_void);
import!(fn SteamAPI_SteamApps_v008() -> *mut ISteamApps);
// future proof?
import!(fn SteamAPI_SteamApps_v009() -> *mut ISteamApps);

#[cfg(feature = "rebuild-reexports")]
include!(concat!(env!("OUT_DIR"), "/reexports.rs"));

#[cfg(not(feature = "rebuild-reexports"))]
include!("reexports.rs");
