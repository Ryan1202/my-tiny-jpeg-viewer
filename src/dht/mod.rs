use std::rc::Rc;

use huffman::{Huffman, HuffmanErrorType};
use rustc_hash::FxHashMap;

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
    pub fn new(
        dc_map: &mut FxHashMap<u8, Rc<HuffmanTable>>,
        ac_map: &mut FxHashMap<u8, Rc<HuffmanTable>>,
        length: u16,
        data: Vec<u8>,
    ) -> Result<(), HuffmanErrorType> {
        let len = length as usize - 2;
        let mut offset = 0 as usize;

        while offset < len {
            let num = data[offset] & 0x0f;
            let ht_type = if data[offset] & 0x10 == 0x10 {
                HuffmanTableType::AC
            } else {
                HuffmanTableType::DC
            };
            let (huffman, off) = Huffman::parse(&data, offset)?;
            match ht_type {
                HuffmanTableType::DC => {
                    dc_map.insert(
                        num,
                        Rc::new(HuffmanTable {
                            id: num,
                            ht_type: ht_type,
                            huff: huffman,
                        }),
                    );
                }
                HuffmanTableType::AC => {
                    ac_map.insert(
                        num,
                        Rc::new(HuffmanTable {
                            id: num,
                            ht_type: ht_type,
                            huff: huffman,
                        }),
                    );
                }
            }
            offset = off;
        }

        Ok(())
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn get_type(&self) -> &HuffmanTableType {
        &self.ht_type
    }
}
