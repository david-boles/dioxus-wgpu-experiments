use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    routing::get,
    Router,
};

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

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
