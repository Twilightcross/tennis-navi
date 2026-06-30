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

---

## 예약 자동화 진행 상황 (reserve_slot)

### 흐름 (reqwest 기반, src-tauri/src/lib.rs)
```
1. POST rsvWOpeInstReservAjaxAction.do   (슬롯 선택, AJAX 헤더 필요)
   → { "selectState": 1 } 이면 성공

2. POST rsvWOpeReservedApplyAction.do    (예약 상세 페이지 취득)
   → HTML에서 insIRsvJKey 등 hidden 필드 전부 파싱 (parse_all_hidden)

3. POST rsvWInstRsvApplyAction.do        (예약 확정)
   → hidden 필드 + purpose/applyNum/recaptchaToken 등 오버라이드
   → 일본어 값("1面" 등) 포함 → Shift_JIS 퍼센트 인코딩 필요 (to_sjis_form, encoding_rs 사용)
```

1, 2단계까지는 정상 동작 확인됨. 3단계(확정)에서 reCAPTCHA에 막혀있음.

### reCAPTCHA 차단 — 시도 내역
| 시도 | recaptchaToken 값 | 결과 |
|--|--|--|
| 빈 문자열 | `""` | `システム異常が発生しました。` |
| 더미 난수 토큰 | `03AGdBq24...` (가짜) | `確認のため、チェックを入れてから...` (체크 에러) |
| JS 폴백 문자열 | `"Failed to load reCAPTCHA JavaScript."` | 동일하게 체크 에러 |

→ 서버가 Google reCAPTCHA siteverify API로 토큰을 실제 검증함. 가짜 토큰은 전부 실패.

### reCAPTCHA 설정값 (상세 페이지 HTML에서 확인)
```js
gRecaptchaActive = true
gRecaptchaSiteKey = '6Lf_ciYpAAAAAEk3QnqYrrxgT9gjiu6GeNVm2VTa'
gRecaptchaActionName = 'webRsv'
gRecaptchaErrMsg = 'Failed to load reCAPTCHA JavaScript.'
```
(JS 원본: `js/prwea1000.js`의 `checkTextValue()` 함수 — reCAPTCHA v3, `grecaptcha.execute(siteKey, {action: 'webRsv'})`)

### 채택한 해결 전략: WebView 토큰 헬퍼

- 2captcha 같은 유료 캡챠 솔빙 서비스는 비용 발생 이유로 제외.
- reCAPTCHA v3는 **도메인 + 사이트 키** 단위로 검증됨 (특정 로그인 페이지일 필요 없음).
  → 로그인 없이 접근 가능한 `index.jsp` 같은 공개 페이지에서도 동일 사이트 키로 토큰 생성 가능 (이론상).
- 따라서 예약 본체는 기존 reqwest 흐름(Shift_JIS 인코딩 포함, 이미 검증됨)을 그대로 두고, **reCAPTCHA 토큰만** 별도의 작은 Tauri WebView 창에서 받아온다.

```
1. reserve_slot 진행 중 (reqwest로 로그인~상세페이지까지 완료)
2. 숨겨진/작은 WebView 창 오픈 → kouen.sports.metro.tokyo.lg.jp 의 공개 페이지로 이동
3. WebView 안에서 구글 reCAPTCHA 스크립트(api.js?render=SITE_KEY) 로드
   → grecaptcha.execute(SITE_KEY, {action: 'webRsv'}) 실행 → 진짜 토큰 획득
4. Tauri IPC로 토큰을 Rust로 전달 (tauri.conf.json에 dangerousRemoteDomainIpcAccess 설정 필요:
   domain: kouen.sports.metro.tokyo.lg.jp, window: "recaptcha-helper")
5. WebView 창 닫고, 받은 진짜 토큰을 기존 to_sjis_form 흐름에 넣어 rsvWInstRsvApplyAction.do POST
```

**왜 WebView로 전체 흐름을 옮기지 않았는가:**
브라우저 JS의 `TextEncoder`는 UTF-8만 지원하고 Shift_JIS 인코딩 기능이 없음. 반면 Rust는 `encoding_rs`로 이미 해결됨. 따라서 reCAPTCHA 토큰 획득(브라우저 필요한 부분)만 분리하고, 나머지는 검증된 Rust 코드 재사용.

**미검증 리스크:**
reCAPTCHA v3는 토큰 유효성(success/fail)과 별개로 **점수(score, 0~1)** 를 매김. 사람처럼 행동(마우스 이동, 체류 시간 등)하지 않는 빈 페이지에서 즉시 스크립트만 실행하면 점수가 낮게 나올 수 있고, 서버가 일정 점수 미만은 거부할 가능성 있음. 실제 사이트가 점검 중이라 미검증 — 사이트 복구 후 테스트 필요.

### TODO (사이트 점검 복구 후)
- [ ] tauri.conf.json에 `withGlobalTauri: true` + `dangerousRemoteDomainIpcAccess` 추가
- [ ] `submit_recaptcha_token` Tauri 커맨드 추가 (oneshot 채널로 reserve_slot에 토큰 전달)
- [ ] `recaptcha-helper` WebviewWindow 생성 로직 추가 (init script로 grecaptcha 실행)
- [ ] reserve_slot의 recaptchaToken 오버라이드를 실제 받은 토큰으로 교체
- [ ] 점수 낮아서 거부될 경우 → 페이지 체류 시간 늘리기/마우스 이벤트 시뮬레이션 등 대응 검토
