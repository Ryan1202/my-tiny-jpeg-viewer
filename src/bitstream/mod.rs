use std::{
    cmp::min,
    fs::File,
    io::{self, BufReader, Read, Seek},
};

#[derive(Debug)]
pub enum BitStreamErrorType {
    IOError(io::Error),
    Empty,
}

#[derive(Debug, Hash)]
pub struct Binary {
    value: usize,
    bit_length: usize,
}

impl Binary {
    pub fn new(value: usize, bit_length: usize) -> Self {
        Self { value, bit_length }
    }

    pub fn push_bit(&mut self, value: usize) {
        self.value = (self.value << 1) | (value & 1);
        self.bit_length += 1;
    }

    pub fn get_value(&self) -> usize {
        self.value
    }
}

impl PartialEq for Binary {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.bit_length == other.bit_length
    }
}

impl Eq for Binary {}

pub trait BitReader {
    fn read_byte(&mut self, offset: usize) -> Result<u8, BitStreamErrorType>;
    fn remove_byte(&mut self) -> Result<u8, BitStreamErrorType>;
}

impl BitReader for BufReader<File> {
    fn read_byte(&mut self, offset: usize) -> Result<u8, BitStreamErrorType> {
        let mut buf = [0u8];
        let pos = self
            .stream_position()
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        self.seek(io::SeekFrom::Current(offset as i64))
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        self.read_exact(&mut buf)
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        if buf[0] == 0xff {
            self.read_exact(&mut buf)
                .map_err(|e| BitStreamErrorType::IOError(e))?;
            if buf[0] == 0x00 {
                buf[0] = 0xff;
            }
        }
        self.seek(io::SeekFrom::Start(pos))
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        Ok(buf[0])
    }
    fn remove_byte(&mut self) -> Result<u8, BitStreamErrorType> {
        let mut buf = [0u8];
        self.read_exact(&mut buf)
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        if buf[0] == 0xff {
            self.read_exact(&mut buf)
                .map_err(|e| BitStreamErrorType::IOError(e))?;
            if buf[0] == 0x00 {
                buf[0] = 0xff;
            }
        }
        Ok(buf[0])
    }
}

impl BitReader for Vec<u8> {
    fn read_byte(&mut self, offset: usize) -> Result<u8, BitStreamErrorType> {
        if self.is_empty() {
            return Err(BitStreamErrorType::Empty);
        } else {
            Ok(self[offset])
        }
    }

    fn remove_byte(&mut self) -> Result<u8, BitStreamErrorType> {
        if self.is_empty() {
            return Err(BitStreamErrorType::Empty);
        } else {
            Ok(self.remove(0))
        }
    }
}

#[derive(Debug)]
pub struct BitStream<'a, R: BitReader> {
    reader: &'a mut R,
    cur_byte: u8,
    bit_start: usize,
}

impl<'a, R: BitReader> BitStream<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            reader: reader,
            cur_byte: 0,
            bit_start: 0,
        }
    }

    pub fn read(&mut self, n: usize) -> Result<usize, BitStreamErrorType> {
        if n == 0 {
            return Err(BitStreamErrorType::Empty);
        }
        let mut result = 0;
        let mut len;
        let mut left_len = n;
        while left_len > 0 {
            if self.bit_start % 8 == 0 {
                self.bit_start = 0;
                self.cur_byte = self.reader.remove_byte()?;
            }
            len = min(8 - self.bit_start, left_len);
            self.bit_start += len;
            result = (result << len)
                | (self.cur_byte as usize >> (8 - self.bit_start)) & ((1 << len) - 1);
            left_len -= len;
        }
        Ok(result)
    }

    pub fn align_byte(&mut self) {
        if 0 < self.bit_start && self.bit_start < 8 {
            self.bit_start = 8;
        }
    }
}
