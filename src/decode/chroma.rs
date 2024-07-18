pub fn ycbcr2rgb(y: f32, cb: f32, cr: f32) -> (u8, u8, u8) {
    let r = (128.0 + y + 1.402*cr).round() as u8;
    let g = (128.0 + y - 0.714*cr - 0.344*cb).round() as u8;
    let b = (128.0 + y + 1.772*cb).round() as u8;
    (r, g, b)
}