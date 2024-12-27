use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Deserialize, Serialize)]
pub struct Pixel<'a> {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(borrow)]
    pub hex_repr: Cow<'a, [u8]>,
}

#[derive(Deserialize, Serialize)]
pub struct PixelRef<'a> {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hex_repr: &'a [u8],
}

#[derive(Deserialize, Serialize)]
pub struct FrameRef<'a> {
    #[serde(borrow)]
    pub data: Vec<Vec<Pixel<'a>>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Frame<'a> {
    // layout: Y(X(pixel))
    #[serde(borrow)]
    pub data: Cow<'a, [Cow<'a, [Pixel<'a>]>]>,
}
