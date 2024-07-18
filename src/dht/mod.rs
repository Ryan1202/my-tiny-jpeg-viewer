use huffman::{Huffman, HuffmanErrorType};

pub mod huffman;

pub enum HuffmanTableType {
    DC,
    AC,
}

pub struct HuffmanTable {
    id: u8,
    ht_type: HuffmanTableType,
    pub huff: Huffman,
}

impl HuffmanTable {
    pub fn new(data: Vec<u8>) -> Result<Self, HuffmanErrorType> {
        let num = data[0] & 0x0f;
        let ht_type = if data[0] & 0x10 == 0x10 {HuffmanTableType::AC} else {HuffmanTableType::DC};

        let huffman = Huffman::parse(data)?;

        Ok(HuffmanTable { id: num, ht_type: ht_type, huff: huffman })
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn get_type(&self) -> &HuffmanTableType {
        &self.ht_type
    }
}