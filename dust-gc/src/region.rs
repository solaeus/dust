use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[cfg(not(target_arch = "wasm32"))]
pub type Region = mmap::MmapRegion;

#[derive(Debug)]
struct ZeroSizedRegionError;

impl Display for ZeroSizedRegionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Tried to allocatate zero-sized memorty region")
    }
}

impl Error for ZeroSizedRegionError {}

mod mmap {
    use super::*;

    use std::io;

    use memmap2::MmapMut;

    pub struct MmapRegion {
        mmap: MmapMut,
    }

    impl MmapRegion {
        pub fn new(size: usize) -> Result<Self, io::Error> {
            if size == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    ZeroSizedRegionError,
                ));
            }

            Ok(Self {
                mmap: MmapMut::map_anon(size)?,
            })
        }

        pub fn base(&self) -> usize {
            self.mmap.as_ptr() as usize
        }

        pub fn size(&self) -> usize {
            self.mmap.len()
        }
    }
}
