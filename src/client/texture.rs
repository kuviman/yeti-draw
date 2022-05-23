use super::*;

pub struct Infinite {
    geng: Geng,
    chunks: HashMap<Vec2<i32>, Chunk>,
}

struct Chunk {
    ugli: ugli::Texture,
}

fn div_down<T: Num>(a: T, b: T) -> T {
    if a < T::ZERO {
        return -div_up(-a, b);
    }
    if b < T::ZERO {
        return -div_up(a, -b);
    }
    a / b
}

fn div_up<T: Num>(a: T, b: T) -> T {
    if a < T::ZERO {
        return -div_down(-a, b);
    }
    if b < T::ZERO {
        return -div_down(a, -b);
    }
    (a + b - T::ONE) / b
}

impl Infinite {
    const CHUNK_SIZE: usize = 64;
    pub fn new(geng: &Geng, texture: Texture) -> Self {
        let mut result = Self {
            geng: geng.clone(),
            chunks: HashMap::new(),
        };
        result.update(Update::Draw(
            texture
                .pixels
                .iter()
                .map(|(&position, &color)| Pixel { position, color })
                .collect(),
        ));
        result
    }
    pub fn update(&mut self, update: Update) {
        match update {
            Update::Draw(pixels) => {
                for pixel in pixels {
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
                    let pixel_position =
                        (pixel.position - chunk_pos * Self::CHUNK_SIZE as i32).map(|x| x as usize);
                    chunk
                        .ugli
                        .sub_image(pixel_position, vec2(1, 1), pixel.color.as_slice());
                }
            }
        }
    }
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &impl geng::AbstractCamera2d) {
        for (&position, chunk) in &self.chunks {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::TexturedQuad::new(
                    AABB::point(position.map(|x| x as f32) * Self::CHUNK_SIZE as f32)
                        .extend_positive(chunk.ugli.size().map(|x| x as f32)),
                    &chunk.ugli,
                ),
            );
        }
    }
}
