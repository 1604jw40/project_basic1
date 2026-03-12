const express = require('express');
const cors = require('cors');
const app = express();

app.use(cors());
app.use(express.json());

// 서버 메모리 DB (서버 종료 전까지 유지됨)
let plannerDB = {};

// 데이터 저장 API
app.post('/api/save', (req, res) => {
    const { date, data } = req.body;
    plannerDB[date] = data;
    console.log(`[저장 완료] 날짜: ${date}`);
    res.json({ success: true });
});

// 데이터 불러오기 API
app.get('/api/load/:date', (req, res) => {
    const date = req.params.date;
    const data = plannerDB[date] || null;
    res.json({ data });
});

// 서버 상태 확인
app.get('/api/status', (req, res) => {
    res.json({ status: "연결됨", time: new Date() });
});

const PORT = 5000;
app.listen(PORT, () => {
    console.log(`=========================================`);
    console.log(`🚀 플래너 DB 서버 가동 중 (포트: ${PORT})`);
    console.log(`주소: http://localhost:${PORT}`);
    console.log(`=========================================`);
});