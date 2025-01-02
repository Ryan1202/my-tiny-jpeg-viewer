use std::{
    borrow::BorrowMut, error, fs::File, io::BufReader,
    rc::Rc,
};

use ndarray::prelude::*;
use rustc_hash::FxHashMap;

use crate::{bitstream::BitStream, component::Component, dht::HuffmanTable, zigzag::ZigZagScan};

use super::dct::DCT;

pub struct MCU {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Block>,
}

pub struct Block {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Vec<[[f32; 8]; 8]>>,
}

fn decode_dct(
    dc: &HuffmanTable,
    last_dc: isize,
    ac: &HuffmanTable,
    bs: &mut BitStream<BufReader<File>>,
) -> [isize; 64] {
    let mut code = [0isize; 64];
    // DC
    let codeval = dc.huff.decode(bs).unwrap();
    let len = codeval as usize;
    if len == 0 {
        code[0] = last_dc;
    } else if len == 1 {
        code[0] = last_dc + bs.read(len).unwrap() as isize * 2 - 1; // 0 -> -1, 1 -> 1
    } else {
        let sign = bs.read(1).unwrap();
        let num = sign << (len - 1) | bs.read(len - 1).unwrap();
        let result;
        if sign == 0 {
            result = -(((!num) & ((1 << len) - 1)) as isize);
        } else {
            result = num as isize;
        }
        code[0] = result + last_dc;
    }

    // AC
    let mut i = 1;
    while i < 64 {
        let codeval = ac.huff.decode(bs).unwrap();
        let zero = codeval >> 4;
        let len = (codeval & 0x0f) as usize;

        if len == 0 {
            if codeval == 0xf0 {
                i += 15;
            } else {
                break;
            }
        } else if len == 1 {
            i += zero as usize;
            code[i] = bs.read(len).unwrap() as isize * 2 - 1; // 0 -> -1, 1 -> 1
        } else {
            let sign = bs.read(1).unwrap();
            let num = sign << (len - 1) | bs.read(len - 1).unwrap();
            let result;
            if sign == 0 {
                result = -(((!num) & ((1 << len) - 1)) as isize);
            } else {
                result = num as isize;
            }
            i += zero as usize;
            code[i] = result;
        }
        i += 1;
    }
    code
}

pub fn decode_blocks(
    mut last_dc: Vec<isize>,
    comps: FxHashMap<u8, Rc<Component>>,
    bs: &mut BitStream<BufReader<File>>,
    dct: &DCT,
) -> Result<(Vec<isize>, MCU), Box<dyn error::Error>> {
    let mut mcu = Vec::new();

    let mut max_width = 0;
    let mut max_height = 0;

    let len = comps.len();
    for idx in 1..=len {
        let comp = comps.get(&(idx as u8)).unwrap().clone();

        let width = comp.get_factor_x() as usize;
        let height = comp.get_factor_y() as usize;
        max_width = std::cmp::max(max_width, width);
        max_height = std::cmp::max(max_height, height);

        let mut block = vec![vec![Default::default(); width]; height];

        let ac_huff = comp.get_ac_huff();
        let dc_huff = comp.get_dc_huff();

        let mut dc = last_dc[idx - 1];

        for y in 0..height {
            for x in 0..width {
                let code = decode_dct(&*dc_huff, dc, &*ac_huff, bs);
                dc = code[0];

                let arr = Array2::from_shape_vec((8, 8), code.to_vec()).unwrap();
                let dqt = &*comp.get_dqt().borrow_mut().table.clone();
                let arr = &arr * dqt;

                let mut result = [[0f32; 8]; 8];
                let zigzag = ZigZagScan::new(8);
                let (mut x1, mut y1) = (0, 0);
                for (x2, y2) in zigzag {
                    result[y2][x2] = arr[[y1, x1]] as f32;
                    x1 += 1;
                    if x1 == 8 {
                        x1 = 0;
                        y1 += 1;
                    }
                }

                block[y][x] = dct.idct2d(result);
            }
        }
        last_dc[idx - 1] = dc;
        mcu.push(Block {
            width: width,
            height: height,
            data: block,
        });
    }
    Ok((
        last_dc,
        MCU {
            data: mcu,
            width: max_width,
            height: max_height,
        },
    ))
}
