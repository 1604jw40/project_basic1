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

// AI 추천 요청 데이터 구조
#[derive(Deserialize)]
struct AIRequest {
    weather: String,
    motto: String,
    plans: String,
}

// 프론트엔드에 전달할 Firebase 설정 구조체 (camelCase 매핑)
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
    // 1. .env 파일로드
    dotenv::dotenv().ok();

    // CORS 설정: 외부 접속(ngrok) 및 로컬 테스트를 위해 모든 출처 허용
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 2. 라우터 설정
    let app = Router::new()
        // 루트 접속 시 index.html 서빙 (ServeFile 사용으로 경로 안정성 확보)
        .route("/", get_service(ServeFile::new("public/index.html")).handle_error(|_| async {
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        // Firebase 설정을 넘겨주는 브릿지 API
        .route("/api/config/firebase", get(get_firebase_config))
        // AI 추천 API
        .route("/api/ai/recommend", post(handle_ai_recommendation))
        .layer(cors);

    // 서버 바인딩 (기본 3000번 포트)
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().expect("유효하지 않은 주소입니다.");
    
    println!("🚀 Smart Planner Server 가동 중: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// .env에서 Firebase 정보를 읽어 JSON으로 반환하는 함수
async fn get_firebase_config() -> impl IntoResponse {
    let config = FirebaseConfig {
        api_key: env::var("FIREBASE_API_KEY").unwrap_or_default(),
        auth_domain: env::var("FIREBASE_AUTH_DOMAIN").unwrap_or_default(),
        project_id: env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
        storage_bucket: env::var("FIREBASE_STORAGE_BUCKET").unwrap_or_default(),
        messaging_sender_id: env::var("FIREBASE_MESSAGING_SENDER_ID").unwrap_or_default(),
        app_id: env::var("FIREBASE_APP_ID").unwrap_or_default(),
        measurement_id: env::var("FIREBASE_MEASUREMENT_ID").unwrap_or_default(),
    };
    Json(config)
}

// Google Gemini API를 호출하여 결과를 반환하는 함수
async fn handle_ai_recommendation(Json(payload): Json<AIRequest>) -> impl IntoResponse {
    // .env에서 API 키 로드
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "서버에 GEMINI_API_KEY가 설정되지 않았습니다.").into_response(),
    };

    // 최신 모델 주소 설정 (사용자 요청에 따른 gemini-2.5-flash 사용)
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