use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::State;

fn to_sjis_form(params: &[(&str, &str)]) -> String {
    let (encoder, _, _) = encoding_rs::SHIFT_JIS.new_encoder().encoding().encode("dummy");
    let _ = encoder;
    params.iter().map(|(k, v)| {
        let enc_k = percent_encode_sjis(k);
        let enc_v = percent_encode_sjis(v);
        format!("{}={}", enc_k, enc_v)
    }).collect::<Vec<_>>().join("&")
}

fn percent_encode_sjis(s: &str) -> String {
    let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(s);
    bytes.iter().map(|&b| {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            (b as char).to_string()
        } else {
            format!("%{:02X}", b)
        }
    }).collect()
}

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
    let tag_start = html[..name_pos].rfind('<')?;
    let tag_end = name_pos + html[name_pos..].find('>')?;
    let tag = &html[tag_start..=tag_end];
    let v_start = tag.find("value=\"")? + 7;
    let v_end = v_start + tag[v_start..].find('"')?;
    Some(tag[v_start..v_end].to_string())
}

// 페이지의 모든 hidden input 필드를 추출
fn parse_all_hidden(html: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut pos = 0;
    while let Some(start) = html[pos..].find("<input") {
        let abs_start = pos + start;
        let tag_end = match html[abs_start..].find('>') {
            Some(e) => abs_start + e + 1,
            None => break,
        };
        let tag = &html[abs_start..tag_end];
        if tag.contains("type=\"hidden\"") || tag.contains("type='hidden'") {
            let name = extract_attr(tag, "name");
            let value = extract_attr(tag, "value");
            if let Some(n) = name {
                result.push((n, value.unwrap_or_default()));
            }
        }
        pos = tag_end;
    }
    result
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let search = format!("{}=\"", attr);
    let start = tag.find(&search)? + search.len();
    let end = start + tag[start..].find('"')?;
    Some(tag[start..end].to_string())
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

    // Step 1: 슬롯 선택 (AJAX)
    let select_res = client
        .post(format!("{}/rsvWOpeInstReservAjaxAction.do", BASE_URL))
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Referer", format!("{}/rsvWOpeInstSrchVacantAction.do", BASE_URL))
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

    eprintln!("[DEBUG] select_res: {}", select_res);

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
            ("iniICd", format!("{}_{}", inst_cd, aki_num).as_str()),
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

    parse_hidden(&detail_page, "insIRsvJKey")
        .ok_or_else(|| format!("insIRsvJKey 파싱 실패. 페이지 앞부분:\n{}", &detail_page[..detail_page.len().min(500)]))?;

    // 상세 페이지의 hidden 필드 전부 추출
    let mut hidden_fields = parse_all_hidden(&detail_page);
    eprintln!("[DEBUG] hidden fields: {:?}", hidden_fields.iter().map(|(k,_)| k).collect::<Vec<_>>());

    // gRecaptcha 관련 변수 탐색
    for keyword in &["gRecaptchaActive", "gRecaptchaSiteKey", "gRecaptchaErrMsg", "gRecaptchaActionName"] {
        if let Some(pos) = detail_page.find(keyword) {
            let s = pos.saturating_sub(10);
            let e = detail_page.len().min(pos + 200);
            eprintln!("[DEBUG] {}: {}", keyword, &detail_page[s..e]);
        } else {
            eprintln!("[DEBUG] {} : 없음", keyword);
        }
    }

    let apply_str = apply_num.to_string();

    // 우리가 추가하거나 덮어쓸 필드
    let overrides = vec![
        ("purpose", "1000_1030"),
        ("applyNum", apply_str.as_str()),
        ("MaxApplyNum", "9999"),
        ("eventName", ""),
        ("applyFlg", "1"),
        ("recaptchaToken", "Failed to load reCAPTCHA JavaScript."),
    ];
    for (k, v) in &overrides {
        if let Some(entry) = hidden_fields.iter_mut().find(|(name, _)| name == k) {
            entry.1 = v.to_string();
        } else {
            hidden_fields.push((k.to_string(), v.to_string()));
        }
    }

    let confirm_params: Vec<(&str, &str)> = hidden_fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let body = to_sjis_form(&confirm_params);
    eprintln!("[DEBUG] confirm body (앞 600자): {}", &body[..body.len().min(600)]);
    let confirm_res = client
        .post(format!("{}/rsvWInstRsvApplyAction.do", BASE_URL))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send().await.map_err(|e| e.to_string())?
        .text().await.map_err(|e| e.to_string())?;

    // body 태그 이후 내용만 추출해서 반환
    let body_content = if let Some(pos) = confirm_res.find("<body") {
        &confirm_res[pos..confirm_res.len().min(pos + 3000)]
    } else {
        &confirm_res[..confirm_res.len().min(3000)]
    };
    Ok(format!("확정 응답:\n{}", body_content))
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
