# tennis-navi 요건 정의서

도쿄도 스포츠시설 예약 사이트에서 테니스 코트 빈 자리를 자동으로 탐색하는 앱

---

## 배경 / 목적

- 대상 사이트: https://kouen.sports.metro.tokyo.lg.jp/web/index.jsp
- 기존에는 날짜·공원을 수동으로 하나씩 검색해야 했음
- 이를 자동화해서 빈 코트 발견 시 즉시 알림을 받는 것이 목표

---

## 개발 단계

### Phase 1 — Mac 데스크톱 앱 (Tauri)
- Mac 메뉴바 상시 실행
- 설정한 조건으로 주기적 자동 검색
- 빈 코트 발견 시 Mac 네이티브 알림
- 비용 0원, 서버 불필요

### Phase 2 — 모바일 확장 (Tauri v2)
- iOS / Android 빌드
- 백그라운드 알림 방식 검토 필요

### Phase 3 — 클라우드 이전 (선택)
- 필요 시 Go 서버 + 클라우드 배포로 전환

---

## 기능 요건

### 검색 조건 설정
- 날짜 범위 지정
- 시간대 지정 (9시 / 11시 / 13시 / 15시 / 17시)
- 종목 선택: テニス（ハード）/ テニス（人工芝）
- 공원 선택: 전체 or 개별 지정

### 자동 검색
- 설정 간격마다 전체 공원 순회 검색
- 빈 자리 감지 (status: 0, rsvNum > 0)

### 알림
- Phase 1: Mac 네이티브 알림 팝업
- 알림 클릭 → 해당 예약 페이지로 브라우저 이동

### 자동 예약 (선택 기능)
- 로그인 자동화 (ID/PW 저장)
- 빈 자리 발견 시 자동 예약 실행

---

## 기술 스펙

### Phase 1
| 항목 | 기술 | 버전 |
|--|--|--|
| 데스크톱 앱 | Tauri | v2.11.3 |
| 백엔드 로직 | Rust | 1.96.0 |
| 프론트 빌드 | Vite + React + TypeScript | - |
| CSS | Tailwind CSS | v4 |
| 알림 | macOS 네이티브 알림 | - |

---

## 대상 사이트 API 분석

### 시스템 정보
| 항목 | 내용 |
|--|--|
| 시스템 | Apache Struts (JSP) |
| 인증 | 세션 쿠키 (JSESSIONID) |
| 검색 | 비로그인으로 가능 |
| 데이터 형식 | JSON (HTML 파싱 불필요) |

### 종목 코드
| 종목 | selectPpsCd |
|--|--|
| テニス（ハード） | 1020 |
| テニス（人工芝） | 1030 |
| (공통 클래스) selectPpsdCd / selectPpsClsCd | 1000 |

### API 호출 순서
```
1. GET  /web/index.jsp
   → 세션 쿠키(JSESSIONID) 획득

2. POST /web/rsvWOpeInstSrchVacantAction.do
   → 서버 세션 상태 초기화
   → params: daystart, selectPpsClPpscd, selectAreaBcd 등

3. GET  /web/rsvWTransFavorite2InfoBuildAjaxAction.do
   → 공원 전체 목록 취득 (JSON)
   → params: displayNo=prwre1000, selectPpsdCd=1000, selectPpsCd=1030, selectAreaCd=0

4. GET  /web/rsvWOpeInstSrchVacantBuildAjaxAction.do
   → 공원별 코트(시설) 목록 취득 (JSON)
   → params: displayNo=prwre1000, bldCd={공원코드}

5. GET  /web/rsvWOpeInstSrchVacantAjaxAction.do
   → 빈 자리 실제 데이터 취득 (JSON) ← 핵심
   → params: displayNo=prwrc2000, useDay=YYYYMMDD, bldCd, instCd, transVacantMode=11, clearFlag=0
```

### 빈 자리 응답 구조 (VacantAjaxAction)
```json
{
  "result": [
    {
      "tzoneNo": 10,
      "tzoneName": "　９時",
      "timeResult": [
        {
          "status": 0,        // 0 = 빈 자리, 210 = 예약 완료
          "rsvNum": 4,        // 빈 코트 수
          "alt": "空き",      // "空き" = 빈 자리, "予約あり" = 예약 완료
          "startTime": 900,
          "endTime": 1100,
          "useDay": 20260623
        }
      ]
    }
  ],
  "weekDay": [ ... ]  // 7일치 날짜 정보
}
```

### 빈 자리 판별 조건
- `status == 0` AND `rsvNum > 0` → 예약 가능

### 공원 목록 (テニス（人工芝）기준, 총 27개)
| 공원명 | bcd |
|--|--|
| 日比谷公園 | 1000 |
| 芝公園 | 1010 |
| 猿江恩賜公園 | 1040 |
| 亀戸中央公園 | 1050 |
| 木場公園 | 1060 |
| 祖師谷公園 | 1070 |
| 東白鬚公園 | 1090 |
| 浮間公園 | 1100 |
| 城北中央公園 | 1110 |
| 赤塚公園 | 1120 |
| 東綾瀬公園 | 1130 |
| 舎人公園 | 1140 |
| 篠崎公園Ａ | 1150 |
| 大島小松川公園 | 1160 |
| 汐入公園 | 1170 |
| 高井戸公園 | 1175 |
| 善福寺川緑地 | 1180 |
| 光が丘公園 | 1190 |
| 石神井公園Ｂ | 1205 |
| 井の頭恩賜公園 | 1220 |
| 武蔵野中央公園 | 1230 |
| 小金井公園 | 1240 |
| 野川公園 | 1260 |
| 府中の森公園 | 1270 |
| 東大和南公園 | 1280 |
| 大井ふ頭海浜公園Ｂ | 1315 |
| 有明テニスＣ人工芝コート | 1360 |

---

## 미결 사항
- [ ] ハード 코트 공원 목록 수집 (selectPpsCd=1020으로 동일 요청)
- [ ] 자동 예약 기능 포함 여부 결정
- [ ] 검색 주기 결정 (5분? 10분?)
- [ ] Mac 알림 클릭 시 이동할 URL 구조 파악
