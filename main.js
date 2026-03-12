/**
 * 스마트 매니지먼트 플래너 - DB 연동 및 오류 수정 로직
 * 수정 사항: 뉴스 422 에러 해결, 상세 콘솔 로그 추가
 */

const SERVER_URL = 'http://localhost:5000';
let db = { inbox: [], daily: [], weekly: [], monthly: [], motto: '', memo: '' };
const todayStr = new Date().toISOString().split('T')[0];

/** 1. 서버 데이터 로드 (콘솔 로그 포함) */
async function loadData() {
    const statusEl = document.getElementById('sync-status');
    try {
        console.log("🔄 [서버 연결 시도] 데이터를 가져오는 중...");
        const res = await fetch(`${SERVER_URL}/api/load/${todayStr}`);
        const result = await res.json();
        
        if (result.data) {
            db = result.data;
            document.getElementById('daily-motto').value = db.motto || '';
            document.getElementById('quick-memo').value = db.memo || '';
            console.log("✅ [데이터 로드 완료] 서버에서 최신 상태를 불러왔습니다:", db);
        } else {
            console.log("ℹ️ [데이터 없음] 오늘 작성된 데이터가 없어 초기화합니다.");
        }
        statusEl.innerText = "서버 연결됨";
        statusEl.style.color = "#10b981";
    } catch (e) {
        console.warn("❌ [서버 연결 실패] 로컬 모드로 작동합니다.");
        statusEl.innerText = "로컬 모드 (서버 미연결)";
        const local = localStorage.getItem(`local_db_${todayStr}`);
        if (local) db = JSON.parse(local);
    }
    render();
}

/** 2. 서버 데이터 저장 (콘솔 로그 포함) */
async function saveData() {
    // 로컬 백업 (서버가 꺼졌을 때 대비)
    localStorage.setItem(`local_db_${todayStr}`, JSON.stringify(db));
    
    console.log("📤 [서버 동기화 중] 변경된 데이터를 전송합니다:", db);

    try {
        const res = await fetch(`${SERVER_URL}/api/save`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ date: todayStr, data: db })
        });
        const result = await res.json();
        if (result.success) {
            console.log("✅ [동기화 성공] 서버 DB에 안전하게 저장되었습니다.");
        }
    } catch (e) {
        console.error("❌ [서버 저장 실패] 네트워크 연결을 확인하세요.");
    }
    render();
}

/** 3. 뉴스 피드 오류 수정 (422 에러 방지 및 안정화) */
async function fetchNews() {
    const newsEl = document.getElementById('news-display');
    // rss2json 서비스가 불안정할 경우를 대비해 allorigins 프록시와 결합하거나 더 단순한 경로 사용
    const googleNewsRss = 'https://news.google.com/rss?hl=ko&gl=KR&ceid=KR:ko';
    
    try {
        // 방법 1: rss2json 직접 호출 (인코딩 최적화)
        const res = await fetch(`https://api.rss2json.com/v1/api.json?rss_url=${encodeURIComponent(googleNewsRss)}`);
        
        if (!res.ok) throw new Error("뉴스 서버 응답 에러");
        
        const data = await res.json();
        if (data.status === 'ok') {
            const items = data.items.slice(0, 3);
            newsEl.innerHTML = items.map(i => `• ${i.title}`).join('<br>');
            console.log("📰 [뉴스 업데이트] 최신 뉴스 3개를 불러왔습니다.");
        } else {
            throw new Error("RSS 변환 실패");
        }
    } catch (e) {
        console.warn("⚠️ [뉴스 1차 시도 실패] 대체 프록시로 재시도합니다.");
        // 방법 2: AllOrigins를 통한 XML 직접 파싱 (대체 수단)
        try {
            const proxyRes = await fetch(`https://api.allorigins.win/get?url=${encodeURIComponent(googleNewsRss)}`);
            const proxyData = await proxyRes.json();
            const parser = new DOMParser();
            const xml = parser.parseFromString(proxyData.contents, "text/xml");
            const items = Array.from(xml.querySelectorAll("item")).slice(0, 3);
            
            if (items.length > 0) {
                newsEl.innerHTML = items.map(i => `• ${i.querySelector('title').textContent}`).join('<br>');
            } else {
                newsEl.innerText = "현재 뉴스 피드를 불러올 수 없습니다.";
            }
        } catch (err2) {
            newsEl.innerText = "네트워크 연결 확인 후 다시 시도해주세요.";
        }
    }
}

