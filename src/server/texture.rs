use super::*;

pub struct Infinite {
    path: std::path::PathBuf,
    chunks: HashMap<Vec2<i32>, AutoSaved<Chunk>>,
}

impl Infinite {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        std::fs::create_dir_all(path.as_ref());
        Self {
            path: path.as_ref().to_owned(),
            chunks: default(),
        }
    }
    pub fn update(&mut self, update: Update) {
        match update {
            Update::Draw(pixels) => {
                for pixel in pixels {
                    let chunk_pos = pixel.position.map(|x| div_down(x, Chunk::SIZE as i32));
                    let mut chunk = self.get_chunk(chunk_pos).write();
                    let in_chunk =
                        (pixel.position - chunk_pos * Chunk::SIZE as i32).map(|x| x as usize);
                    chunk.pixels[in_chunk] = pixel.color;
                }
            }
        }
    }
    pub fn get(&mut self, rect: AABB<i32>) -> Matrix<Rgba<u8>> {
        let mut result =
            Matrix::filled_with(rect.size().map(|x| x as usize), Rgba::TRANSPARENT_BLACK);
        let chunks = AABB {
            x_min: div_down(rect.x_min, Chunk::SIZE as _),
            y_min: div_down(rect.y_min, Chunk::SIZE as _),
            x_max: div_up(rect.x_max, Chunk::SIZE as _),
            y_max: div_up(rect.y_max, Chunk::SIZE as _),
        };
        for chunk_x in chunks.x_min..chunks.x_max {
            for chunk_y in chunks.y_min..chunks.y_max {
                let chunk_pos = vec2(chunk_x, chunk_y);
                let chunk = self.get_chunk(chunk_pos).read();
                let needed = AABB {
                    x_min: (rect.x_min - chunk_x * Chunk::SIZE as i32).max(0),
                    y_min: (rect.y_min - chunk_y * Chunk::SIZE as i32).max(0),
                    x_max: (rect.x_max - chunk_x * Chunk::SIZE as i32).min(Chunk::SIZE as i32),
                    y_max: (rect.y_max - chunk_y * Chunk::SIZE as i32).min(Chunk::SIZE as i32),
                }
                .map(|x| x as usize);
                let origin = chunk_pos * Chunk::SIZE as i32 - rect.bottom_left();
                for x in needed.x_min..needed.x_max {
                    for y in needed.y_min..needed.y_max {
                        let in_chunk = vec2(x, y);
                        result[(origin + in_chunk.map(|x| x.try_into().unwrap()))
                            .map(|x| x.try_into().unwrap())] = chunk.pixels[in_chunk];
                    }
                }
            }
        }
        result
    }
    fn get_chunk(&mut self, chunk_pos: Vec2<i32>) -> &mut AutoSaved<Chunk> {
        self.chunks.entry(chunk_pos).or_insert_with(|| {
            AutoSaved::new(
                self.path
                    .join(format!("{}_{}.chunk", chunk_pos.x, chunk_pos.y)),
            )
        })
    }
}

#[derive(Serialize, Deserialize)]
struct Chunk {
    pixels: Matrix<Rgba<u8>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            pixels: Matrix::filled_with(vec2(Self::SIZE, Self::SIZE), Rgba::TRANSPARENT_BLACK),
        }
    }
}

impl Chunk {
    const SIZE: usize = 256;
}
