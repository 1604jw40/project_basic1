# Smart Planner · Team Workspace

공유 작업 공간에서 할 일과 채팅을 관리하는 웹 플래너 프로토타입입니다. React 기반 화면, Firebase Authentication/Firestore 연동, Rust AI 추천 API와 별도의 Node.js 메모리 저장 API가 함께 들어 있습니다.

> **상태:** 저장소 소스를 기준으로 작성한 안내입니다. 빌드, 브라우저 동작, Firebase 배포 및 외부 API 호출은 별도로 검증하지 않았습니다. 현재 구성을 운영 서비스의 보안 기준을 충족한 상태로 보아서는 안 됩니다.

## 소스에 구현된 기능

- Inbox, 일간·주간·월간 할 일 목록과 완료 상태 관리
- 공동 플래너, 메모·목표 문구, 작업 공간 사용자 및 채팅의 Firestore 연동
- Firebase 이메일/비밀번호 인증 흐름과 닉네임 표시
- 집중 타이머 및 진행률 표시
- 일간 계획과 목표 문구를 Gemini에 보내는 추천 요청

위 목록은 구현 존재를 설명하며, 실제 서비스에서의 정상 작동을 보증하지 않습니다.

## 구성

| 경로 | 역할 |
| --- | --- |
| `public/index.html` | 현재 Rust 루트 경로와 Firebase Hosting이 제공하도록 설정된 화면 |
| `src/main.rs` | Axum 서버, Firebase 설정 응답, Gemini 추천 요청 |
| `Cargo.toml`, `Cargo.lock` | Rust 의존성과 잠금 파일 |
| `.env.example` | 비밀값 없는 서버 환경 변수 예시 |
| `server.js` | 별도 Express 메모리 저장 서버 |
| `package.json`, `package-lock.json` | Node.js 의존성과 실행 스크립트 |
| `project1.html`, `slide.html` | 별도 HTML 자료. 기본 제공 경로는 아님 |
| `firebase.json`, `firestore.rules`, `firestore.indexes.json` | Hosting 및 Firestore 구성 |
| `firebase-config.js` | Firebase 클라이언트 설정 파일 |

## 실행 전 필수 확인

1. **Gemini 자격 증명은 서버 환경 변수 `GEMINI_API_KEY`에서 읽습니다.** 이전 커밋에 하드코딩 값이 남아 있으므로, 기존 키가 유효하다면 소유자가 폐기·재발급해야 합니다. 현재 소스에서 제거한 것만으로 과거 노출이 해결되지는 않습니다. 키의 유효성 확인·폐기 및 Git 이력 변경은 수행하지 않았습니다.
2. 로컬에서는 `.env.example`을 참고하여 Git에서 제외된 `.env`에 본인의 새 키를 설정할 수 있습니다. 이미 설정된 프로세스 환경 변수가 우선합니다. 키가 없거나 빈 문자열·공백뿐이면 AI 요청은 외부 호출 없이 HTTP 503과 설정 안내를 반환합니다. 비밀값이나 `.env`, 서비스 계정 파일을 Git에 커밋하지 마세요.
3. Firebase 웹 설정은 클라이언트 식별 설정이며 자체적으로 데이터 접근을 보호하지 않습니다. 본인 소유의 테스트 프로젝트와 인증 설정을 사용해야 합니다.
4. 저장소의 Firestore 규칙은 **인증된 모든 사용자에게 모든 문서의 읽기·쓰기**를 허용합니다. 팀별 권한 분리 규칙이 아닙니다. 실제 배포 규칙은 미확인입니다. 실사용 데이터 입력 전 접근 제어 검토가 필요합니다.
5. Rust 서버는 모든 네트워크 인터페이스의 3000번 포트에서 수신하고 CORS를 전체 허용합니다. AI 엔드포인트에는 인증 검증이 구현되어 있지 않습니다. 외부 공개 전에 인증·요청 제한·접근 범위를 검토해야 합니다.

## 로컬 실행 경로

아래 명령은 소스와 manifest 기준 안내이며 이번 문서 정비에서 실행 검증하지 않았습니다. 위 보안 항목을 먼저 해결하고 별도 테스트 환경에서 사용하세요.

### Rust 웹 서버

Rust/Cargo(Rust 2021 edition 지원)가 필요합니다. 저장소 루트에서 `.env.example`을 `.env`로 복사하고, AI 기능을 사용할 때만 새 `GEMINI_API_KEY`를 설정합니다. 키 없이도 서버는 시작하도록 구현되어 있지만 AI 요청은 503을 반환합니다.

```sh
cargo run --locked
```

화면 주소는 `http://localhost:3000`입니다. 브라우저에서 React, Babel, Tailwind 및 Firebase SDK를 CDN으로 읽으므로 네트워크 접근이 필요합니다.

| 경로 | 역할 |
| --- | --- |
| `GET /` | `public/index.html` 제공 |
| `GET /api/config/firebase` | 서버에 명시된 Firebase 클라이언트 설정 반환 |
| `POST /api/ai/recommend` | `weather`, `motto`, `plans` 문자열을 받아 Gemini에 추천 요청 |

현재 화면은 Firebase 설정을 자체 포함하며, AI 요청은 기본적으로 현재 origin으로 보냅니다. 5500번 포트에서 화면을 열면 AI 요청만 로컬 3000번 포트를 사용합니다.

### 별도 Node.js 저장 API

Node.js와 npm이 필요하며, 저장소에 버전 고정 설정은 없습니다.

```sh
npm ci
npm start
```

5000번 포트에서 `POST /api/save`, `GET /api/load/:date`, `GET /api/status`를 제공합니다. 데이터는 서버 메모리에만 있어 재시작하면 사라집니다. Rust 서버의 대체 구현이 아니며, 현재 `public/index.html`의 Firestore 저장 흐름과 자동으로 연결되지 않습니다.

## 배포 및 데이터 주의

- Firebase Hosting 설정은 `public/`의 정적 파일과 SPA 재작성만 포함합니다. Rust AI API를 배포하거나 프록시하는 설정은 없습니다. 정적 Hosting 배포만으로 AI 기능까지 제공되지 않습니다.
- AI 추천 사용 시 계획과 목표 문구가 외부 모델 제공자에게 전송됩니다. 개인 정보나 팀의 비공개 정보를 넣지 마세요.
- 운영 배포, 계정 생성, 실제 사용자 데이터 저장 또는 유료 API 호출은 이 문서 정비에서 수행하지 않았습니다.
- 다음 정비 우선순위는 기존 키 폐기·재발급 확인, 팀별 Firestore 권한, AI API 인증/요청 제한, 실행·통합 테스트입니다.
