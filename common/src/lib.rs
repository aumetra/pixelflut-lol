use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Clone, Deserialize, Serialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hex_repr: Vec<u8>,
    pub draw: bool,
}

#[derive(Archive, Clone, Deserialize, Serialize)]
pub struct Frame {
    // layout: Y(X(pixel))
    pub data: Vec<Vec<Pixel>>,
}
