use actix_web::web::{Bytes, Data};
use actix_web::Error;

use futures::stream::Stream;
use serde_json::Value;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{interval, Duration};

use std::pin::Pin;
use std::sync::Mutex;

use super::poll_model::Poll;

pub struct Broadcaster {
    clients: Vec<Sender<Bytes>>,
}

impl Broadcaster {
    pub fn create() -> Data<Mutex<Self>> {
        let me = Data::new(Mutex::new(Broadcaster::new()));
        // let me_clone = me.clone();
        // Use tokio spawn for the ping task
        // tokio::spawn(async move {
        //     Broadcaster::spawn_ping(me_clone).await;
        // });
        me
    }

    pub fn new() -> Self {
        Broadcaster {
            clients: Vec::new(),
        }
    }

    pub async fn spawn_ping(me: Data<Mutex<Self>>) {
        let mut interval = interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let mut broadcaster = me.lock().unwrap();
            broadcaster.remove_stale_clients();
        }
    }

    pub fn remove_stale_clients(&mut self) {
        self.clients.retain(|client| {
            client
                .clone()
                .try_send(Bytes::from("data: ping\n\n"))
                .is_ok()
        });
    }

    pub fn new_client(&mut self) -> Client {
        let (tx, rx) = channel(100);

        // Send initial connection message
        let _ = tx.clone().try_send(Bytes::from("data: connected\n\n"));

        self.clients.push(tx);
        Client(rx)
    }

    pub fn send(&self, msg: &str) {
        let msg = Bytes::from(format!("data: {}\n\n", msg));

        for client in &self.clients {
            let _ = client.clone().try_send(msg.clone());
        }
    }

    pub fn send_updated_poll(&self, poll: &Poll) {
        let poll_json = serde_json::to_string(poll).unwrap();

        let msg = Bytes::from(format!("event: poll_updated\ndata: {}\n\n", poll_json));

        for client in &self.clients {
            let _ = client.clone().try_send(msg.clone());
        }
    }

    pub fn send_poll_results(&self, response: &Value) {
        let poll_json = response.to_string();

        let msg = Bytes::from(format!("event: poll_results\ndata: {}\n\n", poll_json));

        for client in &self.clients {
            let _ = client.clone().try_send(msg.clone());
        }
    }
}

// Wrap Receiver in own type with correct error handling
pub struct Client(Receiver<Bytes>);

impl Stream for Client {
    type Item = Result<Bytes, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            std::task::Poll::Ready(Some(item)) => std::task::Poll::Ready(Some(Ok(item))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

// // Example route handlers
// async fn create_client(broadcaster: Data<Mutex<Broadcaster>>) -> impl Responder {
//     let mut broadcaster = broadcaster.lock().unwrap();
//     let client = broadcaster.new_client();

//     HttpResponse::Ok()
//         .content_type("text/event-stream")
//         .streaming(client)
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     let broadcaster = Broadcaster::create();

//     HttpServer::new(move || {
//         App::new()
//             .app_data(broadcaster.clone())
//             .route("/events", web::get().to(create_client))
//             .route("/send", web::post().to(send_message))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }
