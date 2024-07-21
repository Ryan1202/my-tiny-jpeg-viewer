use rustc_hash::FxHashMap;


pub struct Scan {
    pub components: FxHashMap<u8, ScanComponent>,
}

pub struct ScanComponent {
    frame_comp_id: u8,
    dc_id: u8,
    ac_id: u8,
}

impl ScanComponent {
    fn new(frame_comp_id: u8, dc_id: u8, ac_id: u8) -> Self {
        Self {
            frame_comp_id: frame_comp_id,
            dc_id: dc_id,
            ac_id: ac_id,
        }
    }

    pub fn get_id(&self) -> u8 {
        self.frame_comp_id
    }

    pub fn get_ac_id(&self) -> u8 {
        self.ac_id
    }

    pub fn get_dc_id(&self) -> u8 {
        self.dc_id
    }
}

impl Scan {
    pub fn new(data: Vec<u8>) -> Self {
        let mut components = FxHashMap::default();
        let comp_nr = data[0] as usize;

        for i in 0..comp_nr {
            let frame_comp_id = data[1 + i * 2 + 0];
            let dc_id = data[1 + i * 2 + 1] >> 4;
            let ac_id = data[1 + i * 2 + 1] & 0x0f;
            let comp = ScanComponent::new(frame_comp_id, dc_id, ac_id);
            components.insert(comp.get_id(), comp);
        }

        Self {
            components: components,
        }
    }
}
