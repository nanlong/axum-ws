use axum::{http::StatusCode, response::IntoResponse};

fn test_into_response() -> impl IntoResponse {
    StatusCode::OK
}

fn main() {
    let response = test_into_response();
    println!("{:?}", response.into_response());
}
