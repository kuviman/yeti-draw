use super::*;

pub struct Infinite {
    geng: Geng,
    inner: Texture,
    position: Vec2<i32>,
    ugli: ugli::Texture,
}

impl Infinite {
    pub fn new(geng: &Geng, texture: Texture) -> Self {
        if texture.pixels.is_empty() {
            return Self {
                geng: geng.clone(),
                inner: texture,
                position: vec2(0, 0),
                ugli: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| {
                    Color::TRANSPARENT_BLACK
                }),
            };
        }
        let aabb = AABB::points_bounding_box(texture.pixels.keys().copied());
        let mut ugli = ugli::Texture::new_with(
            geng.ugli(),
            aabb.size().map(|x| (x + 1) as usize),
            |position| {
                let position = position.map(|x| x as i32) + aabb.bottom_left();
                match texture.pixels.get(&position) {
                    Some(color) => color.convert(),
                    None => Color::TRANSPARENT_BLACK,
                }
            },
        );
        ugli.set_filter(ugli::Filter::Nearest);
        Self {
            geng: geng.clone(),
            inner: texture,
            position: aabb.bottom_left(),
            ugli,
        }
    }
    pub fn update(&mut self, update: Update) {
        self.inner.update(update);
        *self = Self::new(&self.geng, self.inner.clone());
    }
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &impl geng::AbstractCamera2d) {
        self.geng.draw_2d(
            framebuffer,
            camera,
            &draw_2d::TexturedQuad::new(
                AABB::point(self.position.map(|x| x as f32))
                    .extend_positive(self.ugli.size().map(|x| x as f32)),
                &self.ugli,
            ),
        );
    }
}
