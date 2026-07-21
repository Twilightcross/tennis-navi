import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AvailableSlot {
  park_name: string;
  tzone_name: string;
  use_day: number;
  rsv_num: number;
}

function formatDay(useDay: number): string {
  const s = String(useDay);
  return `${s.slice(0, 4)}/${s.slice(4, 6)}/${s.slice(6, 8)}`;
}

function App() {
  const [userId, setUserId] = useState("");
  const [password, setPassword] = useState("");
  const [loginStatus, setLoginStatus] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);

  const [slots, setSlots] = useState<AvailableSlot[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleLogin() {
    setLoginStatus("로그인 중...");
    try {
      const msg = await invoke<string>("login", { userId, password });
      setLoginStatus(msg);
      setLoggedIn(true);
    } catch (e) {
      setLoginStatus(`실패: ${e}`);
      setLoggedIn(false);
    }
  }

  async function search() {
    setLoading(true);
    setError("");
    setSlots([]);
    try {
      const today = new Date().toISOString().split("T")[0];
      const result = await invoke<AvailableSlot[]>("search_vacant", { date: today });
      setSlots(result);
    } catch (e) {
      setError(`에러: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="p-6 max-w-3xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">tennis-navi</h1>

      {/* 로그인 */}
      <section className="mb-6 p-4 border rounded">
        <h2 className="font-semibold mb-3">로그인</h2>
        <div className="flex flex-col gap-2 mb-3">
          <input
            type="text"
            placeholder="이용자 번호"
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            className="border rounded px-3 py-2 text-sm"
          />
          <input
            type="password"
            placeholder="비밀번호"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="border rounded px-3 py-2 text-sm"
          />
        </div>
        <button
          onClick={handleLogin}
          className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-800 text-sm"
        >
          로그인
        </button>
        {loginStatus && (
          <p className={`mt-2 text-sm ${loggedIn ? "text-green-600" : "text-red-500"}`}>
            {loginStatus}
          </p>
        )}
      </section>

      {/* 검색 */}
      <section>
        <div className="mb-4">
          <button
            onClick={search}
            disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? "검색 중..." : "빈 코트 검색"}
          </button>
        </div>

        {error && <p className="text-red-500 text-sm">{error}</p>}

        {slots.length > 0 && (
          <div className="mt-2">
            <p className="mb-2 text-sm font-medium">빈 자리 {slots.length}건</p>
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="bg-gray-100">
                  <th className="border p-2 text-left">공원</th>
                  <th className="border p-2 text-left">날짜</th>
                  <th className="border p-2 text-left">시간대</th>
                  <th className="border p-2 text-left">빈 코트</th>
                </tr>
              </thead>
              <tbody>
                {slots.map((s, i) => (
                  <tr key={i} className="hover:bg-gray-50">
                    <td className="border p-2">{s.park_name}</td>
                    <td className="border p-2">{formatDay(s.use_day)}</td>
                    <td className="border p-2">{s.tzone_name}</td>
                    <td className="border p-2">{s.rsv_num}면</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {!loading && slots.length === 0 && !error && (
          <p className="mt-4 text-gray-400 text-sm">버튼을 눌러 검색해주세요.</p>
        )}
      </section>
    </main>
  );
}

export default App;
