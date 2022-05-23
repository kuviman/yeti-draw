use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Texture {
    pub pixels: HashMap<Vec2<i32>, Color<u8>>,
}

impl Texture {
    pub fn new() -> Self {
        Self { pixels: default() }
    }
    pub fn update(&mut self, update: Update) {
        match update {
            Update::Draw(pixels) => {
                for pixel in pixels {
                    self.pixels.insert(pixel.position, pixel.color);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pixel {
    pub position: Vec2<i32>,
    pub color: Color<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Update {
    Draw(Vec<Pixel>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Update(Update),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    Initial(Texture),
    Update(Update),
}
