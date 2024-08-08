use crate::{
    bindings::{
        mcDeviceAttribute_t::{self, *},
        mcDevice_t,
    },
    Dim3, MemSize, Version,
};
use context_spore::AsRaw;
use std::{ffi::c_int, fmt};

#[repr(transparent)]
pub struct Device(mcDevice_t);

impl AsRaw for Device {
    type Raw = mcDevice_t;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Device {
    #[inline]
    pub fn new(index: c_int) -> Self {
        let mut device = 0;
        mxdrv!(mcDeviceGet(&mut device, index));
        Self(device)
    }

    #[inline]
    pub fn count() -> usize {
        let mut count = 0;
        mxdrv!(mcGetDeviceCount(&mut count));
        count as _
    }

    pub fn name(&self) -> String {
        let mut name = [0u8; 256];
        mxdrv!(mcDeviceGetName(
            name.as_mut_ptr().cast(),
            name.len() as _,
            self.0
        ));
        String::from_utf8(name.iter().take_while(|&&c| c != 0).copied().collect()).unwrap()
    }

    #[inline]
    pub fn compute_capability(&self) -> Version {
        Version {
            major: self.get_attribute(mcDeviceAttributeComputeCapabilityMajor),
            minor: self.get_attribute(mcDeviceAttributeComputeCapabilityMinor),
        }
    }

    #[inline]
    pub fn total_memory(&self) -> MemSize {
        let mut bytes = 0;
        mxdrv!(mcDeviceTotalMem(&mut bytes, self.0));
        bytes.into()
    }

    #[inline]
    pub fn alignment(&self) -> usize {
        self.get_attribute(mcDeviceAttributeTextureAlignment) as _
    }

    #[inline]
    pub fn warp_size(&self) -> usize {
        self.get_attribute(mcDeviceAttributeWarpSize) as _
    }

    #[inline]
    pub fn sm_count(&self) -> usize {
        self.get_attribute(mcDeviceAttributeMultiProcessorCount) as _
    }

    pub fn max_grid_dims(&self) -> Dim3 {
        Dim3 {
            x: self.get_attribute(mcDeviceAttributeMaxGridDimX) as _,
            y: self.get_attribute(mcDeviceAttributeMaxGridDimY) as _,
            z: self.get_attribute(mcDeviceAttributeMaxGridDimZ) as _,
        }
    }

    pub fn block_limit(&self) -> BlockLimit {
        BlockLimit {
            max_threads: self.get_attribute(mcDeviceAttributeMaxThreadsPerBlock) as _,
            max_dims: Dim3 {
                x: self.get_attribute(mcDeviceAttributeMaxBlockDimX) as _,
                y: self.get_attribute(mcDeviceAttributeMaxBlockDimY) as _,
                z: self.get_attribute(mcDeviceAttributeMaxBlockDimZ) as _,
            },
            max_smem: self
                .get_attribute(mcDeviceAttributeMaxSharedMemoryPerBlock)
                .into(),
            max_registers: self
                .get_attribute(mcDeviceAttributeMaxRegistersPerBlock)
                .into(),
        }
    }

    pub fn sm_limit(&self) -> SMLimit {
        SMLimit {
            max_blocks: self.get_attribute(mcDevAttrMaxBlocksPerMultiprocessor) as _,
            max_threads: self.get_attribute(mcDeviceAttributeMaxThreadsPerMultiProcessor) as _,
            max_smem: self
                .get_attribute(mcDeviceAttributeMaxSharedMemoryPerMultiprocessor)
                .into(),
            max_registers: self
                .get_attribute(mcDeviceAttributeMaxRegistersPerMultiprocessor)
                .into(),
        }
    }

    #[inline]
    pub fn info(&self) -> InfoFmt {
        InfoFmt(self)
    }

    #[inline]
    fn get_attribute(&self, attr: mcDeviceAttribute_t) -> c_int {
        let mut value = 0;
        mxdrv!(mcDeviceGetAttribute(&mut value, attr, self.0));
        value
    }
}

pub struct InfoFmt<'a>(&'a Device);

impl fmt::Display for InfoFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let block_limit = self.0.block_limit();
        let sm_limit = self.0.sm_limit();
        let grid = self.0.max_grid_dims();
        writeln!(
            f,
            "\
GPU{} ({})
  cc = {}
  gmem = {}
  alignment = {}
  warp size = {}
  sm count = {}
  block limit
    threads = {} (x: {}, y: {}, z: {})
    smem = {}
    registers = {}
  sm limit
    blocks = {}
    threads = {}
    smem = {}
    registers = {}
  grid = (x: {}, y: {}, z: {})",
            self.0 .0,
            self.0.name(),
            self.0.compute_capability(),
            self.0.total_memory(),
            self.0.alignment(),
            self.0.warp_size(),
            self.0.sm_count(),
            block_limit.max_threads,
            block_limit.max_dims.x,
            block_limit.max_dims.y,
            block_limit.max_dims.z,
            block_limit.max_smem,
            block_limit.max_registers,
            sm_limit.max_blocks,
            sm_limit.max_threads,
            sm_limit.max_smem,
            sm_limit.max_registers,
            grid.x,
            grid.y,
            grid.z,
        )
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BlockLimit {
    pub max_threads: usize,
    pub max_dims: Dim3,
    pub max_smem: MemSize,
    pub max_registers: MemSize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SMLimit {
    pub max_blocks: usize,
    pub max_threads: usize,
    pub max_smem: MemSize,
    pub max_registers: MemSize,
}

#[test]
fn test() {
    if let Err(crate::NoDevice) = crate::init() {
        return;
    }
    for i in 0..Device::count() {
        println!("{}", Device::new(i as _).info());
    }
}
