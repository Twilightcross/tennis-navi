import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function Login() {
  const [userId, setUserId] = useState("");
  const [password, setPassword] = useState("");
  const [loginStatus, setLoginStatus] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);

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

  return (
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
  );
}
