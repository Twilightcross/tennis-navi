use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::State;

mod courts;
mod mail;

const BASE_URL: &str = "https://kouen.sports.metro.tokyo.lg.jp/web";

struct AppState {
    client: Mutex<reqwest::Client>,
}

#[derive(Serialize, Deserialize)]
struct TimeResult {
    status: i32,
    #[serde(rename = "rsvNum")] rsv_num: i32,
    alt: String,
    #[serde(rename = "startTime")] start_time: i32,
    #[serde(rename = "endTime")] end_time: i32,
    #[serde(rename = "useDay")] use_day: i64,
}

#[derive(Serialize, Deserialize)]
struct TZone {
    #[serde(rename = "tzoneName")] tzone_name: String,
    #[serde(rename = "tzoneNo")] tzone_no: i32,
    #[serde(rename = "timeResult")] time_result: Vec<TimeResult>,
}

#[derive(Serialize, Deserialize)]
struct VacantResponse { result: Vec<TZone> }

#[derive(Serialize, Deserialize)]
pub(crate) struct AvailableSlot {
    park_name: String,
    pub(crate) tzone_name: String,
    pub(crate) use_day: i64,
    pub(crate) rsv_num: i32,
}

fn parse_hidden(html: &str, field_name: &str) -> Option<String> {
    let search = format!("name=\"{}\"", field_name);
    let name_pos = html.find(&search)?;
    let tag_start = html[..name_pos].rfind('<')?;
    let tag_end = name_pos + html[name_pos..].find('>')?;
    let tag = &html[tag_start..=tag_end];
    let v_start = tag.find("value=\"")? + 7;
    let v_end = v_start + tag[v_start..].find('"')?;
    Some(tag[v_start..v_end].to_string())
}


#[tauri::command]
async fn login(
    state: State<'_, AppState>,
    user_id: String,
    password: String,
) -> Result<String, String> {
    let client = state.client.lock().await;

    client.get(format!("{}/index.jsp", BASE_URL)).send().await.map_err(|e| e.to_string())?;

    let mut nav_params = HashMap::new();
    nav_params.insert("displayNo", "index");
    nav_params.insert("displayNoFrm", "index");
    let login_page = client
        .post(format!("{}/rsvWTransUserLoginAction.do", BASE_URL))
        .form(&nav_params)
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    let login_j_key = parse_hidden(&login_page, "loginJKey")
        .ok_or("loginJKey를 찾을 수 없습니다.".to_string())?;

    let char_passes: Vec<(&str, String)> = password
        .chars()
        .map(|c| ("loginCharPass", c.to_string()))
        .collect();

    let mut params: Vec<(&str, &str)> = vec![
        ("userId", user_id.as_str()),
        ("password", password.as_str()),
        ("fcflg", ""),
        ("displayNo", "pawab2100"),
        ("loginJKey", login_j_key.as_str()),
    ];
    for (k, v) in &char_passes {
        params.push((k, v.as_str()));
    }

    let body = client
        .post(format!("{}/rsvWUserAttestationLoginAction.do", BASE_URL))
        .form(&params)
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    if body.contains("pawab2000") {
        Ok("로그인 성공".to_string())
    } else if body.contains("pawab2100") {
        Err("ID 또는 비밀번호가 틀렸습니다.".to_string())
    } else {
        Err(format!("알 수 없는 응답: {}", &body[..body.len().min(100)]))
    }
}

/// 이번달 말일 계산
fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = (date.year(), date.month());
    let (next_y, next_m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    NaiveDate::from_ymd_opt(next_y, next_m, 1).unwrap() - Duration::days(1)
}

/// 검색 기간: 오늘 ~ 1주일 뒤 (임시로 1주만 검색, 원래는 이번달 말일까지)
pub(crate) fn current_search_period() -> (NaiveDate, NaiveDate) {
    let today = Local::now().date_naive();
    (today, today + Duration::days(6))
}

fn is_weekend(use_day: i64) -> bool {
    let s = use_day.to_string();
    let parsed = (|| {
        Some(NaiveDate::from_ymd_opt(
            s.get(0..4)?.parse().ok()?,
            s.get(4..6)?.parse().ok()?,
            s.get(6..8)?.parse().ok()?,
        )?)
    })();
    matches!(parsed.map(|d| d.weekday()), Some(Weekday::Sat) | Some(Weekday::Sun))
}

fn use_day_num(date: NaiveDate) -> i64 {
    date.format("%Y%m%d").to_string().parse().unwrap_or(0)
}

