use std::{
    cmp::min,
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek},
};

#[derive(Debug)]
pub enum BitStreamErrorType {
    IOError(io::Error),
    Empty,
}

#[derive(Debug, Hash)]
pub struct Binary {
    pub value: usize,
    pub bit_length: usize,
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
    fn get_position(&mut self) -> Result<usize, BitStreamErrorType> {
        Ok(0)
    }
    fn read_byte(&mut self, offset: usize) -> Result<(u8, usize), BitStreamErrorType>;
    fn skip_byte(&mut self, count: usize) -> Result<(), BitStreamErrorType>;
    fn remove_byte(&mut self) -> Result<u8, BitStreamErrorType>;
    fn print_pos(&mut self);
}

impl BitReader for BufReader<File> {
    fn get_position(&mut self) -> Result<usize, BitStreamErrorType> {
        self.stream_position()
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        Ok(0)
    }
    fn read_byte(&mut self, offset: usize) -> Result<(u8, usize), BitStreamErrorType> {
        let buffer = self.fill_buf()
            .map_err(|e| BitStreamErrorType::IOError(e))?;

        if offset + 1 >= buffer.len() {
            let mut buf = [0u8];
            let mut next_pos = offset + 1;
    
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
                    next_pos += 1;
                }
            }
            self.seek(io::SeekFrom::Start(pos))
                .map_err(|e| BitStreamErrorType::IOError(e))?;
            Ok((buf[0], next_pos))
        } else {
            if buffer[offset] == 0xff && buffer[offset + 1] == 0x00 {
                Ok((0xff, offset + 2))
            } else {
                Ok((buffer[offset], offset + 1))
            }
        }
    }


    fn skip_byte(&mut self, offset: usize) -> Result<(), BitStreamErrorType> {
        self.seek(io::SeekFrom::Current(offset as i64))
            .map_err(|e| BitStreamErrorType::IOError(e))?;
        Ok(())
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
    fn print_pos(&mut self) {
        let pos = self.stream_position()
            .map_err(|e| BitStreamErrorType::IOError(e))
            .unwrap();
        print!("pos: {:02x} ", pos);
    }
}

impl BitReader for Vec<u8> {
    fn read_byte(&mut self, offset: usize) -> Result<(u8, usize), BitStreamErrorType> {
        if self.is_empty() {
            return Err(BitStreamErrorType::Empty);
        } else {
            if self[offset] == 0xff && self[offset + 1] == 0x00 {
                Ok((self[offset], offset + 2))
            } else {
                Ok((self[offset], offset + 1))
            }
        }
    }
    fn skip_byte(&mut self, offset: usize) -> Result<(), BitStreamErrorType> {
        if self.len() < offset {
            return Err(BitStreamErrorType::Empty);
        } else {
            self.drain(0..offset);
            Ok(())
        }
    }
    fn remove_byte(&mut self) -> Result<u8, BitStreamErrorType> {
        if self.is_empty() {
            return Err(BitStreamErrorType::Empty);
        } else {
            Ok(self.remove(0))
        }
    }
    fn print_pos(&mut self) {
        println!("left: {}", self.len());
    }
}

#[derive(Debug)]
pub struct BitStream<'a, R: BitReader> {
    reader: &'a mut R,
    cur_byte: u8,
    bit_start: usize,
    position: usize,
}

impl<'a, R: BitReader> BitStream<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        let mut bs = Self {
            reader: reader,
            cur_byte: 0,
            bit_start: 0,
            position: 0,
        };
        bs.position = bs.reader.get_position().unwrap();
        bs
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

    pub fn try_read(&mut self, n: usize) -> Result<usize, BitStreamErrorType> {
        if n == 0 {
            return Err(BitStreamErrorType::Empty);
        }
        let mut result = 0;
        let mut len;
        let mut left_len = n;
        let mut bit_start = self.bit_start;
        let mut cur_byte = self.cur_byte;
        let mut offset = 0;
        while left_len > 0 {
            if bit_start % 8 == 0 {
                bit_start = 0;
                (cur_byte, offset) = self.reader.read_byte(offset)?;
            }
            len = min(8 - bit_start, left_len);
            bit_start += len;
            result = (result << len)
                | (cur_byte as usize >> (8 - bit_start)) & ((1 << len) - 1);
            left_len -= len;
        }
        Ok(result)
    }

    pub fn skip(&mut self, n: usize) -> Result<(), BitStreamErrorType> {
        if self.bit_start % 8 == 0 {
            self.bit_start = 0;
            self.cur_byte = self.reader.remove_byte()?;
        }

        let skip_bit = (self.bit_start + n) % 8;
        let skip_byte = (self.bit_start + n) / 8;
        if skip_byte > 1 {
            self.reader.skip_byte(skip_byte - 1)?;
        }
        if (skip_byte >= 1 || self.bit_start % 8 == 0) && skip_bit > 0 {
            self.cur_byte = self.reader.remove_byte()?;
        }
        self.bit_start = skip_bit;
        Ok(())
    }

    pub fn get_bit_start(&self) -> usize {
        self.bit_start
    }

    pub fn align_byte(&mut self) {
        if 0 < self.bit_start && self.bit_start < 8 {
            self.bit_start = 8;
        }
    }

    pub fn print_pos(&mut self) {
        self.reader.print_pos();
        print!("bit_start: {:02x} ", self.bit_start);
    }
}
