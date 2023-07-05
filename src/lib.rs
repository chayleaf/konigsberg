use once_cell::sync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_void, CStr},
    sync::{Mutex, RwLock},
};
use steamworks_sys::{
    AppId_t, CSteamID, EUserHasLicenseForAppResult, HSteamPipe, HSteamUser, ISteamApps,
};

mod ffi;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Interface {
    Apps(u8),
    Client(u8),
    User(u8),
}

fn str_ver(ver: &str) -> Option<u8> {
    if ver.len() < 3 {
        return None;
    }
    ver[ver.len() - 3..].parse::<u8>().ok()
}

unsafe fn parse_ver(ver: *const c_char) -> Option<Interface> {
    let ver = CStr::from_ptr(ver).to_str().ok()?;
    if ver.starts_with("STEAMAPPS_INTERFACE_VERSION") {
        Some(Interface::Apps(str_ver(ver)?))
    } else if ver.starts_with("SteamUser") {
        Some(Interface::User(str_ver(ver)?))
    } else if ver.starts_with("SteamClient") {
        Some(Interface::Client(str_ver(ver)?))
    } else {
        None
    }
}

#[allow(clippy::missing_safety_doc)]
#[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
unsafe extern "C" fn b_is_dlc_installed(_this: *mut c_void, _app_id: AppId_t) -> bool {
    true
}
#[allow(clippy::missing_safety_doc)]
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
unsafe extern "fastcall" fn b_is_dlc_installed(
    _this: *mut c_void,
    _edx: usize,
    _app_id: AppId_t,
) -> bool {
    true
}
#[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
#[allow(clippy::missing_safety_doc, improper_ctypes_definitions)]
unsafe extern "C" fn user_has_license_for_app(
    _this: *mut c_void,
    _steam_id: CSteamID,
    _app_id: AppId_t,
) -> EUserHasLicenseForAppResult {
    EUserHasLicenseForAppResult::k_EUserHasLicenseResultHasLicense
}
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
#[allow(clippy::missing_safety_doc, improper_ctypes_definitions)]
unsafe extern "fastcall" fn user_has_license_for_app(
    _this: *mut c_void,
    _edx: usize,
    _steam_id: CSteamID,
    _app_id: AppId_t,
) -> EUserHasLicenseForAppResult {
    EUserHasLicenseForAppResult::k_EUserHasLicenseResultHasLicense
}

#[derive(Copy, Clone)]
struct OrigSteamClientFns {
    #[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
    generic: Option<
        unsafe extern "C" fn(*mut c_void, HSteamUser, HSteamPipe, *const c_char) -> *mut c_void,
    >,
    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    generic: Option<
        unsafe extern "fastcall" fn(
            *mut c_void,
            usize,
            HSteamUser,
            HSteamPipe,
            *const c_char,
        ) -> *mut c_void,
    >,
    #[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
    apps: unsafe extern "C" fn(*mut c_void, HSteamUser, HSteamPipe, *const c_char) -> *mut c_void,
    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    apps: unsafe extern "fastcall" fn(
        *mut c_void,
        usize,
        HSteamUser,
        HSteamPipe,
        *const c_char,
    ) -> *mut c_void,
    #[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
    user: unsafe extern "C" fn(*mut c_void, HSteamUser, HSteamPipe, *const c_char) -> *mut c_void,
    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    user: unsafe extern "fastcall" fn(
        *mut c_void,
        usize,
        HSteamUser,
        HSteamPipe,
        *const c_char,
    ) -> *mut c_void,
}

#[derive(Copy, Clone)]
enum SteamClientFn {
    Generic,
    Apps,
    User,
}

#[allow(clippy::type_complexity)]
static ORIG_CLIENT_FNS: OnceCell<RwLock<HashMap<usize, OrigSteamClientFns>>> = OnceCell::new();

unsafe fn steam_client_common(
    this: *mut c_void,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
    field: SteamClientFn,
) -> *mut c_void {
    patch_ptr(
        parse_ver(ver),
        ORIG_CLIENT_FNS
            .get()
            .and_then(|rwlock| {
                rwlock.read().ok().and_then(|lock| {
                    lock.get(&(*(this as *mut usize)))
                        .and_then(|val| match field {
                            SteamClientFn::Apps => Some(val.apps),
                            SteamClientFn::User => Some(val.user),
                            SteamClientFn::Generic => val.generic,
                        })
                })
            })
            .map(|func| {
                func(
                    this,
                    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                    0,
                    steam_user,
                    steam_pipe,
                    ver,
                )
            })
            .unwrap_or(std::ptr::null_mut()),
    )
}

#[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_i_steam_generic_interface(
    this: *mut c_void,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::Generic)
}
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "fastcall" fn get_i_steam_generic_interface(
    this: *mut c_void,
    _edx: usize,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::Generic)
}
#[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_i_steam_user(
    this: *mut c_void,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::User)
}
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "fastcall" fn get_i_steam_user(
    this: *mut c_void,
    _edx: usize,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::User)
}
#[cfg(any(not(target_os = "windows"), not(target_pointer_width = "32")))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_i_steam_apps(
    this: *mut c_void,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::Apps)
}
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
#[allow(clippy::missing_safety_doc)]
unsafe extern "fastcall" fn get_i_steam_apps(
    this: *mut c_void,
    _edx: usize,
    steam_user: HSteamUser,
    steam_pipe: HSteamPipe,
    ver: *const c_char,
) -> *mut c_void {
    steam_client_common(this, steam_user, steam_pipe, ver, SteamClientFn::Apps)
}