#[tauri::command]
async fn search_vacant(
    state: State<'_, AppState>,
    inst_cd: String,
    day_filter: String,
) -> Result<Vec<AvailableSlot>, String> {
    let &(bld_cd, park_name, inst_cd, sport_code) = courts::COURTS
        .iter()
        .find(|(_, _, i, _)| *i == inst_cd)
        .ok_or_else(|| format!("알 수 없는 코트입니다: {}", inst_cd))?;

    let client = state.client.lock().await;

    // 로그인 없이 검색만 할 경우 세션(JSESSIONID)이 없어 이후 요청이 에러 화면을 반환하므로 먼저 세션을 확보한다.
    client.get(format!("{}/index.jsp", BASE_URL)).send().await.map_err(|e| e.to_string())?;

    let (period_start, period_end) = current_search_period();
    let period_start_num = use_day_num(period_start);
    let period_end_num = use_day_num(period_end);

    let ppscl_ppscd = format!("1000_{}", sport_code);
    let mut slots: Vec<AvailableSlot> = vec![];

    // 사이트 API가 한 번 호출에 요청한 날짜 기준 7일치만 반환하므로, 7일 간격으로 훑어서 이번달 전체를 커버한다.
    let mut anchor = period_start;
    while anchor <= period_end {
        let date_str = anchor.format("%Y-%m-%d").to_string();
        let use_day_str = anchor.format("%Y%m%d").to_string();

        let mut init_params = HashMap::new();
        init_params.insert("daystart", date_str.as_str());
        init_params.insert("useDay", use_day_str.as_str());
        init_params.insert("selectPpsClPpscd", ppscl_ppscd.as_str());
        init_params.insert("selectPpsClsCd", "1000");
        init_params.insert("selectPpsCd", sport_code);
        init_params.insert("selectBldCd", bld_cd);
        init_params.insert("selectInstCd", inst_cd);
        init_params.insert("selectAreaBcd", bld_cd);
        init_params.insert("selectIcd", "0");
        init_params.insert("penaltyday", "3");
        init_params.insert("penalty", "3");
        init_params.insert("dayofweekClearFlg", "1");
        init_params.insert("timezoneClearFlg", "1");
        init_params.insert("displayNo", "prwrc2000");
        init_params.insert("displayNoFrm", "prwrc2000");
        init_params.insert("selectSize", "0");
        init_params.insert("applyFlg", "0");
        init_params.insert("initBcd", "null");
        init_params.insert("initIcd", "null");
        init_params.insert("initPpsClPpscd", "null");

        client
            .post(format!("{}/rsvWOpeInstSrchVacantAction.do", BASE_URL))
            .form(&init_params)
            .send().await.map_err(|e| e.to_string())?;

        let mut params = HashMap::new();
        params.insert("displayNo", "prwrc2000");
        params.insert("useDay", use_day_str.as_str());
        params.insert("bldCd", bld_cd);
        params.insert("instCd", inst_cd);
        params.insert("transVacantMode", "11");
        params.insert("clearFlag", "0");

        let body = client
            .post(format!("{}/rsvWOpeInstSrchVacantAjaxAction.do", BASE_URL))
            .form(&params)
            .send().await.map_err(|e| e.to_string())?
            .text().await.map_err(|e| e.to_string())?;

        let vacant: VacantResponse = serde_json::from_str(&body)
            .map_err(|e| format!("{}: {}", e, &body[..body.len().min(200)]))?;

        for tzone in vacant.result {
            for tr in tzone.time_result {
                let in_range = tr.use_day >= period_start_num && tr.use_day <= period_end_num;
                let day_ok = match day_filter.as_str() {
                    "weekday" => !is_weekend(tr.use_day),
                    "weekend" => is_weekend(tr.use_day),
                    _ => true,
                };
                if tr.status == 0 && tr.rsv_num > 0 && in_range && day_ok {
                    slots.push(AvailableSlot {
                        park_name: park_name.to_string(),
                        tzone_name: tzone.tzone_name.trim().to_string(),
                        use_day: tr.use_day,
                        rsv_num: tr.rsv_num,
                    });
                }
            }
        }

        anchor += Duration::days(7);
    }

    Ok(slots)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .build()
        .expect("HTTP 클라이언트 생성 실패");

    tauri::Builder::default()
        .manage(AppState { client: Mutex::new(client) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login, courts::list_courts, search_vacant, mail::send_result_mail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
