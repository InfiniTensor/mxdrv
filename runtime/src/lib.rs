#![cfg(detected_mx)]

#[macro_use]
#[allow(unused, non_upper_case_globals, non_camel_case_types, non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[macro_export]
    macro_rules! mxdrv {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let error = unsafe { $f };
            assert_eq!(error, mcError_t::mcSuccess);
        }};
    }
}

mod context;
mod device;
mod event;
mod memory;
mod stream;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NoDevice;

pub use context::{Context, CurrentCtx};
pub use context_spore::{impl_spore, AsRaw, ContextResource, ContextSpore, RawContainer};
pub use device::{BlockLimit, Device, SMLimit};
pub use event::{Event, EventSpore};
pub use memory::{
    memcpy_d2d, memcpy_d2h, memcpy_h2d, DevByte, DevMem, DevMemSpore, HostMem, HostMemSpore,
};
pub use stream::{Stream, StreamSpore};

use std::{
    cmp::Ordering,
    ffi::{c_int, c_uint},
    fmt,
};

struct Blob<P> {
    ptr: P,
    len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Dim3 {
    pub x: c_uint,
    pub y: c_uint,
    pub z: c_uint,
}

impl From<()> for Dim3 {
    #[inline]
    fn from(_: ()) -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}

impl From<c_uint> for Dim3 {
    #[inline]
    fn from(x: c_uint) -> Self {
        Self { x, y: 1, z: 1 }
    }
}

impl From<(c_uint, c_uint)> for Dim3 {
    #[inline]
    fn from((y, x): (c_uint, c_uint)) -> Self {
        Self { x, y, z: 1 }
    }
}

impl From<(c_uint, c_uint, c_uint)> for Dim3 {
    #[inline]
    fn from((z, y, x): (c_uint, c_uint, c_uint)) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
}

impl Version {
    #[inline]
    pub fn to_arch_string(&self) -> String {
        format!("{}{}", self.major, self.minor)
    }
}

impl PartialOrd for Version {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&self.major) {
            Ordering::Equal => self.minor.cmp(&other.minor),
            other => other,
        }
    }
}

impl fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MemSize(pub usize);

impl fmt::Display for MemSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "0")
        } else {
            let zeros = self.0.trailing_zeros();
            if zeros >= 40 {
                write!(f, "{}TiB", self.0 >> 40)
            } else if zeros >= 30 {
                write!(f, "{}GiB", self.0 >> 30)
            } else if zeros >= 20 {
                write!(f, "{}MiB", self.0 >> 20)
            } else if zeros >= 10 {
                write!(f, "{}KiB", self.0 >> 10)
            } else {
                write!(f, "{}B", self.0)
            }
        }
    }
}

impl From<c_int> for MemSize {
    #[inline]
    fn from(value: c_int) -> Self {
        Self(value as _)
    }
}

impl From<usize> for MemSize {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[allow(unused_variables, non_snake_case)]
#[inline(always)]
pub fn init() -> Result<(), NoDevice> {
    use bindings::{mcInit, mcError_t::*};
    match unsafe { mcInit(0) } {
        mcSuccess => Ok(()),
        mcErrorInvalidDevice => Err(NoDevice),
        e => panic!("Failed to initialize MUSA: {e:?}"),
    }
}

pub fn get_device_count() -> i32 {
    let mut a = 0;
    mxdrv!(mcGetDeviceCount(&mut a));
    a
}

#[test]
fn test_binding() {
    let _ = init();
    println!("{}", get_device_count());

}