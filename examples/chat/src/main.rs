use axum::{response::Html, routing::get, Router};
use axum_ws::WebSocket;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Default)]
struct UserSocket;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let user_socket = WebSocket::<UserSocket>::new("/socket");

    let app = Router::new()
        .route("/", get(index))
        .nest_service(
            "/assets",
            ServeDir::new(
                "/Users/jeff/Projects/workspace/axum-ws/examples/chat/priv/static/assets",
            ),
        )
        .merge(user_socket)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!(
        "listening on http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../templates/index.html"))
}
