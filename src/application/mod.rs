mod jfif;
use jfif::JFIF;

use crate::segment::Segment;

#[derive(Debug)]
pub enum InterchangeFormat {
    JFIF(JFIF),
    EXIF,
    Unknown,
}

pub enum IfErrorType {
    InvalidInterchangeFormat,
}

impl InterchangeFormat {
    pub fn new(n: u8, seg: &Segment) -> Self {
        if n == 0 {
            InterchangeFormat::JFIF(JFIF::new(&seg.data))
        } else if n == 1 {
            InterchangeFormat::EXIF
        } else {
            println!("无效的交换格式！");
            InterchangeFormat::Unknown
        }
    }
}
