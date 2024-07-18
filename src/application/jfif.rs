#[derive(Debug)]
pub struct JFIF {
    version: String,
    units: u8,
    x_density: u16,
    y_density: u16,
    x_thumbnail: u8,
    y_thumbnail: u8,
    nail_data: Vec<u8>,
}

impl JFIF {
    pub fn new(data: &Vec<u8>) -> Self {
        let x = data[..5].to_vec();
        let y = vec![b'J', b'F', b'I', b'F', 0];
        if x != y {
            panic!("Invalid JFIF file format");
        }

        let version = data[5].to_string() + "." + &data[6].to_string();
        let units = data[7];
        let xd = u16::from_be_bytes([data[8], data[9]]);
        let yd = u16::from_be_bytes([data[10], data[11]]);
        let xt = data[12];
        let yt = data[13];
        let nail_data = data[14..].to_vec();
        JFIF {
            version: version,
            units: units,
            x_density: xd,
            y_density: yd,
            x_thumbnail: xt,
            y_thumbnail: yt,
            nail_data: nail_data,
        }
    }
}