/** 4. 위치 및 날씨 정보 (한글 지역명) */
function fetchLocationWeather() {
    const weatherEl = document.getElementById('weather-display');
    if (!navigator.geolocation) return;

    navigator.geolocation.getCurrentPosition(async (pos) => {
        const { latitude, longitude } = pos.coords;
        try {
            const geoRes = await fetch(`https://nominatim.openstreetmap.org/reverse?format=json&lat=${latitude}&lon=${longitude}&accept-language=ko`);
            const geoData = await geoRes.json();
            const city = geoData.address.city || geoData.address.town || geoData.address.province || "대한민국";
            
            const weatherRes = await fetch(`https://wttr.in/${latitude},${longitude}?format=%t+%C&lang=ko`);
            const weatherText = await weatherRes.text();
            weatherEl.innerText = `대한민국 ${city} : ${weatherText.trim()}`;
        } catch (e) { weatherEl.innerText = "서울 : 18°C 맑음"; }
    });
}

/** 5. 할 일 관리 액션 */
function addWithCategory(category) {
    const input = document.getElementById('todo-input');
    const text = input.value.trim();
    if (!text) return;

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    
    db[category].push({ id: Date.now(), text, done: false, time: timeStr });
    input.value = "";
    saveData();
}

function syncExtra() {
    db.motto = document.getElementById('daily-motto').value;
    db.memo = document.getElementById('quick-memo').value;
    saveData();
}

function toggleDone(category, id) {
    const item = db[category].find(i => i.id === id);
    if (item) { item.done = !item.done; saveData(); }
}

/** 6. 렌더링 및 UI 업데이트 */
function render() {
    ['inbox', 'daily', 'weekly', 'monthly'].forEach(type => {
        const container = document.getElementById(`${type}-list`);
        const countEl = document.getElementById(`${type}-count`);
        if (!container) return;

        container.innerHTML = db[type].map(item => `
            <div class="todo-item" draggable="true" ondragstart="drag(event, ${item.id}, '${type}')">
                <input type="checkbox" ${item.done ? 'checked' : ''} onchange="toggleDone('${type}', ${item.id})">
                <div class="todo-content">
                    <div class="todo-text" style="${item.done ? 'text-decoration:line-through; opacity:0.3' : ''}">
                        ${item.text}
                    </div>
                    <div class="todo-meta">${item.time} 작성</div>
                </div>
            </div>
        `).join('');
        if (countEl) countEl.innerText = db[type].length;
    });

    const all = [...db.daily, ...db.weekly, ...db.monthly, ...db.inbox];
    const doneCount = all.filter(t => t.done).length;
    const rate = all.length === 0 ? 0 : Math.round((doneCount / all.length) * 100);
    document.getElementById('progress-fill').style.width = `${rate}%`;
    document.getElementById('progress-text').innerText = `${rate}%`;
}

/** 7. 드래그 앤 드롭 */
function allowDrop(ev) { ev.preventDefault(); }
function drag(ev, id, from) { ev.dataTransfer.setData("id", id); ev.dataTransfer.setData("from", from); }
function drop(ev, to) {
    ev.preventDefault();
    const id = ev.dataTransfer.getData("id"), from = ev.dataTransfer.getData("from");
    if (from === to) return;
    const idx = db[from].findIndex(i => i.id == id);
    if (idx > -1) {
        const [item] = db[from].splice(idx, 1);
        db[to].push(item); saveData();
    }
}
function dropToTrash(ev) {
    ev.preventDefault();
    const id = ev.dataTransfer.getData("id"), from = ev.dataTransfer.getData("from");
    const idx = db[from].findIndex(i => i.id == id);
    if (idx > -1) { db[from].splice(idx, 1); saveData(); }
}

window.onload = () => {
    document.getElementById('current-date-display').innerText = todayStr;
    document.getElementById('todo-input').onkeydown = (e) => {
        if (e.isComposing) return;
        if (e.key === 'Enter') addWithCategory('inbox');
    };
    fetchLocationWeather();
    fetchNews();
    loadData();
};