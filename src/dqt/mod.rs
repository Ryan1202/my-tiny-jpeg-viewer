use core::fmt;
use ndarray::{prelude::*, OwnedRepr, ShapeError};
use std::{collections::HashMap, rc::Rc};

use crate::zigzag::ZigZagScan;

#[derive(Debug)]
pub struct Dqt {
    id: u8,
    precision: u8,
    pub table: Rc<ArrayBase<OwnedRepr<isize>, Dim<[usize; 2]>>>,
}

impl fmt::Display for Dqt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "dqt[{}]({}bit):{{",
            self.id,
            if self.precision == 0 { 8 } else { 16 }
        )?;
        let zigzag = ZigZagScan::new(8);
        write!(f, "[")?;
        let mut i = 0;
        for (x, y) in zigzag.into_iter() {
            if self.table[[y, x]] < 10 {
                write!(f, " ")?;
            }
            write!(f, "{}, ", self.table[[y, x]])?;
            if i != 0 && i != 63 && i % 8 == 0 {
                write!(f, "],\n[")?;
            }
            i += 1;
        }
        write!(f, "]}}")
    }
}

impl Dqt {
    pub fn new(
        map: &mut HashMap<u8, Rc<Dqt>>,
        length: u16,
        data: Vec<u8>,
    ) -> Result<(), ShapeError> {
        let mut len = length - 2;
        let mut offset = 0;

        while len > 0 {
            let precision = data[offset + 0] >> 4;
            let num = data[offset + 0] & 0x0f;
            let mut table = Vec::with_capacity(64);
            for i in 0..64 {
                table.push(data[offset + i + 1] as isize);
            }
            let arr = Array2::from_shape_vec((8, 8), table)?;
            map.insert(
                num,
                Rc::new(Self {
                    id: num,
                    precision,
                    table: Rc::new(arr),
                }),
            );

            len -= 1 + 64 * (precision as u16 + 1);
            offset += 1 + 64 * (precision as usize + 1);
        }
        Ok(())
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
