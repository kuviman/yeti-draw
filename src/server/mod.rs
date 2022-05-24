use super::*;

mod texture;

type ClientId = u64;
type ClientState = Box<dyn geng::net::Sender<ServerMessage>>;

struct ServerState {
    next_client_id: ClientId,
    clients: HashMap<ClientId, ClientState>,
    state: texture::Infinite,
}

impl ServerState {
    fn new() -> Self {
        Self {
            next_client_id: 0,
            clients: default(),
            state: texture::Infinite::new("save"),
        }
    }
    fn handle_message(&mut self, client_id: ClientId, message: ClientMessage) {
        match message {
            ClientMessage::Download { area } => {
                self.clients
                    .get_mut(&client_id)
                    .unwrap()
                    .send(ServerMessage::Download {
                        position: area.bottom_left(),
                        data: self.state.get(area),
                    });
            }
            ClientMessage::Update { id, update } => {
                for (&other_client_id, client) in &mut self.clients {
                    client.send(ServerMessage::Update {
                        your_id: if other_client_id == client_id {
                            Some(id)
                        } else {
                            None
                        },
                        update: update.clone(), // TODO: not clone
                    });
                }
                self.state.update(update);
            }
        }
    }
}

pub struct Server {
    state: Arc<Mutex<ServerState>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState::new())),
        }
    }
}

pub struct ClientConnection {
    id: ClientId,
    state: Arc<Mutex<ServerState>>,
}

impl geng::net::Receiver<ClientMessage> for ClientConnection {
    fn handle(&mut self, message: ClientMessage) {
        self.state.lock().unwrap().handle_message(self.id, message);
    }
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        self.state.lock().unwrap().clients.remove(&self.id);
    }
}

impl geng::net::server::App for Server {
    type Client = ClientConnection;
    type ServerMessage = ServerMessage;
    type ClientMessage = ClientMessage;
    fn connect(
        &mut self,
        mut sender: Box<dyn geng::net::Sender<ServerMessage>>,
    ) -> ClientConnection {
        let mut state = self.state.lock().unwrap();
        let id = state.next_client_id;
        state.next_client_id += 1;
        state.clients.insert(id, sender);
        ClientConnection {
            id,
            state: self.state.clone(),
        }
    }
}
