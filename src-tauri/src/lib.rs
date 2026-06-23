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
    start_time: i32,
    end_time: i32,
    bld_cd: String,
    inst_cd: String,
    tzone_no: i32,
}

fn parse_hidden(html: &str, field_name: &str) -> Option<String> {
    let search = format!("name=\"{}\"", field_name);
    let name_pos = html.find(&search)?;
    // name= 위치에서 가장 가까운 <input 태그 시작점을 역방향 탐색
    let tag_start = html[..name_pos].rfind('<')?;
    let tag_end = name_pos + html[name_pos..].find('>')?;
    let tag = &html[tag_start..=tag_end];
    // 태그 안에서 value= 탐색 (속성 순서 무관)
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
                        start_time: tr.start_time,
                        end_time: tr.end_time,
                        bld_cd: bld_cd.to_string(),
                        inst_cd: inst_cd.to_string(),
                        tzone_no: tzone.tzone_no,
                    });
                }
            }
        }
    }

    Ok(slots)
}

#[tauri::command]
async fn reserve_slot(
    state: State<'_, AppState>,
    bld_cd: String,
    inst_cd: String,
    use_day: i64,
    start_time: i32,
    end_time: i32,
    tzone_no: i32,
    aki_num: i32,
    apply_num: i32,
) -> Result<String, String> {
    let client = state.client.lock().await;
    let use_day_str = use_day.to_string();
    let start_str = start_time.to_string();
    let end_str = end_time.to_string();
    let tzone_str = tzone_no.to_string();
    let aki_str = aki_num.to_string();

    // Step 1: 슬롯 선택
    let select_res = client
        .post(format!("{}/rsvWOpeInstReservAjaxAction.do", BASE_URL))
        .form(&[
            ("displayNo", "prwrc2000"),
            ("bldCd", bld_cd.as_str()),
            ("instCd", inst_cd.as_str()),
            ("useDay", use_day_str.as_str()),
            ("startTime", start_str.as_str()),
            ("endTime", end_str.as_str()),
            ("tzoneNo", tzone_str.as_str()),
            ("akiNum", aki_str.as_str()),
            ("selectNum", "0"),
        ])
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    if !select_res.contains("\"selectState\":1") {
        return Err(format!("슬롯 선택 실패: {}", select_res));
    }

    // Step 2: 예약 상세 페이지 취득 + insIRsvJKey, stimeZoneNo 파싱
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let use_date = chrono::NaiveDate::parse_from_str(&use_day_str, "%Y%m%d")
        .map_err(|e| e.to_string())?;
    let view_days: Vec<String> = (0..7)
        .map(|i| (use_date + chrono::Duration::days(i)).format("%Y%m%d").to_string())
        .collect();

    let detail_page = client
        .post(format!("{}/rsvWOpeReservedApplyAction.do", BASE_URL))
        .form(&[
            ("daystart", today.as_str()),
            ("selectPpsClPpscd", "1000_1030"),
            ("penaltyday", "3"),
            ("dayofweekClearFlg", "0"),
            ("timezoneClearFlg", "0"),
            ("selectAreaBcd", bld_cd.as_str()),
            ("selectIcd", "0"),
            ("iniBCd", bld_cd.as_str()),
            ("iniICd", inst_cd.as_str()),
            ("displayNo", "prwrc2000"),
            ("displayNoFrm", "prwrc2000"),
            ("selectSize", "1"),
            ("selectBldCd", bld_cd.as_str()),
            ("selectInstCd", inst_cd.as_str()),
            ("useDay", use_day_str.as_str()),
            ("selectPpsClsCd", "1000"),
            ("selectPpsCd", "1030"),
            ("viewDay1", view_days[0].as_str()),
            ("viewDay2", view_days[1].as_str()),
            ("viewDay3", view_days[2].as_str()),
            ("viewDay4", view_days[3].as_str()),
            ("viewDay5", view_days[4].as_str()),
            ("viewDay6", view_days[5].as_str()),
            ("viewDay7", view_days[6].as_str()),
            ("applyFlg", "1"),
            ("penalty", "3"),
            ("initBcd", "null"),
            ("initIcd", "null"),
            ("initPpsClPpscd", "null"),
        ])
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    let ins_j_key = parse_hidden(&detail_page, "insIRsvJKey")
        .ok_or_else(|| format!("insIRsvJKey 파싱 실패. 페이지 앞부분:\n{}", &detail_page[..detail_page.len().min(500)]))?;

    let stime_zone = parse_hidden(&detail_page, "stimeZoneNo")
        .unwrap_or_else(|| tzone_str.clone());
    let etime_zone = parse_hidden(&detail_page, "etimeZoneNo")
        .unwrap_or_else(|| tzone_str.clone());

    let apply_str = apply_num.to_string();

    // Step 3: 예약 확정 (recaptchaToken 빈 값으로 테스트)
    let confirm_res = client
        .post(format!("{}/rsvWInstRsvApplyAction.do", BASE_URL))
        .form(&[
            ("stimeZoneNo", stime_zone.as_str()),
            ("etimeZoneNo", etime_zone.as_str()),
            ("purpose", "1000_1030"),
            ("ppsdCd", "1000"),
            ("ppsCd", "1030"),
            ("applyNum", apply_str.as_str()),
            ("MaxApplyNum", "9999"),
            ("recaptchaToken", ""),
            ("displayNo", "prwea1000"),
            ("selectRsvDetailNo", "0"),
            ("insIRsvJKey", ins_j_key.as_str()),
        ])
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    // 결과 확인 (어떤 페이지가 반환되는지 확인)
    let snippet = &confirm_res[..confirm_res.len().min(300)];
    Ok(format!("확정 응답:\n{}", snippet))
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
        .invoke_handler(tauri::generate_handler![login, search_vacant, reserve_slot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
