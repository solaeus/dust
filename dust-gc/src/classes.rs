#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SizeClass(u8);

impl SizeClass {
    pub const CLASS_COUNT: usize = 68;
    pub const LARGE_OBJECT_CLASS: SizeClass = SizeClass(0);

    const SIZES: [usize; Self::CLASS_COUNT] = [
        0, 8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288,
        320, 352, 384, 416, 448, 480, 512, 576, 640, 704, 768, 896, 1024, 1152, 1280, 1408, 1536,
        1792, 2048, 2304, 2688, 3072, 3200, 3456, 4096, 4864, 5376, 6144, 6528, 6784, 6912, 8192,
        9472, 9728, 10240, 10880, 12288, 13568, 14336, 16384, 18432, 19072, 20480, 21760, 24576,
        27264, 28672, 32768,
    ];

    pub fn from_size(size: usize) -> Option<Self> {
        todo!()
    }

    pub const fn size(&self) -> usize {
        Self::SIZES[self.0 as usize]
    }

    pub const fn index(&self) -> u8 {
        self.0
    }

    pub fn page_count(&self) -> usize {
        todo!()
    }

    pub fn is_large(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanClass(u8);

impl SpanClass {
    pub const SPAN_CLASS_COUNT: usize = 136;

    pub fn new(size_class: SizeClass, noscan: bool) -> Self {
        todo!()
    }

    pub fn size_class(&self) -> SizeClass {
        todo!()
    }

    pub fn is_noscan(&self) -> bool {
        todo!()
    }

    pub fn is_scan(&self) -> bool {
        !self.is_noscan()
    }

    pub const fn index(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_size() {
        assert_eq!(SizeClass::from_size(0), Some(SizeClass(1)));
        assert_eq!(SizeClass::from_size(1), Some(SizeClass(1)));
        assert_eq!(SizeClass::from_size(8), Some(SizeClass(1)));
        assert_eq!(SizeClass::from_size(9), Some(SizeClass(2)));
        assert_eq!(SizeClass::from_size(15), Some(SizeClass(2)));
        assert_eq!(SizeClass::from_size(16), Some(SizeClass(2)));
        assert_eq!(SizeClass::from_size(17), Some(SizeClass(3)));
        assert_eq!(SizeClass::from_size(1000), Some(SizeClass(31)));
        assert_eq!(SizeClass::from_size(4096), Some(SizeClass(44)));
        assert_eq!(SizeClass::from_size(32768), Some(SizeClass(67)));
        assert_eq!(SizeClass::from_size(32769), None);
    }

    #[test]
    fn alloc_size() {
        assert_eq!(SizeClass(1).page_count(), 1);
        assert_eq!(SizeClass(44).page_count(), 1);
        assert_eq!(SizeClass(51).page_count(), 1);
        assert_eq!(SizeClass(67).page_count(), 4);
    }
}
