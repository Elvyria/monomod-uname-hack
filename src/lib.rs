#![no_main]
#![no_std]

use core::{panic::PanicInfo, sync::atomic::{AtomicU8, Ordering::Relaxed}};

use libc_alloc::LibcAlloc;
use frida_gum::{interceptor::Interceptor, Module, NativePointer, Gum};

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;
static mut LIBC_UNAME: UnameSign = uname;

lazy_static::lazy_static! {
    static ref COUNTER: AtomicU8 = AtomicU8::new(0);
    static ref GUM: Gum = unsafe { Gum::obtain() };
}

type UnameSign = unsafe extern "C" fn(buf: *mut libc::utsname) -> libc::c_int;

unsafe extern "C" fn uname(buf: *mut libc::utsname) -> libc::c_int {
    let result = LIBC_UNAME(buf);

    static ARCH: &[i8; 5] = unsafe { core::mem::transmute(b"AMD64") };
    (*buf).version[..5].copy_from_slice(ARCH);

    if COUNTER.fetch_add(1, Relaxed) > 0 {
        let mut interceptor = Interceptor::obtain(&GUM);
        interceptor.revert(NativePointer(LIBC_UNAME as *mut libc::c_void));
    }

    result
}

#[ctor::ctor]
fn init() {
    let mut interceptor = Interceptor::obtain(&GUM);
    let native = Module::find_export_by_name(None, "uname").unwrap();

    unsafe {
        LIBC_UNAME = core::mem::transmute(interceptor.replace_fast(native, NativePointer(uname as *mut libc::c_void)).unwrap());
    }
}

#[panic_handler]
unsafe fn panic(_panic: &PanicInfo<'_>) -> ! { libc::abort() }
