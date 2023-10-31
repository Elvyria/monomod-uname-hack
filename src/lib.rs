use std::ptr::null_mut;

use frida_gum::{interceptor::Interceptor, Module, NativePointer, Gum};
use libc::{c_void, c_int};
use lazy_static::lazy_static;

lazy_static! { static ref GUM: Gum = unsafe { Gum::obtain() }; }

unsafe extern "C" fn uname(buf: *mut libc::utsname) -> c_int {
    let result = libc::uname(buf);

    static ARCH: &[i8; 5] = unsafe { std::mem::transmute(b"AMD64") };
    (*buf).version[..5].copy_from_slice(ARCH);

    result
}

#[ctor::ctor]
fn init() {
    let mut interceptor = Interceptor::obtain(&GUM);
    let native = Module::find_export_by_name(None, "uname").unwrap();
    interceptor.replace(native, NativePointer(uname as *mut c_void), NativePointer(null_mut())).unwrap();
}
