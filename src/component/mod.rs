use std::rc::Rc;

use frame::Frame;
use rustc_hash::FxHashMap;
use scan::Scan;

use crate::{dht::HuffmanTable, dqt::Dqt};

pub mod frame;
pub mod scan;

pub struct Component {
    id: u8,
    factor_x: u8,
    factor_y: u8,
    quantization: Rc<Dqt>,
    dc_huffman_table: Rc<HuffmanTable>,
    ac_huffman_table: Rc<HuffmanTable>,
}

#[derive(Debug)]
pub enum ComponentErrorType {
    InvalidQuantizationId(u8),
    InvalidFrameId(u8),
}

impl Component {
    pub fn new(
        frame: &Frame,
        dqt_map: FxHashMap<u8, Rc<Dqt>>,
        dc_map: FxHashMap<u8, Rc<HuffmanTable>>,
        ac_map: FxHashMap<u8, Rc<HuffmanTable>>,
        scan_map: Scan,
    ) -> Result<FxHashMap<u8, Rc<Self>>, ComponentErrorType> {
        let mut map = FxHashMap::default();

        for (id, comp) in scan_map.components {
            let dc_huff = dc_map[&comp.get_dc_id()].clone();
            let ac_huff = ac_map[&comp.get_ac_id()].clone();
            let fcomp = match frame.components.get(&id) {
                Some(fcomp) => fcomp,
                None => {
                    return Err(ComponentErrorType::InvalidFrameId(id));
                }
            };
            let qid = fcomp.get_qid();
            let qt = match dqt_map.get(&qid) {
                Some(qt) => (*qt).clone(),
                None => {
                    return Err(ComponentErrorType::InvalidQuantizationId(qid));
                }
            };
            let comp = Self {
                id: id,
                factor_x: fcomp.get_factor_x(),
                factor_y: fcomp.get_factor_y(),
                quantization: qt,
                dc_huffman_table: dc_huff.clone(),
                ac_huffman_table: ac_huff.clone(),
            };
            map.insert(comp.id, Rc::new(comp));
        }

        Ok(map)
    }

    pub fn get_factor_x(&self) -> u8 {
        self.factor_x
    }

    pub fn get_factor_y(&self) -> u8 {
        self.factor_y
    }

    pub fn get_ac_huff(&self) -> Rc<HuffmanTable> {
        self.ac_huffman_table.clone()
    }

    pub fn get_dc_huff(&self) -> Rc<HuffmanTable> {
        self.dc_huffman_table.clone()
    }

    pub fn get_dqt(&self) -> Rc<Dqt> {
        self.quantization.clone()
    }
}
