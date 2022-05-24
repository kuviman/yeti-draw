use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Texture {
    pub pixels: HashMap<Vec2<i32>, Color<u8>>,
}

impl Texture {
    pub fn load() -> Self {
        match std::fs::File::open("draw.save") {
            Ok(file) => bincode::deserialize_from(file).expect("Failed to load save"),
            Err(_) => Self { pixels: default() },
        }
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
    pub fn save(&self) {
        bincode::serialize_into(
            std::fs::File::create("draw.save").expect("Failed to create save"),
            self,
        )
        .expect("Failed to write save");
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

pub type UpdateId = u64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Update { id: UpdateId, update: Update },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    Initial(Texture),
    Update {
        your_id: Option<UpdateId>,
        update: Update,
    },
}
