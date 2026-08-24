<div align="center">

# 🎾 tennis-navi

**東京都立公園テニスコートの空き状況を自動検索するデスクトップアプリ**

[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

[English](README.en.md)

</div>

---

## 📌 背景・目的

[東京都立公園スポーツ施設予約サイト](https://kouen.sports.metro.tokyo.lg.jp/web/index.jsp) は
Struts/JSP ベースの旧式システムで、テニスコートの空き状況を確認するには
公園ごと・日付ごとに手動でページを切り替えて検索する必要がある。

この手間を解消するため、サイトの内部 AJAX API（非公開）をブラウザの通信ログから解析し、
ログイン〜空き状況取得までを自動化するデスクトップアプリを開発した。

## ✨ 機能

- **ログイン** — 予約サイトのセッション Cookie（JSESSIONID）とログイントークンを再現し、ID / パスワードで認証
- **空きコート検索** — 指定日のテニスコート空き状況を取得し、公園名・時間帯・空き面数を一覧表示

## 🛠 技術スタック

| レイヤー | 技術 |
|--|--|
| デスクトップフレームワーク | Tauri v2 |
| フロントエンド | React 19 + TypeScript + Vite + Tailwind CSS v4 |
| バックエンドロジック | Rust（reqwest / tokio / serde） |

### なぜ Tauri（Electron ではなく）か

- **軽量・省リソース** — OS 標準の WebView を利用するため、Chromium を同梱する Electron と比べてバンドルサイズ・メモリ使用量が大幅に小さい。常駐前提のツールなのでリソース消費は重要な選定基準だった
- **Rust バックエンド** — ログイン・スクレイピング等のコアロジックを Rust で書けるため、型安全性とパフォーマンスを確保しつつ Rust の学習も兼ねられる
- **将来のモバイル展開** — Tauri v2 は iOS / Android ビルドも公式サポートしており、React UI とロジックの多くをそのまま流用してモバイル版へ拡張できる

## 🏗 仕組み

対象サイトに公式 API は存在しない。ブラウザの Network タブでリクエストを解析し、
以下の呼び出し順序を Rust 側（`reqwest` + Cookie ストア）で再現している。

```
1. GET  index.jsp                              → セッション Cookie 取得
2. POST rsvWTransUserLoginAction.do             → ログインページの hidden トークン取得
3. POST rsvWUserAttestationLoginAction.do       → ID / パスワードでログイン
4. POST rsvWOpeInstSrchVacantAction.do          → 検索条件をサーバーセッションに設定
5. POST rsvWOpeInstSrchVacantAjaxAction.do      → 空き状況 JSON を取得
```

取得した JSON から `status == 0 && rsvNum > 0`（空きあり）の枠だけを抽出し、UI に表示する。

## 🚀 開発環境での起動

```bash
npm install
npm run tauri dev
```

## 🗺 今後の予定

- [ ] 対象公園の拡張（現状は 1 公園のみ対応）
- [ ] 日付範囲・時間帯・種目の条件指定
- [ ] 空き検知時の macOS ネイティブ通知
- [ ] 定期自動検索（バックグラウンド実行）
- [ ] モバイル対応（Tauri v2 で iOS / Android ビルド）

詳細な要件・API 調査メモは [REQUIREMENTS.md](REQUIREMENTS.md) を参照。

## ⚠️ Note

個人利用を目的としたプロジェクトです。自分のアカウントでの予約状況確認を自動化することのみを目的としており、
対象サイトへの過度な負荷や第三者データの収集は意図していません。
