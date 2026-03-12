use axum::{
    routing::{get, post, get_service},
    extract::Json,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeFile;
use reqwest::Client;
use std::env;

#[derive(Deserialize)]
struct AIRequest {
    weather: String,
    motto: String,
    plans: String,
}

// 💡 프론트엔드에 전달할 Firebase 설정 구조체
#[derive(Serialize)]
struct FirebaseConfig {
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(rename = "authDomain")]
    auth_domain: String,
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(rename = "storageBucket")]
    storage_bucket: String,
    #[serde(rename = "messagingSenderId")]
    messaging_sender_id: String,
    #[serde(rename = "appId")]
    app_id: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get_service(ServeFile::new("public/index.html")).handle_error(|_| async {
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        // 💡 새로운 설정 브릿지 엔드포인트
        .route("/api/config/firebase", get(get_firebase_config))
        .route("/api/ai/recommend", post(handle_ai_recommendation))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Smart Planner Server 가동 중: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 💡 .env에서 Firebase 정보를 읽어 JSON으로 반환하는 함수
async fn get_firebase_config() -> impl IntoResponse {
    let config = FirebaseConfig {
        api_key: env::var("FIREBASE_API_KEY").unwrap_or_default(),
        auth_domain: env::var("FIREBASE_AUTH_DOMAIN").unwrap_or_default(),
        project_id: env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
        storage_bucket: env::var("FIREBASE_STORAGE_BUCKET").unwrap_or_default(),
        messaging_sender_id: env::var("FIREBASE_MESSAGING_SENDER_ID").unwrap_or_default(),
        app_id: env::var("FIREBASE_APP_ID").unwrap_or_default(),
    };
    Json(config)
}

async fn handle_ai_recommendation(Json(payload): Json<AIRequest>) -> impl IntoResponse {
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "API 키 누락").into_response(),
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();
    let prompt = format!(
        "Role: Coach for a shared workspace. Context: Weather {}, Motto '{}', Plans [{}]. \
        Recommend 3 specific tasks. JSON ONLY: {{\"recoms\":[{{\"title\":\"task\",\"desc\":\"reason\"}}]}}. Korean lang.",
        payload.weather, payload.motto, payload.plans
    );

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "generationConfig": { "responseMimeType": "application/json", "temperature": 0.5 }
        }))
        .send()
        .await;

    match response {
        Ok(res) => {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                if let Some(text) = data["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    return (StatusCode::OK, text.to_string()).into_response();
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, "AI 분석 오류").into_response()
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "AI 통신 실패").into_response(),
    }
}