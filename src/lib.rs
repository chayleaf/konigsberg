use once_cell::sync::OnceCell;
use std::{
    collections::HashSet,
    ffi::{c_char, c_void, CStr},
    sync::Mutex,
};
use steamworks_sys::{AppId_t, HSteamUser};

mod ffi;

/*
pub struct Context {
    func: Option<unsafe fn(*mut c_void)>,
    counter: usize,
    ptr: *mut c_void,
}
#[allow(non_snake_case, clippy::missing_safety_doc)]
pub unsafe extern "C" fn SteamInternal_ContextInit(
    _ctx: *mut Context,
) -> *mut c_void {
    std::ptr::null_mut()
}*/

struct Offsets {
    offset_b_is_dlc_installed: usize,
    // offset_get_dlc_count: usize,
    // offset_b_get_dlc_data_by_index: Option<usize>,
}

unsafe fn offsets(ver: *const c_char) -> Option<Offsets> {
    let ver = CStr::from_ptr(ver).to_str().ok()?;
    if !ver.starts_with("STEAMAPPS_INTERFACE_VERSION")
        || ver.ends_with("001")
        || ver.ends_with("002")
    {
        return None;
    }
    Some(Offsets {
        offset_b_is_dlc_installed: 7,
        // offset_b_get_dlc_data_by_index: ()
    })
}

// on unix, thiscall doesn't exist so this is fine
// on windows, it isn't
#[allow(clippy::missing_safety_doc)]
#[cfg(not(target_os = "windows"))]
unsafe extern "C" fn b_is_dlc_installed(
    // rdi
    _this: *mut c_void,
    // rsi
    _app_id: AppId_t,
) -> bool {
    true
}

// thiscall...
#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "windows")]
unsafe extern "C" fn b_is_dlc_installed(_app_id: AppId_t) -> bool {
    true
}

unsafe fn patch_ptr(ver: *const c_char, ret: *mut c_void) -> *mut c_void {
    static PATCH_DONE: OnceCell<Mutex<HashSet<usize>>> = OnceCell::new();
    if !ret.is_null() {
        if let Some(ofs) = offsets(ver) {
            let done = PATCH_DONE.get_or_init(Default::default);
            let vtable = *(ret as *mut *mut usize);
            let mut lock = done.lock().unwrap();
            if lock.contains(&(vtable as usize)) {
                return ret;
            }
            let p_b_is_dlc_installed = vtable.add(ofs.offset_b_is_dlc_installed);
            #[cfg(target_os = "windows")]
            let p_b_is_dlc_installed =
                p_b_is_dlc_installed as *mut Option<unsafe extern "C" fn(AppId_t) -> bool>;
            #[cfg(not(target_os = "windows"))]
            let p_b_is_dlc_installed = p_b_is_dlc_installed
                as *mut Option<unsafe extern "C" fn(*mut c_void, AppId_t) -> bool>;
            let _handle = region::protect_with_handle(
                p_b_is_dlc_installed,
                std::mem::size_of::<usize>(),
                region::Protection::READ_WRITE_EXECUTE,
            )
            .expect("mprotect failed");
            *p_b_is_dlc_installed = Some(b_is_dlc_installed);
            lock.insert(vtable as usize);
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
    patch_ptr(ver, ffi::SteamInternal_FindOrCreateUserInterface(user, ver))
}
