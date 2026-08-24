<div align="center">

# 🎾 tennis-navi

**A desktop app that automatically checks tennis court availability at Tokyo metropolitan park facilities**

[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[日本語](README.md)

</div>

---

## 📌 Background

The [Tokyo Metropolitan Park Sports Facility reservation site](https://kouen.sports.metro.tokyo.lg.jp/web/index.jsp)
runs on a legacy Struts/JSP system. Checking tennis court availability means manually switching
between pages for each park and each date — tedious to do by hand.

To remove that friction, I reverse-engineered the site's internal (undocumented) AJAX API by
inspecting browser network traffic, and built a desktop app that automates everything from login
to fetching court availability.

## ✨ Features

- **Login** — Reproduces the reservation site's session cookie (JSESSIONID) and login-token flow to authenticate with a user ID / password
- **Court availability search** — Fetches tennis court availability for a given date and lists park name, time slot, and number of open courts

## 🛠 Tech Stack

| Layer | Technology |
|--|--|
| Desktop framework | Tauri v2 |
| Frontend | React 19 + TypeScript + Vite + Tailwind CSS v4 |
| Backend logic | Rust (reqwest / tokio / serde) |

### Why Tauri over Electron

- **Lightweight** — Tauri uses the OS's native WebView instead of bundling Chromium, so binary size and memory footprint are far smaller than Electron. Since this tool is meant to run continuously in the background, resource usage was a key factor
- **Rust backend** — Core logic (login, request handling, parsing) is written in Rust, giving type safety and performance while also serving as a way to build up Rust experience
- **Mobile-ready path** — Tauri v2 officially supports iOS / Android builds, so most of the React UI and business logic can be reused when extending to mobile

## 🏗 How it works

The target site has no official API. I analyzed requests via the browser's Network tab and
reproduced the following call sequence in Rust, using `reqwest` with a cookie store:

```
1. GET  index.jsp                              → obtain session cookie
2. POST rsvWTransUserLoginAction.do             → fetch hidden login token from the login page
3. POST rsvWUserAttestationLoginAction.do       → authenticate with user ID / password
4. POST rsvWOpeInstSrchVacantAction.do          → set search conditions on the server session
5. POST rsvWOpeInstSrchVacantAjaxAction.do      → fetch availability data as JSON
```

From the returned JSON, only slots matching `status == 0 && rsvNum > 0` (available) are
extracted and rendered in the UI.

## 🚀 Getting Started

```bash
npm install
npm run tauri dev
```

## 🗺 Roadmap

- [ ] Support more parks (currently hardcoded to a single park)
- [ ] Filter by date range, time slot, and sport type
- [ ] Native macOS notification when a slot opens up
- [ ] Periodic background search
- [ ] Mobile builds (iOS / Android via Tauri v2)

See [REQUIREMENTS.md](REQUIREMENTS.md) (Japanese) for detailed requirements and API research notes.

## ⚠️ Note

This is a personal-use project. Its sole purpose is automating availability checks for my own
account — it is not intended to place excessive load on the target site or to collect data
about third parties.
