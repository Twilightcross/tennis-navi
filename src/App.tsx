import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AvailableSlot {
  park_name: string;
  tzone_name: string;
  use_day: number;
  rsv_num: number;
}

type Court = [instCd: string, name: string, sportCode: string];

const SPORT_TYPES: [code: string, label: string][] = [
  ["1030", "人工芝コート"],
  ["1020", "ハードコート"],
];

function formatDay(useDay: number): string {
  const s = String(useDay);
  return `${s.slice(0, 4)}/${s.slice(4, 6)}/${s.slice(6, 8)}`;
}

const today = new Date().toISOString().split("T")[0];

function App() {
  const [date, setDate] = useState(today);
  const [courts, setCourts] = useState<Court[]>([]);
  const [sportCode, setSportCode] = useState(SPORT_TYPES[0][0]);
  const [instCd, setInstCd] = useState("");
  const [slots, setSlots] = useState<AvailableSlot[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const courtsOfType = courts.filter(([, , code]) => code === sportCode);

  useEffect(() => {
    invoke<Court[]>("list_courts").then(setCourts);
  }, []);

  useEffect(() => {
    if (courtsOfType.length > 0 && !courtsOfType.some(([code]) => code === instCd)) {
      setInstCd(courtsOfType[0][0]);
    }
  }, [sportCode, courts]);

  async function search() {
    setLoading(true);
    setError("");
    setSlots([]);
    try {
      const result = await invoke<AvailableSlot[]>("search_vacant", { date, instCd });
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

      {/* 검색 */}
      <section>
        <div className="mb-4 flex items-center gap-2">
          <input
            type="date"
            lang="ja"
            value={date}
            min={today}
            onChange={(e) => setDate(e.target.value)}
            className="border rounded px-3 py-2 text-sm"
          />
          <select
            value={sportCode}
            onChange={(e) => setSportCode(e.target.value)}
            className="border rounded px-3 py-2 text-sm"
          >
            {SPORT_TYPES.map(([code, label]) => (
              <option key={code} value={code}>
                {label}
              </option>
            ))}
          </select>
          <select
            value={instCd}
            onChange={(e) => setInstCd(e.target.value)}
            className="border rounded px-3 py-2 text-sm"
          >
            {courtsOfType.map(([code, name]) => (
              <option key={code} value={code}>
                {name}
              </option>
            ))}
          </select>
          <button
            onClick={search}
            disabled={loading || !instCd}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? "検索中..." : "検索"}
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
