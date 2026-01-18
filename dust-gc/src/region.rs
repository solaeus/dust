use std::{io, ptr::NonNull};

pub struct Region {
    pointer: NonNull<u8>,
    size: usize,
}

impl Region {
    pub fn new(size: usize) -> Result<Self, RegionError> {
        if size == 0 {
            return Err(RegionError::ZeroSizedRegion);
        }

        todo!()
    }

    pub fn pointer(&self) -> NonNull<u8> {
        self.pointer
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug)]
pub enum RegionError {
    ZeroSizedRegion,
    Os(io::Error),
    #[cfg(unix)]
    Platform(nix::errno::Errno),
}
