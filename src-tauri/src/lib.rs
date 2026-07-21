use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::State;

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

#[derive(Serialize)]
struct AvailableSlot {
    park_name: String,
    tzone_name: String,
    use_day: i64,
    rsv_num: i32,
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

#[tauri::command]
async fn search_vacant(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<AvailableSlot>, String> {
    let use_day_str = date.replace("-", "");
    let client = state.client.lock().await;

    let parks = vec![
        ("東白鬚公園", "1090", "10900030"),
    ];

    let mut slots: Vec<AvailableSlot> = vec![];

    for (park_name, bld_cd, inst_cd) in &parks {
        let mut init_params = HashMap::new();
        init_params.insert("daystart", date.as_str());
        init_params.insert("useDay", use_day_str.as_str());
        init_params.insert("selectPpsClPpscd", "1000_1030");
        init_params.insert("selectPpsClsCd", "1000");
        init_params.insert("selectPpsCd", "1030");
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
                if tr.status == 0 && tr.rsv_num > 0 {
                    slots.push(AvailableSlot {
                        park_name: park_name.to_string(),
                        tzone_name: tzone.tzone_name.trim().to_string(),
                        use_day: tr.use_day,
                        rsv_num: tr.rsv_num,
                    });
                }
            }
        }
    }

    Ok(slots)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .build()
        .expect("HTTP 클라이언트 생성 실패");

    tauri::Builder::default()
        .manage(AppState { client: Mutex::new(client) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login, search_vacant])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
