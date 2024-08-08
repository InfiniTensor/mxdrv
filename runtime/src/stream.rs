use crate::{bindings::mcStream_t, CurrentCtx};
use context_spore::{impl_spore, AsRaw};
use std::{marker::PhantomData, ptr::null_mut};

impl_spore!(Stream and StreamSpore by (CurrentCtx, mcStream_t));

impl CurrentCtx {
    #[inline]
    pub fn stream(&self) -> Stream {
        let mut stream = null_mut();
        mxdrv!(mcStreamCreate(&mut stream));
        Stream(unsafe { self.wrap_raw(stream) }, PhantomData)
    }
}

impl Drop for Stream<'_> {
    #[inline]
    fn drop(&mut self) {
        self.synchronize();
        mxdrv!(mcStreamDestroy(self.0.rss));
    }
}

impl AsRaw for Stream<'_> {
    type Raw = mcStream_t;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss
    }
}

impl Stream<'_> {
    #[inline]
    pub fn synchronize(&self) {
        mxdrv!(mcStreamSynchronize(self.0.rss));
    }
}