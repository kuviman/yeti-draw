use super::*;

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

pub struct Client {
    geng: Geng,
    connection: Connection,
    state: State,
    framebuffer_size: Vec2<usize>,
    camera: geng::Camera2d,
    stroke: Option<Stroke>,
}

struct Stroke {
    pixels: HashSet<Vec2<i32>>,
    last_position: Vec2<f32>,
}

impl Client {
    pub fn new(geng: &Geng, initial_state: State, connection: Connection) -> Self {
        Self {
            geng: geng.clone(),
            connection,
            state: initial_state,
            framebuffer_size: vec2(1, 1),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 100.0,
            },
            stroke: None,
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
        for (&position, &color) in &self.state.image {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(position.map(|x| x as f32)).extend_positive(vec2(1.0, 1.0)),
                    color.convert(),
                ),
            );
        }
        if let Some(stroke) = &self.stroke {
            for &position in &stroke.pixels {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        AABB::point(position.map(|x| x as f32)).extend_positive(vec2(1.0, 1.0)),
                        Color::BLACK,
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
                                color: Color::BLACK,
                            })
                            .collect(),
                    )));
                }
            }
            _ => {}
        }
    }
}
