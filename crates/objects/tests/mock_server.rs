use axum::{
    Router,
    body::Body,
    extract::Query,
    http::{Response, StatusCode},
    routing::get,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Deserialize)]
struct MaterialQuery {
    material_id: Uuid,
}

pub const MATERIAL_1: Uuid = Uuid::from_u128(0xa7088dd09776456cb88a9bad24a3a12c);
pub const MATERIAL_2: Uuid = Uuid::from_u128(0xed2b483cc235562b80b773703a9f3f4d);

/// create a mock asset server to test download
pub async fn mock_asset_server() -> String {
    let app = Router::new().route(
        "/",
        get(|Query(params): Query<MaterialQuery>| async move {
            let path = format!("tests/data/materials/{}.json", params.material_id);

            let data = tokio::fs::read(path).await.unwrap();

            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(data))
                .unwrap()
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{address}")
}
