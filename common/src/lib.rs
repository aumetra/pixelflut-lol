use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hex_repr: Vec<u8>,
}

#[derive(Archive, Deserialize, Serialize)]
pub struct Frame {
    // layout: Y(X(pixel))
    pub data: Vec<Vec<Pixel>>,
}
