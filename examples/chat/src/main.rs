use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_ws::{Channel, Payload, Socket, Topic, WebSocket};
use serde_json::json;
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

    let user_socket = WebSocket::<UserSocket>::new("/socket")
        .connect(socket_connect)
        .id(socket_id)
        .channel("room:*", room_channel);

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

async fn socket_connect(params: serde_json::Value, _socket: Socket) -> impl IntoResponse {
    // 通过 url 参数传递 token，可以在这里进行 token 验证，如果验证失败可以返回错误信息，然后断开连接
    // 验证通过后可以将用户信息存储到 socket 的 assigns 中，方便后续使用
    println!("token: {:?}", params);

    "ok"
}

async fn socket_id(_socket: Socket) -> Option<String> {
    // 规范 socket id，可以根据业务需求生成唯一的 socket id
    // 如果返回 None，则会使用默认的 socket id
    // 可以作为 broadcast_from 的 user_id 参数
    Some("user:1".to_string())
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct User {
    id: i32,
    name: String,
}

async fn room_join(
    topic: Topic,
    payload: Payload,
    socket: Socket,
) -> anyhow::Result<serde_json::Value> {
    // 加入房间时可以进行权限验证，如果验证失败可以返回错误信息，然后断开连接
    println!("room_join: {:?}, {:?}", topic, payload);

    let mut socket = socket.lock().await;
    // assigns 可以存储任意类型的数据
    socket.assigns.insert::<i32>("user_id", 1);
    socket.assigns.insert::<User>(
        "user",
        User {
            id: 1,
            name: "test".to_string(),
        },
    );

    // 返回错误信息将会导致连接失败，并返回错误信息给客户端，handler函数也会如此
    // Err(anyhow::anyhow!(json!({"reason": "auth failed"})))

    // 连接成功，信息将发送给客户端
    Ok(json!({"user_id": 1}))
}

async fn handler_test(payload: Payload, socket: Socket) -> anyhow::Result<&'static str> {
    // 事件处理函数，可以在这里处理业务逻辑，然后返回结果
    println!("handler_test: {:?}", payload);

    let socket = socket.lock().await;
    let user_id = socket.assigns.get::<i32>("user_id").unwrap();
    let user = socket.assigns.get::<User>("user").unwrap();
    println!("user_id: {:?}", user_id);
    println!("user: {:?}", user);
    // 主动推送事件给当前用户
    socket
        .push(
            "push_event",
            Ok(serde_json::json!({"data": "user push event1"})),
        )
        .await?;

    // 广播事件给当前用户所在的房间
    socket
        .broadcast(
            "broadcast_event",
            Ok(serde_json::json!({"data": "user broadcast event1"})),
        )
        .await?;

    // 广播事件给当前用户所在的房间，但不包括当前用户
    socket
        .broadcast_from(
            "1",
            "broadcast_from_event",
            Ok(serde_json::json!({"data": "user broadcast from event1"})),
        )
        .await?;

    // 返回信息给客户端
    Ok("test")
}

async fn handler_test2(payload: Payload, _socket: Socket) -> anyhow::Result<()> {
    println!("handler_test2: {:?}", payload);

    // 广播事件给所有用户，可以在http业务逻辑中调用
    WebSocket::<UserSocket>::broadcast(
        "room:1",
        "websocket_broadcast_event",
        Ok(serde_json::json!({"data": "broadcast event2"})),
    )
    .await?;

    // 广播事件给所有用户，但不包括指定的用户，可以在http业务逻辑中调用
    WebSocket::<UserSocket>::broadcast_from(
        "1",
        "room:1",
        "websocket_broadcast_from_event",
        Ok(serde_json::json!({"data": "broadcast event2"})),
    )
    .await?;

    // 如果返回unit，不会将信息发送给客户端
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../templates/index.html"))
}
