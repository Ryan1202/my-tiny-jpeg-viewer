use std::{
    fs::File,
    io::{self, BufReader, Read, Seek},
};

#[derive(Debug, Clone, Copy)]
pub enum SegmentType {
    SOI,
    APPn(u8),
    DQT,
    SOFn(u8),
    DHT,
    DRI,
    SOS(u64, u64),
    COM,
    EOI,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub segment_type: SegmentType,
    pub length: u16,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum SegmentErrorKind {
    IOError(io::Error),
    InvalidSegment,
    InvalidSegmentType,
    InvalidSegmentLength,
}

impl Segment {
    fn new(reader: &mut BufReader<File>, offset: usize) -> Result<Self, SegmentErrorKind> {
        reader
            .seek(io::SeekFrom::Start(offset as u64))
            .map_err(|e| SegmentErrorKind::IOError(e))?;

        let mut buffer = [0u8; 2];
        reader
            .read_exact(&mut buffer)
            .map_err(|e| SegmentErrorKind::IOError(e))?;

        // 确认标记(0xFF)
        let sign = buffer[0];
        if sign != 0xFF {
            return Err(SegmentErrorKind::InvalidSegment);
        }

        // 判断类型
        let segment_type = match buffer[1] {
            0xD8 => SegmentType::SOI,
            0xDB => SegmentType::DQT,
            0xFE => SegmentType::COM,
            0xC4 => SegmentType::DHT,
            0xDA => {
                reader
                    .seek(io::SeekFrom::Current(12))
                    .map_err(|e| SegmentErrorKind::IOError(e))?;
                let scandata_start = reader
                    .stream_position()
                    .map_err(|e| SegmentErrorKind::IOError(e))?;
                let scandata_end;

                loop {
                    let mut buffer = [0u8; 1];
                    reader
                        .read_exact(&mut buffer)
                        .map_err(|e| SegmentErrorKind::IOError(e))?;
                    if buffer[0] == 0xFF {
                        reader
                            .read_exact(&mut buffer)
                            .map_err(|e| SegmentErrorKind::IOError(e))?;
                        if buffer[0] == 0xD9 {
                            scandata_end = reader
                                .stream_position()
                                .map_err(|e| SegmentErrorKind::IOError(e))?
                                - 2;
                            break;
                        }
                    }
                }
                reader
                    .seek(io::SeekFrom::Start(scandata_start - 12))
                    .map_err(|e| SegmentErrorKind::IOError(e))?;
                SegmentType::SOS(scandata_start, scandata_end)
            }
            0xDD => SegmentType::DRI,
            0xD9 => SegmentType::EOI,
            n => {
                if n >= 0xC0 && n < 0xC4 {
                    SegmentType::SOFn(n - 0xC0)
                } else if n >= 0xE0 && n < 0xF0 {
                    SegmentType::APPn(n - 0xE0)
                } else {
                    return Err(SegmentErrorKind::InvalidSegmentType);
                }
            }
        };
        if let SegmentType::SOI = segment_type {
            return Ok(Self {
                segment_type: segment_type,
                length: 0,
                data: vec![],
            });
        }
        if let SegmentType::EOI = segment_type {
            return Ok(Self {
                segment_type: segment_type,
                length: 0,
                data: vec![],
            });
        }

        reader
            .read_exact(&mut buffer)
            .map_err(|e| SegmentErrorKind::IOError(e))?;
        let length = u16::from_be_bytes([buffer[0], buffer[1]]);
        if length > 65533 {
            return Err(SegmentErrorKind::InvalidSegmentLength);
        }

        let mut data = vec![0u8; length as usize - 2];
        reader
            .read_exact(&mut data)
            .map_err(|e| SegmentErrorKind::IOError(e))?;

        Ok(Self {
            segment_type: segment_type,
            length: length,
            data: data,
        })
    }

    pub fn from_file(reader: &mut BufReader<File>) -> Result<Vec<Self>, SegmentErrorKind> {
        let mut i = 0;
        let mut segments = Vec::new();
        let mut offset = 0;
        loop {
            let segment = Self::new(reader, offset)?;
            println!(
                "Segment {}: {{Type:{:?},Length:{}}}",
                i, segment.segment_type, segment.length
            );

            let _type = segment.segment_type;

            i += 1;
            offset += segment.length as usize + 2;
            if let SegmentType::SOS(_, end) = _type {
                offset = end as usize;
            }
            segments.push(segment);

            if let SegmentType::EOI = _type {
                break;
            }
        }

        Ok(segments)
    }
}
