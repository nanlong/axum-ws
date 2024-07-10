use axum::{response::Html, routing::get, Router};
use axum_ws::{Channel, Payload, Socket, Topic, WebSocket};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Default)]
struct UserSocket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let room_channel = Channel::new()
        .join(room_join)
        .handler("test", handler_test)
        .handler("test2", handler_test2);

    let user_socket = WebSocket::<UserSocket>::new("/socket").channel("room:*", room_channel);

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/assets", ServeDir::new("priv/static/assets"))
        .merge(user_socket)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!(
        "listening on http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn room_join(topic: Topic, payload: Payload, socket: Socket) -> anyhow::Result<&'static str> {
    println!("room_join: {:?}, {:?}", topic, payload);

    let mut socket = socket.lock().await;
    socket.assigns.insert::<i32>("user_id", 1);

    Ok("ok")
}

async fn handler_test(payload: Payload, socket: Socket) -> anyhow::Result<&'static str> {
    println!("handler_test: {:?}", payload);

    let socket = socket.lock().await;
    let user_id = socket.assigns.get::<i32>("user_id").unwrap();
    println!("user_id: {:?}", user_id);

    Ok("test")
}

async fn handler_test2(payload: Payload, _socket: Socket) {
    println!("handler_test2: {:?}", payload);
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../templates/index.html"))
}
