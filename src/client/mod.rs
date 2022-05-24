use super::*;

mod texture;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

struct ReversibleUpdate {
    forward: Update,
    backward: Update,
}

pub struct Client {
    geng: Geng,
    connection: Connection,
    state: texture::Infinite,
    framebuffer_size: Vec2<usize>,
    camera: geng::Camera2d,
    stroke: Option<Stroke>,
    color: Color<f32>,
    brush_size: f32,
    camera_drag_start: Option<Vec2<f32>>,
    next_update_id: UpdateId,
    unconfirmed_updates: Vec<(UpdateId, ReversibleUpdate)>,
}

struct Stroke {
    pixels: HashSet<Vec2<i32>>,
    texture: texture::Infinite,
    last_position: Vec2<f32>,
}

impl Client {
    pub fn new(geng: &Geng, initial_state: Texture, connection: Connection) -> Self {
        Self {
            geng: geng.clone(),
            connection,
            state: texture::Infinite::from(geng, initial_state),
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
            next_update_id: 0,
            unconfirmed_updates: default(),
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
                        stroke.texture.update(Update::Draw(vec![Pixel {
                            position: vec2(x, y),
                            color: self.color.convert(),
                        }]));
                    }
                }
            }
            stroke.last_position = position;
        }
    }
    fn update(&mut self, update: Update) {
        let id = self.next_update_id;
        self.next_update_id += 1;
        let backward = self.state.update(update.clone()); // TODO: no clone
        self.unconfirmed_updates.push((
            id,
            ReversibleUpdate {
                forward: update.clone(), // TODO: no clone
                backward,
            },
        ));
        self.connection.send(ClientMessage::Update { id, update });
    }
}

impl geng::State for Client {
    fn update(&mut self, delta_time: f64) {
        let new_messages: Vec<ServerMessage> = self.connection.new_messages().collect();
        if !new_messages.is_empty() {
            let last_confirmed = new_messages
                .iter()
                .filter_map(|message| match message {
                    ServerMessage::Update { your_id, .. } => *your_id,
                    _ => None,
                })
                .max();
            let mut redo = Vec::new();
            while let Some((id, update)) = self.unconfirmed_updates.pop() {
                self.state.update(update.backward.clone()); // TODO: no clone
                if Some(id) == last_confirmed {
                    break;
                }
                redo.push((id, update));
            }
            while let Some((id, update)) = self.unconfirmed_updates.pop() {
                self.state.update(update.backward);
            }
            for message in new_messages {
                match message {
                    ServerMessage::Initial(_) => unreachable!(),
                    ServerMessage::Update { your_id, update } => {
                        self.state.update(update);
                    }
                }
            }
            while let Some((id, update)) = redo.pop() {
                self.state.update(update.forward.clone()); // TODO: no clone
                self.unconfirmed_updates.push((id, update));
            }
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::WHITE), None);
        self.state.draw(framebuffer, &self.camera);
        if let Some(stroke) = &self.stroke {
            stroke.texture.draw(framebuffer, &self.camera);
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            // Camera controls
            geng::Event::Wheel { delta } => {
                let prev_pos = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    self.geng.window().cursor_position().map(|x| x as f32),
                );
                self.camera.fov =
                    (self.camera.fov * 1.01f32.powf(-delta as f32)).clamp(100.0, 3000.0);
                let current_pos = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    self.geng.window().cursor_position().map(|x| x as f32),
                );
                self.camera.center += prev_pos - current_pos;
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
                    texture: texture::Infinite::new(&self.geng),
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
                    self.update(Update::Draw(
                        stroke
                            .pixels
                            .into_iter()
                            .map(|position| Pixel {
                                position,
                                color: self.color.convert(),
                            })
                            .collect(),
                    ));
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
