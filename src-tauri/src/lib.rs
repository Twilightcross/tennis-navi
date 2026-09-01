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

/// (bldCd, 표시명, instCd, selectPpsCd) — 선택 가능한 코트 목록
/// instCd가 코트 단위 고유 식별자 (같은 공원이 하드/인공잔디 시설을 모두 가진 경우가 있어 bldCd만으로는 구분 불가)
const COURTS: &[(&str, &str, &str, &str)] = &[
    // 하드코트 (selectPpsCd=1020)
    ("1310", "大井ふ頭海浜公園Ａ", "13100050", "1020"),
    ("1315", "大井ふ頭海浜公園Ｂ", "13150030", "1020"),
    ("1350", "有明テニスＡ屋外ハードコート", "13500010", "1020"),
    ("1370", "有明テニスＢインドアコート", "13700010", "1020"),
    // 인공잔디 코트 (selectPpsCd=1030)
    ("1000", "日比谷公園", "10000010", "1030"),
    ("1010", "芝公園", "10100030", "1030"),
    ("1040", "猿江恩賜公園", "10400030", "1030"),
    ("1050", "亀戸中央公園", "10500010", "1030"),
    ("1060", "木場公園", "10600010", "1030"),
    ("1070", "祖師谷公園", "10700010", "1030"),
    ("1090", "東白鬚公園", "10900030", "1030"),
    ("1100", "浮間公園", "11000020", "1030"),
    ("1110", "城北中央公園（照明有）", "11100030", "1030"),
    ("1110", "城北中央公園（照明無）", "11100130", "1030"),
    ("1120", "赤塚公園", "11200020", "1030"),
    ("1130", "東綾瀬公園", "11300040", "1030"),
    ("1140", "舎人公園", "11400030", "1030"),
    ("1150", "篠崎公園Ａ", "11500050", "1030"),
    ("1160", "大島小松川公園", "11600030", "1030"),
    ("1170", "汐入公園", "11700010", "1030"),
    ("1175", "高井戸公園", "11750030", "1030"),
    ("1180", "善福寺川緑地", "11800030", "1030"),
    ("1190", "光が丘公園", "11900050", "1030"),
    ("1205", "石神井公園Ｂ", "12050030", "1030"),
    ("1220", "井の頭恩賜公園", "12200020", "1030"),
    ("1230", "武蔵野中央公園", "12300010", "1030"),
    ("1240", "小金井公園", "12400020", "1030"),
    ("1260", "野川公園", "12600010", "1030"),
    ("1270", "府中の森公園", "12700020", "1030"),
    ("1280", "東大和南公園", "12800020", "1030"),
    ("1315", "大井ふ頭海浜公園Ｂ", "13150090", "1030"),
    ("1360", "有明テニスＣ人工芝コート", "13600010", "1030"),
];

#[tauri::command]
fn list_courts() -> Vec<(String, String, String)> {
    COURTS
        .iter()
        .map(|(_bld_cd, name, inst_cd, sport_code)| {
            (inst_cd.to_string(), name.to_string(), sport_code.to_string())
        })
        .collect()
}

#[tauri::command]
async fn search_vacant(
    state: State<'_, AppState>,
    date: String,
    inst_cd: String,
) -> Result<Vec<AvailableSlot>, String> {
    let &(bld_cd, park_name, inst_cd, sport_code) = COURTS
        .iter()
        .find(|(_, _, i, _)| *i == inst_cd)
        .ok_or_else(|| format!("알 수 없는 코트입니다: {}", inst_cd))?;

    let use_day_str = date.replace("-", "");
    let client = state.client.lock().await;

    // 로그인 없이 검색만 할 경우 세션(JSESSIONID)이 없어 이후 요청이 에러 화면을 반환하므로 먼저 세션을 확보한다.
    client.get(format!("{}/index.jsp", BASE_URL)).send().await.map_err(|e| e.to_string())?;

    let ppscl_ppscd = format!("1000_{}", sport_code);
    let mut init_params = HashMap::new();
    init_params.insert("daystart", date.as_str());
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

    let mut slots: Vec<AvailableSlot> = vec![];
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
        .invoke_handler(tauri::generate_handler![login, list_courts, search_vacant])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
