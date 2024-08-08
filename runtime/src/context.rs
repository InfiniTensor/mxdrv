use crate::{
    bindings::{mcCtx_t, mcDevice_t, MCcontext},
    Device,
};
use context_spore::{AsRaw, RawContainer};
use std::{
    mem::{align_of, size_of},
    ptr::null_mut,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Context {
    ctx: mcCtx_t,
    dev: mcDevice_t,
    primary: bool,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        const { assert!(size_of::<Context>() == size_of::<[usize; 2]>()) }
        const { assert!(align_of::<Context>() == align_of::<usize>()) }

        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        mxdrv!(mcCtxCreate(&mut ctx, 0, dev));
        mxdrv!(mcCtxPopCurrent(null_mut()));
        Context {
            ctx,
            dev,
            primary: false,
        }
    }

    #[inline]
    pub fn retain_primary(&self) -> Context {
        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        mxdrv!(mcDevicePrimaryCtxRetain(&mut ctx, dev));
        Context {
            ctx,
            dev,
            primary: true,
        }
    }
}

impl Drop for Context {
    #[inline]
    fn drop(&mut self) {
        if self.primary {
            // mcDevicePrimaryCtxRelease 这个函数api中没有，但是有记录
            mxdrv!(mcDevicePrimaryCtxReset(self.dev));
        } else {
            mxdrv!(mcCtxDestroy(self.ctx))
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl AsRaw for Context {
    type Raw = MCcontext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.ctx
    }
}

impl Context {
    #[inline]
    pub fn device(&self) -> Device {
        Device::new(self.dev)
    }

    #[inline]
    pub fn apply<T>(&self, f: impl FnOnce(&CurrentCtx) -> T) -> T {
        mxdrv!(mcCtxPushCurrent(self.ctx));
        let ans = f(&CurrentCtx(self.ctx));
        let mut top = null_mut();
        mxdrv!(mcCtxPopCurrent(&mut top));
        assert_eq!(top, self.ctx);
        ans
    }
}

#[repr(transparent)]
pub struct CurrentCtx(MCcontext);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NoCtxError;

impl AsRaw for CurrentCtx {
    type Raw = MCcontext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl CurrentCtx {
    #[inline]
    pub fn dev(&self) -> Device {
        let mut dev = 0;
        mxdrv!(mcCtxGetDevice(&mut dev));
        Device::new(dev)
    }

    #[inline]
    pub fn synchronize(&self) {
        mxdrv!(mcCtxSynchronize());
    }

    #[inline]
    pub fn apply_current<T>(f: impl FnOnce(&Self) -> T) -> Result<T, NoCtxError> {
        let mut raw = null_mut();
        mxdrv!(mcCtxGetCurrent(&mut raw));
        if !raw.is_null() {
            Ok(f(&Self(raw)))
        } else {
            Err(NoCtxError)
        }
    }

    /// 直接指定当前上下文，并执行依赖上下文的操作。
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context.
    #[inline]
    pub unsafe fn apply_current_unchecked<T>(raw: MCcontext, f: impl FnOnce(&Self) -> T) -> T {
        f(&Self(raw))
    }

    /// Designates `raw` as the current context.
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context.
    /// Generally, this method only used for [`RawContainer::ctx`] with limited lifetime.
    #[inline]
    pub unsafe fn from_raw(raw: &MCcontext) -> &Self {
        &*(raw as *const _ as *const _)
    }

    /// Wrap a raw object in a `RawContainer`.
    ///
    /// # Safety
    ///
    /// The raw object must be created in this [`Context`].
    #[inline]
    pub unsafe fn wrap_raw<T: Unpin + 'static>(&self, rss: T) -> RawContainer<MCcontext, T> {
        RawContainer { ctx: self.0, rss }
    }
}

impl CurrentCtx {
    pub fn lock_page<T>(&self, slice: &[T]) {
        let ptrs = slice.as_ptr_range();
        mxdrv!(mcHostRegister(
            ptrs.start as _,
            ptrs.end as usize - ptrs.start as usize,
            0,
        ));
    }

    pub fn unlock_page<T>(&self, slice: &[T]) {
        mxdrv!(mcHostUnregister(slice.as_ptr() as _));
    }
}

#[test]
fn test_primary() {
    if let Err(crate::NoDevice) = crate::init() {
        return;
    }
    let dev = crate::Device::new(0);
    let mut flags = 0;
    let mut active = 0;
    mxdrv!(mcDevicePrimaryCtxGetState(
        dev.as_raw(),
        &mut flags,
        &mut active
    ));
    assert_eq!(flags, 0);
    assert_eq!(active, 0);

    let mut pctx = null_mut();
    mxdrv!(mcDevicePrimaryCtxRetain(&mut pctx, dev.as_raw()));
    assert!(!pctx.is_null());

    mxdrv!(mcDevicePrimaryCtxGetState(
        dev.as_raw(),
        &mut flags,
        &mut active
    ));
    assert_eq!(flags, 0);
    assert_ne!(active, 0);

    mxdrv!(mcCtxGetCurrent(&mut pctx));
    assert!(pctx.is_null());
}