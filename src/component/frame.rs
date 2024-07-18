use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum FrameTypeCoding {
    HuffmanCoding,
    ArithmeticCoding,
}
#[derive(Clone, Copy)]
pub enum FrameType {
    BaselineDCT,
    ExtendedDCT(FrameTypeCoding),
    ProgressiveDCT(FrameTypeCoding),
    Lossless(FrameTypeCoding),
}

pub struct Frame {
    frame_type: FrameType,
    sample_precision: u8,
    height: u16,
    width: u16,
    pub components: HashMap<u8, FrameComponent>,
}
pub struct FrameComponent {
    id: u8,
    sample_factor_x: u8,
    sample_factor_y: u8,
    quantization_table_id: u8,
}

pub enum FrameErrorType {
    InvalidFrameType(u8),
}

impl Frame {
    pub fn new(n: u8, data: Vec<u8>) -> Result<Self, FrameErrorType> {
        let precision = data[0];
        let height = u16::from_be_bytes([data[1], data[2]]);
        let width = u16::from_be_bytes([data[3], data[4]]);
        let comp_nr = data[5] as usize;
        let mut v = HashMap::with_capacity(comp_nr as usize);
        for i in 0..comp_nr {
            let comp = FrameComponent::new(data[i*3 + 6], data[i*3 + 7], data[i*3 + 8]);
            v.insert(comp.get_id(), comp);
        }

        let frame_type = match n {
            0 => {FrameType::BaselineDCT},
            1 => {FrameType::ExtendedDCT(FrameTypeCoding::HuffmanCoding)},
            2 => {FrameType::ProgressiveDCT(FrameTypeCoding::HuffmanCoding)},
            3 => {FrameType::Lossless(FrameTypeCoding::HuffmanCoding)},
            9 => {FrameType::ExtendedDCT(FrameTypeCoding::ArithmeticCoding)},
            10 => {FrameType::ProgressiveDCT(FrameTypeCoding::ArithmeticCoding)},
            11 => {FrameType::Lossless(FrameTypeCoding::ArithmeticCoding)},
            _ => {return Err(FrameErrorType::InvalidFrameType(n));},
        };

        Ok(Self { frame_type: frame_type, sample_precision: precision, height: height, width: width, components: v })
    }

    pub fn get_type(&self) -> FrameType {
        self.frame_type
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }
}

impl FrameComponent {
    fn new(c: u8, hv: u8, tq: u8) -> Self {
        Self {
            id: c,
            sample_factor_x: hv >> 4,
            sample_factor_y: hv & 0x0F,
            quantization_table_id: tq,
        }
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_factor_x(&self) -> u8 {
        self.sample_factor_x
    }

    pub fn get_factor_y(&self) -> u8 {
        self.sample_factor_y
    }

    pub fn get_qid(&self) -> u8 {
        self.quantization_table_id
    }
}