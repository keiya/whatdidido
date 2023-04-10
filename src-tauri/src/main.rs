use axum::{
    body::Body,
    routing::post,
    Router,
};
use axum_macros::debug_handler;
use axum::http::{StatusCode, Response};
use serde::Deserialize;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use core::convert::Infallible;
//use std::convert::Infallible;
use std::error::Error;
use tokio::runtime::Handle;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Deserialize)]
struct TabTitle {
    title: String,
}

#[debug_handler]
async fn save_title(tab_title: axum::extract::Json<TabTitle>) -> Result<Response<Body>, Infallible> {
    // タブのタイトルを保存するロジックをここに実装します
    // 例: データベースに保存する
    println!("Received tab title: {}", tab_title.title);

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .expect("Failed to build the response");

    Ok(response)
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// fn main() {
//     tauri::Builder::default()
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }

fn main() -> Result<(), Box<dyn Error>> {
    // Tokio Runtime を作成
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let axum_handle = runtime.spawn( async {
        let app = Router::new()
            .route("/api/save-title", post(save_title))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .into_inner(),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Tauri Builder の作成
    let tauri_builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");

    // ここで、メインスレッド上で Tauri を実行
    let tauri_handle = Handle::current().spawn(async move {
        tauri_builder.run().await;
    });

    // 両方のタスクが完了するのを待つ
    runtime.block_on(async move {
        let _ = tokio::join!(axum_handle, tauri_handle);
    });

    Ok(())
}

