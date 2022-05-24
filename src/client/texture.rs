use super::*;

pub struct Infinite {
    geng: Geng,
    pixels: HashMap<Vec2<i32>, Color<u8>>,
    chunks: HashMap<Vec2<i32>, Chunk>,
}

struct Chunk {
    ugli: ugli::Texture,
}

impl Infinite {
    const CHUNK_SIZE: usize = 64;
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            pixels: default(),
            chunks: HashMap::new(),
        }
    }
    pub fn upload(&mut self, position: Vec2<i32>, data: Matrix<Color<u8>>) {
        assert_eq!(position.x % Self::CHUNK_SIZE as i32, 0);
        assert_eq!(position.y % Self::CHUNK_SIZE as i32, 0);
        let chunk_pos = position / Self::CHUNK_SIZE as i32;
        for x in 0..data.size().x {
            for y in 0..data.size().y {
                self.pixels
                    .insert(position + vec2(x, y).map(|x| x as i32), data[vec2(x, y)]);
            }
        }
        self.chunks.insert(
            chunk_pos,
            Chunk {
                ugli: {
                    let mut texture = ugli::Texture::new_with(
                        self.geng.ugli(),
                        vec2(Self::CHUNK_SIZE, Self::CHUNK_SIZE),
                        |pos| data[pos].convert(),
                    );
                    texture.set_filter(ugli::Filter::Nearest);
                    texture
                },
            },
        );
    }
    pub fn update(&mut self, update: Update) -> Update {
        match update {
            Update::Draw(pixels) => {
                let mut reverse = Vec::with_capacity(pixels.len());
                for pixel in pixels {
                    reverse.push(Pixel {
                        position: pixel.position,
                        color: self
                            .pixels
                            .get(&pixel.position)
                            .copied()
                            .unwrap_or(Color::TRANSPARENT_BLACK),
                    });
                    self.pixels.insert(pixel.position, pixel.color);

                    let chunk_pos = pixel.position.map(|x| div_down(x, Self::CHUNK_SIZE as _));
                    let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| Chunk {
                        ugli: {
                            let mut texture = ugli::Texture::new_with(
                                self.geng.ugli(),
                                vec2(Self::CHUNK_SIZE, Self::CHUNK_SIZE),
                                |_| Color::TRANSPARENT_BLACK,
                            );
                            texture.set_filter(ugli::Filter::Nearest);
                            texture
                        },
                    });
                    let chunk = match self.chunks.get_mut(&chunk_pos) {
                        Some(chunk) => chunk,
                        None => continue,
                    };
                    let pixel_position =
                        (pixel.position - chunk_pos * Self::CHUNK_SIZE as i32).map(|x| x as usize);
                    chunk
                        .ugli
                        .sub_image(pixel_position, vec2(1, 1), pixel.color.as_slice());
                }
                Update::Draw(reverse)
            }
        }
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
    ) -> Option<AABB<i32>> {
        let aabb = camera
            .view_area(framebuffer.size().map(|x| x as f32))
            .bounding_box();
        let chunks = AABB {
            x_min: (aabb.x_min as f32 / Self::CHUNK_SIZE as f32).floor() as i32,
            y_min: (aabb.y_min as f32 / Self::CHUNK_SIZE as f32).floor() as i32,
            x_max: (aabb.x_max as f32 / Self::CHUNK_SIZE as f32).ceil() as i32,
            y_max: (aabb.y_max as f32 / Self::CHUNK_SIZE as f32).ceil() as i32,
        };
        let mut request = None;
        for chunk_x in chunks.x_min..=chunks.x_max {
            for chunk_y in chunks.y_min..=chunks.y_max {
                let chunk_pos = vec2(chunk_x, chunk_y);
                if let Some(chunk) = self.chunks.get(&chunk_pos) {
                    self.geng.draw_2d(
                        framebuffer,
                        camera,
                        &draw_2d::TexturedQuad::new(
                            AABB::point(chunk_pos.map(|x| x as f32) * Self::CHUNK_SIZE as f32)
                                .extend_positive(chunk.ugli.size().map(|x| x as f32)),
                            &chunk.ugli,
                        ),
                    );
                } else {
                    request = Some(
                        AABB::point(chunk_pos * Self::CHUNK_SIZE as i32).extend_positive(vec2(
                            Self::CHUNK_SIZE as i32,
                            Self::CHUNK_SIZE as i32,
                        )),
                    );
                }
            }
        }
        request
    }
}
