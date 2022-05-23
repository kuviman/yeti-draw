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
    brush_size: f32,
    camera_drag_start: Option<Vec2<f32>>,
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
            brush_size: 1.0,
            color: Color::BLACK,
            camera_drag_start: None,
        }
    }
    fn screen_to_world(&self, position: Vec2<f64>) -> Vec2<f32> {
        self.camera
            .screen_to_world(
                self.framebuffer_size.map(|x| x as f32),
                position.map(|x| x as f32),
            )
            .map(|x| x.round())
    }
    fn mouse_move(&mut self, position: Vec2<f32>) {
        fn distance(a: Vec2<f32>, b: Vec2<f32>, p: Vec2<f32>) -> f32 {
            if Vec2::dot(p - a, b - a) <= 0.0 {
                return (p - a).len();
            }
            if Vec2::dot(p - b, a - b) <= 0.0 {
                return (p - b).len();
            }
            Vec2::skew(p - a, (b - a).normalize_or_zero()).abs()
        }

        if let Some(stroke) = &mut self.stroke {
            let a = position;
            let b = stroke.last_position;
            let aabb = AABB::points_bounding_box([a, b]).extend_uniform(self.brush_size);
            for x in aabb.x_min.floor() as i32..=aabb.x_max.ceil() as i32 {
                for y in aabb.y_min.floor() as i32..=aabb.y_max.ceil() as i32 {
                    let p = vec2(x as f32 + 0.5, y as f32 + 0.5);
                    if distance(a, b, p) < self.brush_size {
                        stroke.pixels.insert(vec2(x, y));
                    }
                }
            }
            stroke.last_position = position;
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
            // Camera controls
            geng::Event::Wheel { delta } => {
                self.camera.fov =
                    (self.camera.fov * 1.01f32.powf(-delta as f32)).clamp(100.0, 3000.0);
            }
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Middle,
            } => {
                self.camera_drag_start = Some(self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                ));
            }
            geng::Event::MouseMove { position, .. } => {
                if let Some(start) = self.camera_drag_start {
                    let current_pos = self.camera.screen_to_world(
                        self.framebuffer_size.map(|x| x as f32),
                        position.map(|x| x as f32),
                    );
                    self.camera.center += start - current_pos;
                }
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Middle,
                ..
            } => self.camera_drag_start = None,
            _ => {}
        }

        match event {
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                let position = self.screen_to_world(position);
                self.stroke = Some(Stroke {
                    pixels: default(),
                    last_position: position,
                });
                self.mouse_move(position);
            }
            geng::Event::MouseMove { position, delta: _ } => {
                let position = self.screen_to_world(position);
                self.mouse_move(position);
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let position = self.screen_to_world(position);
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
                geng::Key::PageUp => {
                    self.brush_size = (self.brush_size + 0.5).min(10.0);
                }
                geng::Key::PageDown => {
                    self.brush_size = (self.brush_size - 0.5).max(0.5);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
