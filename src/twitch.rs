use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

enum State {
    SetupRequired,
    Disconnected,
    Connected(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        mpsc::Receiver<Message>,
    ),
}

pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(Message),
}

pub struct Connection(mpsc::Sender<Message>);

pub enum Message {
    Connected,
    Disconnected,
    User(String),
}