unsafe fn patch2(object: *mut c_void, offsets: &[(usize, *mut c_void)], pre_hook: impl FnOnce()) {
    static PATCH_DONE: OnceCell<Mutex<HashSet<usize>>> = OnceCell::new();

    if offsets.is_empty() {
        return;
    }

    let patch_done = PATCH_DONE.get_or_init(Default::default);
    let vtable = *(object as *mut *mut usize);
    let mut lock = patch_done.lock().unwrap();
    if lock.contains(&(vtable as usize)) {
        return;
    }

    pre_hook();

    for (offset, func) in offsets.iter().copied() {
        let p_func = vtable.add(offset) as *mut *mut c_void;
        let _handle = region::protect_with_handle(
            p_func,
            std::mem::size_of::<usize>(),
            region::Protection::READ_WRITE_EXECUTE,
        )
        .expect("mprotect failed");
        *p_func = func;
    }

    lock.insert(vtable as usize);
}
unsafe fn patch(object: *mut c_void, offsets: &[(usize, *mut c_void)]) {
    patch2(object, offsets, || {});
}

unsafe fn patch_ptr(ver: Option<Interface>, ret: *mut c_void) -> *mut c_void {
    if !ret.is_null() {
        if let Some(ver) = ver {
            match ver {
                Interface::Apps(..=1) => {}
                Interface::Apps(n @ 2..) => {
                    let mut patches = vec![
                        // this is issubscribedapp, same sig as isdlcinstalled
                        (
                            6,
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                b_is_dlc_installed
                                    as unsafe extern "C" fn(*mut c_void, AppId_t) -> bool,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                b_is_dlc_installed
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        AppId_t,
                                    )
                                        -> bool,
                            )),
                        ),
                    ];
                    if n >= 3 {
                        patches.push((
                            7,
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                b_is_dlc_installed
                                    as unsafe extern "C" fn(*mut c_void, AppId_t) -> bool,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                b_is_dlc_installed
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        AppId_t,
                                    )
                                        -> bool,
                            )),
                        ));
                    }
                    patch(ret, &patches);
                }
                Interface::User(..=11) => {}
                Interface::User(n) => {
                    patch(
                        ret,
                        &[(
                            match n {
                                ..=12 => 15,
                                13..=14 => 16,
                                15.. => 17,
                            },
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                user_has_license_for_app
                                    as unsafe extern "C" fn(
                                        *mut c_void,
                                        CSteamID,
                                        AppId_t,
                                    )
                                        -> EUserHasLicenseForAppResult,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                user_has_license_for_app
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        CSteamID,
                                        AppId_t,
                                    )
                                        -> EUserHasLicenseForAppResult,
                            )),
                        )],
                    );
                }
                Interface::Client(..=5) => {}
                Interface::Client(n) => {
                    let offset_apps = match n {
                        ..=6 => 16,
                        7 => 18,
                        8 => 15,
                        9..=11 => 16,
                        12.. => 15,
                    };
                    let offset_user = match n {
                        ..=6 => 6,
                        7.. => 5,
                    };
                    let offset_generic = match n {
                        ..=6 => None,
                        7 => Some(14),
                        8..=11 => Some(13),
                        12.. => Some(12),
                    };

                    let mut patches = vec![
                        (
                            offset_apps,
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                get_i_steam_apps
                                    as unsafe extern "C" fn(
                                        *mut c_void,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                get_i_steam_apps
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                        ),
                        (
                            offset_user,
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                get_i_steam_user
                                    as unsafe extern "C" fn(
                                        *mut c_void,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                get_i_steam_user
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                        ),
                    ];
                    if let Some(offset_generic) = offset_generic {
                        patches.push((
                            offset_generic,
                            #[cfg(any(
                                not(target_os = "windows"),
                                not(target_pointer_width = "32")
                            ))]
                            std::mem::transmute(Some(
                                get_i_steam_generic_interface
                                    as unsafe extern "C" fn(
                                        *mut c_void,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                            std::mem::transmute(Some(
                                get_i_steam_generic_interface
                                    as unsafe extern "fastcall" fn(
                                        *mut c_void,
                                        usize,
                                        HSteamUser,
                                        HSteamPipe,
                                        *const c_char,
                                    )
                                        -> *mut c_void,
                            )),
                        ));
                    }
                    patch2(ret, &patches, || {
                        let vtable = *(ret as *mut *mut usize);
                        let mut lock = ORIG_CLIENT_FNS
                            .get_or_init(Default::default)
                            .write()
                            .unwrap();
                        let get = |ofs| vtable.add(ofs);
                        lock.insert(
                            vtable as usize,
                            OrigSteamClientFns {
                                generic: offset_generic
                                    .map(get)
                                    .map(|x| std::mem::transmute::<_, Option<_>>(x).unwrap()),
                                apps: std::mem::transmute::<_, Option<_>>(get(offset_apps))
                                    .unwrap(),
                                user: std::mem::transmute::<_, Option<_>>(get(offset_user))
                                    .unwrap(),
                            },
                        );
                    });
                }
            }
        }
    }
    ret
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn SteamInternal_FindOrCreateUserInterface(
    user: HSteamUser,
    ver: *const c_char,
) -> *mut c_void {
    patch_ptr(
        parse_ver(ver),
        ffi::SteamInternal_FindOrCreateUserInterface(user, ver),
    )
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn SteamAPI_SteamApps_v008() -> *mut ISteamApps {
    patch_ptr(
        Some(Interface::Apps(8)),
        ffi::SteamAPI_SteamApps_v008() as *mut c_void,
    ) as *mut ISteamApps
}
#[allow(non_snake_case, clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn SteamAPI_SteamApps_v009() -> *mut ISteamApps {
    patch_ptr(
        Some(Interface::Apps(9)),
        ffi::SteamAPI_SteamApps_v009() as *mut c_void,
    ) as *mut ISteamApps
}
