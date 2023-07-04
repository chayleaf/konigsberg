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
                    .or_else(|_| sym(&$link[2..]))
                    .or_else(|_| sym(stringify!($name)))
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
            }
            #[cfg(target_os = "windows")]
            {
                libloading::Library::new("./steam_api.orig.dll")
            }
            #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
            {
                libloading::Library::new("./libsteam_api.orig.so")
            }
        }
        .expect("failed to load steam api lib")
    })
}

unsafe fn sym<T>(name: &str) -> Result<libloading::Symbol<T>, libloading::Error> {
    lib().get(name.as_bytes())
}

import!(fn SteamInternal_FindOrCreateUserInterface(hSteamUser: HSteamUser, pszVersion: *const c_char) -> *mut c_void);

#[cfg(feature = "rebuild-reexports")]
include!(concat!(env!("OUT_DIR"), "/reexports.rs"));

#[cfg(not(feature = "rebuild-reexports"))]
include!("reexports.rs");

/*reexport!(fn SteamAPI_Init() -> bool);
reexport!(fn SteamAPI_Shutdown());
reexport!(fn SteamAPI_IsSteamRunning() -> bool);
reexport!(fn SteamAPI_GetHSteamUser() -> HSteamUser);
reexport!(fn SteamAPI_RunCallbacks());
reexport!(fn SteamAPI_RegisterCallback(pCallback: *mut CCallbackBase, iCallback: std::os::raw::c_int));
reexport!(fn SteamAPI_UnregisterCallback(pCallback: *mut CCallbackBase));
reexport!(fn SteamAPI_RegisterCallResult(pCallback: *mut CCallbackBase, hAPICall: SteamAPICall_t));
reexport!(fn SteamAPI_UnregisterCallResult(pCallback: *mut CCallbackBase, hAPICall: SteamAPICall_t));
reexport!(fn SteamGameServer_Shutdown());
reexport!(fn SteamGameServer_GetHSteamUser() -> HSteamUser);
reexport!(fn SteamGameServer_RunCallbacks());
reexport!(fn SteamInternal_ContextInit(pContextInitData: *mut c_void) -> *mut c_void);
reexport!(fn SteamInternal_FindOrCreateGameServerInterface(hSteamUser: HSteamUser, pszVersion: *const c_char) -> *mut c_void);
reexport!(fn SteamInternal_GameServer_Init(
    unIP: uint32,
    usLegacySteamPort: uint16,
    usGamePort: uint16,
    usQueryPort: uint16,
    eServerMode: EServerMode,
    pchVersionString: *const c_char
) -> bool);*/
