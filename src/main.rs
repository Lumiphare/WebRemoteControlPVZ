pub mod webrtc;
pub mod control;

use anyhow::Result;
// use serde::Deserialize;

use axum::{
    response::{Html}, routing::{get, post}, Json, Router
};
use tokio::sync::oneshot;

async fn serve_html() -> impl axum::response::IntoResponse {
    // Html(include_str!("C:/Users/Administrator/Desktop/webrtc/rtp_to_webrtc.html"))
    let html_content = std::fs::read_to_string("C:/Users/Administrator/Desktop/webrtc/rtp_to_webrtc.html")
    .expect("Failed to read HTML file");

    // 返回 HTML 响应
    Html(html_content)
}
#[derive(serde::Serialize)]
struct ResponseData {
    data: String,
}

async fn recv_and_send_possession(Json(payload): Json<serde_json::Value>) -> Json<ResponseData> {

    let (tx,rx) = oneshot::channel();
    let (tx2,rx2) = oneshot::channel();

    let user_session = payload.get("data").unwrap();
    if let Some(s) = user_session.as_str() {
        tx.send(s.to_string());
    } else {
        return Json(ResponseData {data: "Invaild SDP".to_string(),})
    }
    tokio::spawn(crate::webrtc::connect::connect_and_rtp((tx2, rx)));
    match rx2.await {
        Ok(data) => {
            // println!("{}", data);
            return Json(ResponseData {r#data: data,})
        },
        Err(_) => {
            eprintln!("Invaild receive SDP");
            return Json(ResponseData {r#data: "can not get SDP".to_string(),});
        }
    }

}
#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/", get(serve_html))
        .route("/post", post(recv_and_send_possession));

    let listener = tokio::net::TcpListener::bind("192.168.50.34:3000")
    .await
    .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}