use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Seek},
    path::Path,
    rc::Rc,
};

use application::InterchangeFormat;
use bitstream::BitStream;
use component::{
    frame::{Frame, FrameErrorType},
    scan::Scan,
    Component,
};
use decode::dct::DCT;
use dht::{huffman::HuffmanErrorType, HuffmanTable};
use dqt::Dqt;
use segment::{Segment, SegmentType};

pub mod application;
pub mod bitstream;
pub mod component;
pub mod decode;
pub mod dht;
pub mod dqt;
pub mod segment;
pub mod ui;
pub mod zigzag;

pub async fn get_jpeg_image_async(path: String) -> (usize, usize, Vec<u8>) {
    get_jpeg_image(path)
}

pub fn get_jpeg_image(path: String) -> (usize, usize, Vec<u8>) {
    let jpg_file = File::open(path).expect("打开文件失败！");
    // let jpg_file = File::open("s11138117.jpg").expect("打开文件失败！");
    let mut reader = BufReader::new(jpg_file);
    let segs = match Segment::from_file(&mut reader) {
        Ok(segs) => segs,
        Err(e) => {
            panic!("解析失败: {:?}", e);
        }
    };

    let mut _if = Vec::new();
    let mut dqt_map = HashMap::new();
    let mut dc_map = HashMap::new();
    let mut ac_map = HashMap::new();
    let mut frame = None;
    let mut scan = None;
    let mut st = None;
    let mut ed = None;
    let mut restart_interval = None;

    for ele in segs {
        match ele.segment_type {
            SegmentType::SOI => {
                continue;
            }
            SegmentType::APPn(n) => match n {
                0..=1 => {
                    _if.push(InterchangeFormat::new(n, &ele));
                }
                _ => {
                    // println!("不支持的段类型: APP{} !", n);
                }
            },
            SegmentType::DQT => {
                match Dqt::new(&mut dqt_map, ele.length, ele.data) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("In DQT: NDArray: Shape Error! {:?}", e)
                    }
                };
            }
            SegmentType::DHT => {
                match HuffmanTable::new(&mut dc_map, &mut ac_map, ele.length, ele.data) {
                    Ok(_) => {}
                    Err(e) => match e {
                        HuffmanErrorType::BitStreamError(_) => {
                            panic!("In Huffman: BitStream Read Error!")
                        }
                        HuffmanErrorType::DecodeError(bs) => {
                            panic!("In Huffman: Decode Error! BitStream:{}", bs)
                        }
                        HuffmanErrorType::InvalidCode(symbol, code) => {
                            panic!("In Huffman: Invalid Code: Symbol:{}, Code:{}", symbol, code)
                        }
                        HuffmanErrorType::TypeConversionError => {
                            panic!("In Huffman: Type Conversion Error!")
                        }
                    },
                };
            }
            SegmentType::SOFn(n) => {
                frame = match Frame::new(n, ele.data) {
                    Ok(f) => Some(f),
                    Err(e) => match e {
                        FrameErrorType::InvalidFrameType(n) => {
                            panic!("In Frame: Invalid Frame Type:{}", n)
                        }
                    },
                };
            }
            SegmentType::SOS(start, end) => {
                scan = Some(Scan::new(ele.data));
                st = Some(start);
                ed = Some(end);
            }
            SegmentType::DRI => {
                restart_interval = Some(u16::from_be_bytes([ele.data[0], ele.data[1]]));
            }
            _ => {
                // println!("不支持的段类型!");
            }
        }
    }

    let frame = frame.unwrap();
    let scan = scan.unwrap();
    let st = st.unwrap();
    let ed = ed.unwrap();

    reader.seek(std::io::SeekFrom::Start(st)).unwrap();
    let mut bs = BitStream::new(&mut reader);

    let width = frame.get_width() as usize;
    let height = frame.get_height() as usize;

    let dct = DCT::new();
    let comps = Component::new(&frame, dqt_map, dc_map, ac_map, scan).unwrap();
    (
        width,
        height,
        decode::decode_image(&frame, comps, &mut bs, restart_interval, dct).unwrap(),
    )
}
