use axum::{
    body::Body,
    extract::{ws::WebSocket, WebSocketUpgrade},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    routing::get,
    Router,
};

mod resample;

include!(concat!(env!("OUT_DIR"), "/client.rs"));

#[tokio::main]
async fn main() {
    let mut app = Router::new();
    for (path, mime_type, contents) in CLIENT_FILES {
        app = app.route(
            path,
            get(|| async {
                let mut response = Response::builder().status(StatusCode::OK);
                if let Some(mime_type) = mime_type.as_ref() {
                    response = response.header(header::CONTENT_TYPE, *mime_type)
                }
                response.body(Body::from(*contents)).unwrap()
            }),
        )
    }

    app = app.route(
        "/signal",
        get(|ws: WebSocketUpgrade| async { ws.on_upgrade(signal_handler) }),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn signal_handler(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        dbg!(msg);
    }
}
