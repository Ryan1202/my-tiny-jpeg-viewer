use std::{collections::HashMap, error, fs::File, io::BufReader, rc::Rc, vec};

use chroma::ycbcr2rgb;
use dct::DCT;
use mcu::decode_blocks;

use crate::{bitstream::BitStream, component::{frame::Frame, Component}};

pub mod mcu;
pub mod dct;
mod chroma;

pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

pub fn decode_mcu(
    last_dc: Vec<isize>,
    comps: HashMap<u8, Rc<Component>>,
    bs: &mut BitStream<BufReader<File>>,
    dct: &DCT,
) -> Result<(Vec<isize>, Vec<u8>), Box<dyn error::Error>> {
    let (_last_dc, mcu) = decode_blocks(last_dc, comps, bs, dct).unwrap();
    
    let block_y = &mcu.data[0];
    let block_cb = &mcu.data[1];
    let block_cr = &mcu.data[2];
    let mut buffer = vec![0; (mcu.width * 8) * (mcu.height * 8) * 4];

    for y0 in 0..mcu.height {
        for x0 in 0..mcu.width {
            let block_base = ((y0 * (8 * mcu.width) * 8) + (x0 * 8)) * 4;

            let idx_y = Coordinate { x: x0 % block_y.width, y: y0 % block_y.height};
            let idx_cb = Coordinate { x: x0 % block_cb.width, y: y0 % block_cb.height};
            let idx_cr = Coordinate { x: x0 % block_cr.width, y: y0 % block_cr.height};

            let y = &block_y.data[idx_y.y][idx_y.x];
            let cb = &block_cb.data[idx_cb.y][idx_cb.x];
            let cr = &block_cr.data[idx_cr.y][idx_cr.x];

            let y_factor_x = mcu.width / block_y.width;
            let y_factor_y = mcu.height / block_y.height;
            let cb_factor_x = mcu.width / block_cb.width;
            let cb_factor_y = mcu.height / block_cb.height;
            let cr_factor_x = mcu.width / block_cr.width;
            let cr_factor_y = mcu.height / block_cr.height;

            let y_off_x = x0 * 8 / y_factor_x;
            let y_off_y = y0 * 8 / y_factor_y;
            let cb_off_x = x0 * 8 / cb_factor_x;
            let cb_off_y = y0 * 8 / cb_factor_y;
            let cr_off_x = x0 * 8 / cr_factor_x;
            let cr_off_y = y0 * 8 / cr_factor_y;

            for y1 in 0..8 {
                let mut offset = block_base + (y1 * mcu.width * 8) * 4;
                for x1 in 0..8 {
                    let (r, g, b) = ycbcr2rgb(
                        y[(y_off_y + y1/y_factor_y)%8][(y_off_x + x1/y_factor_x)%8],
                        cb[(cb_off_y + y1/cb_factor_y)%8][(cb_off_x + x1/cb_factor_x)%8],
                        cr[(cr_off_y + y1/cr_factor_y)%8][(cr_off_x + x1/cr_factor_x)%8]);
                    buffer[offset + 0] = r;
                    buffer[offset + 1] = g;
                    buffer[offset + 2] = b;
                    buffer[offset + 3] = 0xff;
                    offset += 4;
                }
            }
        }
    }
    Ok((_last_dc, buffer))
}

pub fn decode_image(
    frame: &Frame,
    comps: HashMap<u8, Rc<Component>>,
    bs: &mut BitStream<BufReader<File>>,
    restart_interval: Option<u16>,
    dct: DCT,
) -> Result<Vec<u8>, Box<dyn error::Error>> {
    let mut last_dc = vec![0isize; 3];

    let width = frame.get_width() as usize;
    let height = frame.get_height() as usize;

    let mut buffer = vec![Default::default();width*height*4];
    
    let mut mcu_width = 0;
    let mut mcu_height = 0;
    for (id, comp) in frame.components.iter() {
        if comp.get_factor_x() as usize > mcu_width {
            mcu_width = comp.get_factor_x() as usize;
        }
        if comp.get_factor_y() as usize > mcu_height {
            mcu_height = comp.get_factor_y() as usize;
        }
    }
    mcu_width *= 8;
    mcu_height *= 8;

    let x_cnt = (width + mcu_width - 1) / mcu_width;
    let y_cnt = (height + mcu_height - 1) / mcu_height;

    let mut cnt = 0;

    for y1 in 0..y_cnt {
        for x1 in 0..x_cnt {
            let mcu;
            (last_dc, mcu) = decode_mcu(last_dc, comps.clone(), bs, &dct)?;
            let mcu_base = ((y1 * mcu_height * width) + (x1 * mcu_width)) * 4;
            for y2 in 0..mcu_height {
                if y1*mcu_height + y2 >= height {
                    break;
                }
                let mut offset1 = mcu_base + y2 * width * 4;
                let mut offset2 = y2 * mcu_width * 4;
                for x2 in 0..mcu_width {
                    if x1*mcu_width + x2 >= width {
                        break;
                    }
                    buffer[offset1 + 0] = mcu[offset2 + 0];
                    buffer[offset1 + 1] = mcu[offset2 + 1];
                    buffer[offset1 + 2] = mcu[offset2 + 2];
                    buffer[offset1 + 3] = mcu[offset2 + 3];
                    offset1 += 4;
                    offset2 += 4;
                }
            }
            if let Some(ri) = restart_interval {
                cnt += 1;
                if cnt >= ri {
                    bs.align_byte();
                    bs.read(8).unwrap();
                    last_dc = [0].repeat(3);
                    cnt = 0;
                }
            }
        }
    }

    Ok(buffer)
}