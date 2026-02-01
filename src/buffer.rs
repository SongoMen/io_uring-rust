pub unsafe trait IoBuf {
    fn stable_ptr(&self) -> *const u8;
    fn bytes_init(&self) -> usize;
    fn bytes_total(&self) -> usize;
}

unsafe impl IoBuf for Vec<u8> {
    fn stable_ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn bytes_init(&self) -> usize {
        self.len()
    }
    fn bytes_total(&self) -> usize {
        self.capacity()
    }
}

pub unsafe trait IoBufMut: IoBuf {
    fn stable_mut_ptr(&mut self) -> *mut u8;
}

unsafe impl IoBufMut for Vec<u8> {
    fn stable_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
}
