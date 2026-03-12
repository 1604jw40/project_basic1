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

// AI 추천 요청 데이터 구조
#[derive(Deserialize)]
struct AIRequest {
    weather: String,
    motto: String,
    plans: String,
}

// 프론트엔드에 전달할 Firebase 설정 구조체
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
    #[serde(rename = "measurementId")]
    measurement_id: String,
}

#[tokio::main]
async fn main() {
    // CORS 설정: 외부 접속 및 로컬 테스트를 위해 모든 출처 허용
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 라우터 설정
    let app = Router::new()
        // 루트 접속 시 index.html 서빙
        .route("/", get_service(ServeFile::new("public/index.html")).handle_error(|_| async {
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        // Firebase 설정을 반환하는 API
        .route("/api/config/firebase", get(get_firebase_config))
        // AI 추천 분석 API
        .route("/api/ai/recommend", post(handle_ai_recommendation))
        .layer(cors);

    // 포트 설정 (기본 3000)
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Smart Planner Server 가동 중: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 💡 Firebase 정보를 코드에 직접 명시하여 반환합니다.
async fn get_firebase_config() -> impl IntoResponse {
    let config = FirebaseConfig {
        api_key: "AIzaSyDJQwt9GT0MrkSqs1gzFEcRBAmY0-xYRO0".to_string(),
        auth_domain: "projectbasic1-57e28.firebaseapp.com".to_string(),
        project_id: "projectbasic1-57e28".to_string(),
        storage_bucket: "projectbasic1-57e28.firebasestorage.app".to_string(),
        messaging_sender_id: "610613658787".to_string(),
        app_id: "1:610613658787:web:5bec507bac0a9ed1512b16".to_string(),
        measurement_id: "G-FZES5J73VK".to_string(),
    };
    Json(config)
}

// Google Gemini API를 호출하여 결과를 반환하는 함수
async fn handle_ai_recommendation(Json(payload): Json<AIRequest>) -> impl IntoResponse {
    // 💡 Gemini API 키를 코드에 직접 명시합니다.
    let api_key = "AIzaSyD1CqpjQfcaZLbn3DZ7jT7ovXsOJ6f5UM0";

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();
    
    // AI에 전달할 프롬프트 구성
    let prompt = format!(
        "Role: Coach for a shared workspace. Context: Weather {}, Motto '{}', Plans [{}]. \
        Recommend 3 specific tasks. JSON ONLY: {{\"recoms\":[{{\"title\":\"task\",\"desc\":\"reason\"}}]}}. Korean lang.",
        payload.weather, payload.motto, payload.plans
    );

    // 외부 API 호출
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
            (StatusCode::INTERNAL_SERVER_ERROR, "AI 응답 데이터를 처리할 수 없습니다.").into_response()
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "AI 서버와의 통신에 실패했습니다.").into_response(),
    }
}