pub trait Widget {
    fn draw(
        &mut self,
        canvas: &mut [u8],
        global_offset_x: usize,
        global_offset_y: usize,
        width: usize,
    );
}
