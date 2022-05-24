use super::*;

type ClientId = u64;
type ClientState = Box<dyn geng::net::Sender<ServerMessage>>;

struct ServerState {
    next_client_id: ClientId,
    clients: HashMap<ClientId, ClientState>,
    state: Texture,
}

impl ServerState {
    fn new() -> Self {
        Self {
            next_client_id: 0,
            clients: default(),
            state: Texture::load(),
        }
    }
    fn handle_message(&mut self, client_id: ClientId, message: ClientMessage) {
        match message {
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
        self.state.save();
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
        sender.send(ServerMessage::Initial(state.state.clone())); // TODO: not clone
        let id = state.next_client_id;
        state.next_client_id += 1;
        state.clients.insert(id, sender);
        ClientConnection {
            id,
            state: self.state.clone(),
        }
    }
}
