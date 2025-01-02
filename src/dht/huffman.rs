use std::{fmt::Debug, fs::File, io::BufReader};

use rustc_hash::FxHashMap;

use crate::bitstream::{Binary, BitStream, BitStreamErrorType};

const HUFFMAN_INDEX_BITS: usize = 8;
const HUFFMAN_TABLE_SIZE: usize = 1 << HUFFMAN_INDEX_BITS;

#[derive(Clone, Copy)]
pub enum HuffmanTableValue {
    Defined(u8 /* 值 */, u8 /* 位长度 */),
    Undefined,
}

#[derive(Clone)]
pub struct HuffmanTable {
    pub table: [HuffmanTableValue; HUFFMAN_TABLE_SIZE],
}
impl Debug for HuffmanTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.table.iter().map(|v| match v {
                HuffmanTableValue::Defined(v, l) => format!("Defined: {}, code {}bit(s)", v, l),
                HuffmanTableValue::Undefined => "Undefined".to_string(),
            }))
            .finish()
    }
}

#[derive(Debug)]
pub struct Huffman {
    _length: [u8; 16],
    map: FxHashMap<Binary, u8>,
    table: HuffmanTable,
}

#[derive(Debug)]
pub enum HuffmanErrorType {
    InvalidCode(u8 /* symbol */, usize /* code */),
    BitStreamError(BitStreamErrorType),
    DecodeError(usize /* code */),
    TypeConversionError,
}

impl Huffman {
    pub fn parse(data: &Vec<u8>, offset: usize) -> Result<(Self, usize), HuffmanErrorType> {
        let mut length = [0; 16];
        let mut val: Vec<Vec<u8>> = Vec::with_capacity(16);
        let mut off = offset + 17;
        let mut code = 0usize;
        let mut map = FxHashMap::default();
        let mut table = HuffmanTable {
            table: [HuffmanTableValue::Undefined; HUFFMAN_TABLE_SIZE],
        };

        for i in 0..16 {
            length[i] = data[offset + i + 1];
            let bit_length = i + 1;
            let mut v = Vec::new();
            for j in 0..length[i] as usize {
                v.push(data[off + j]);
                if code >= (1 << (i + 2)) {
                    return Err(HuffmanErrorType::InvalidCode(data[off + j], code));
                }
                map.insert(Binary::new(code, bit_length), data[off + j]);

                if bit_length <= HUFFMAN_INDEX_BITS {
                    let left_bit_length = HUFFMAN_INDEX_BITS - bit_length;
                    let value = code << left_bit_length;
                    for k in 0..(1 << left_bit_length) {
                        if let HuffmanTableValue::Undefined = table.table[value + k] {
                            table.table[value + k] =
                                HuffmanTableValue::Defined(data[off + j], bit_length as u8);
                        }
                    }
                }

                code += 1;
            }
            val.push(v);
            off += length[i] as usize;
            code <<= 1;
        }

        Ok((
            Self {
                _length: length,
                map: map,
                table: table,
            },
            off,
        ))
    }

    pub fn decode(&self, code: &mut BitStream<BufReader<File>>) -> Result<u8, HuffmanErrorType> {
        let value = code.try_read(16)
            .map_err(|e| HuffmanErrorType::BitStreamError(e))?;

        let test_read = value >> (16 - HUFFMAN_INDEX_BITS);
        let test_value = self.table.table[test_read as usize];
        if let HuffmanTableValue::Defined(value, bit_length) = test_value {
            code.read(bit_length as usize)
                .map_err(|e| HuffmanErrorType::BitStreamError(e))?;
            return Ok(value);
        }

        let mut read = Binary::new(test_read, HUFFMAN_INDEX_BITS);

        for i in HUFFMAN_INDEX_BITS..16 {
            read.value = value >> (15 - i);
            read.bit_length = i + 1;
            if let Some(v) = self.map.get(&read) {
                code.read(i + 1)
                    .map_err(|e| HuffmanErrorType::BitStreamError(e))?;
                return Ok(*v);
            }
        }
        Err(HuffmanErrorType::DecodeError(read.get_value()))
    }
}

#[cfg(test)]
mod huffman_test {
    use super::*;

    #[test]
    fn huffman_test() {
        let (huffman, _) = Huffman::parse(
            &vec![
                0x11, 0, 2, 2, 2, 1, 3, 2, 5, 2, 4, 5, 5, 0, 3, 0, 0, 1, 2, 0, 3, 4, 0x11, 0x21, 5,
                0x12, 0x31, 0x13, 0x41, 6, 0x22, 0x32, 0x51, 0x61, 0x14, 0x71, 0x23, 0x81, 0x91,
                0xa1, 0x15, 0x42, 0xb1, 0xc1, 0xd1, 7, 0x33, 0x52, 0xe1, 0xf0, 0x24, 0x62, 0xf1,
            ],
            0,
        )
        .unwrap();
        dbg!(huffman.map);
    }
}
