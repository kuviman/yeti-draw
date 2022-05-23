use super::*;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

pub struct ClientTexture {
    geng: Geng,
    inner: Texture,
    position: Vec2<i32>,
    ugli: ugli::Texture,
}

impl ClientTexture {
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
}

pub struct Client {
    geng: Geng,
    connection: Connection,
    state: ClientTexture,
    framebuffer_size: Vec2<usize>,
    camera: geng::Camera2d,
    stroke: Option<Stroke>,
    color: Color<f32>,
}

struct Stroke {
    pixels: HashSet<Vec2<i32>>,
    last_position: Vec2<f32>,
}

impl Client {
    pub fn new(geng: &Geng, initial_state: Texture, connection: Connection) -> Self {
        Self {
            geng: geng.clone(),
            connection,
            state: ClientTexture::new(geng, initial_state),
            framebuffer_size: vec2(1, 1),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 100.0,
            },
            stroke: None,
            color: Color::BLACK,
        }
    }
}

impl geng::State for Client {
    fn update(&mut self, delta_time: f64) {
        for message in self.connection.new_messages() {
            match message {
                ServerMessage::Initial(_) => unreachable!(),
                ServerMessage::Update(update) => {
                    self.state.update(update);
                }
            }
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::WHITE), None);
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::TexturedQuad::new(
                AABB::point(self.state.position.map(|x| x as f32))
                    .extend_positive(self.state.ugli.size().map(|x| x as f32)),
                &self.state.ugli,
            ),
        );
        if let Some(stroke) = &self.stroke {
            for &position in &stroke.pixels {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        AABB::point(position.map(|x| x as f32)).extend_positive(vec2(1.0, 1.0)),
                        self.color,
                    ),
                );
            }
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                let position = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                self.stroke = Some(Stroke {
                    pixels: default(),
                    last_position: position,
                });
            }
            geng::Event::MouseMove { position, delta: _ } => {
                let position = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                if let Some(stroke) = &mut self.stroke {
                    let v = position - stroke.last_position;
                    let n = (v.len() * 2.0 + 1.0) as i32;
                    for i in 0..=n {
                        stroke.pixels.insert(
                            (stroke.last_position + v * i as f32 / n as f32)
                                .map(|x| x.floor() as i32),
                        );
                    }
                    stroke.last_position = position;
                }
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let position = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                if let Some(stroke) = self.stroke.take() {
                    self.connection.send(ClientMessage::Update(Update::Draw(
                        stroke
                            .pixels
                            .into_iter()
                            .map(|position| Pixel {
                                position,
                                color: self.color.convert(),
                            })
                            .collect(),
                    )));
                }
            }
            geng::Event::KeyDown { key } => match key {
                geng::Key::W => {
                    self.color = Color::WHITE;
                }
                geng::Key::B => {
                    self.color = Color::BLACK;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
