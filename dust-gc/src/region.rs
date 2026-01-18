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
}

pub enum RegionError {
    ZeroSizedRegion,
    Os(io::Error),
    #[cfg(unix)]
    Platform(nix::errno::Errno),
}

fn page_size() -> Result<usize, RegionError> {
    #[cfg(unix)]
    let size = { nix::libc::_SC_PAGESIZE };

    if size <= 0 {
        Err(RegionError::Os(io::Error::last_os_error()))
    } else {
        Ok(size as usize)
    }
}
