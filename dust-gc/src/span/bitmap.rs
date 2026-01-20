pub struct Bitmap {
    byte_pointer: *const u8,
}

impl Bitmap {
    pub fn get_bit(&self, index: usize) -> bool {
        let bit_mask = 1 << (index % 8) as u8;
        let byte_pointer = self.get_byte_pointer_at(index / 8);
        let byte = unsafe { *byte_pointer };

        byte & bit_mask != 0
    }

    fn get_byte_pointer_at(&self, index: usize) -> *const u8 {
        unsafe { self.byte_pointer.add(index) }
    }
}

pub struct BitmapArena {
    free: usize,
    bytes: [u8; Self::PAYLOAD_SIZE],
}

impl BitmapArena {
    const SIZE: usize = 1024 * 64;
    const HEADER_SIZE: usize = size_of::<usize>();
    const PAYLOAD_SIZE: usize = Self::SIZE - Self::HEADER_SIZE;

    pub fn new() -> Self {
        Self {
            free: 0,
            bytes: [0; Self::PAYLOAD_SIZE],
        }
    }

    pub fn get_bitmap(&mut self, bit_count: usize) -> Option<Bitmap> {
        let bytes_needed = bit_count.div_ceil(8);

        if bytes_needed <= self.free {
            let used = Self::PAYLOAD_SIZE - self.free;
            let byte_pointer = unsafe { self.bytes.as_ptr().add(used) };

            self.free -= bytes_needed;

            Some(Bitmap { byte_pointer })
        } else {
            None
        }
    }
}
