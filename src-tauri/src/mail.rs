use chrono::{Datelike, NaiveDate, Weekday};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::BTreeMap;

use crate::{current_search_period, AvailableSlot};

/// 고정 수신 주소로 검색 결과 메일 발송 (Gmail SMTP)
const MAIL_TO: &str = "ub3679@gmail.com";
const RESERVE_SITE_URL: &str = "https://kouen.sports.metro.tokyo.lg.jp/web/index.jsp";

fn weekday_ja(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "月",
        Weekday::Tue => "火",
        Weekday::Wed => "水",
        Weekday::Thu => "木",
        Weekday::Fri => "金",
        Weekday::Sat => "土",
        Weekday::Sun => "日",
    }
}

fn format_date(date: NaiveDate) -> String {
    format!("{}({})", date.format("%Y/%m/%d"), weekday_ja(date.weekday()))
}

fn format_use_day(use_day: i64) -> String {
    let s = use_day.to_string();
    let parsed = (|| {
        Some(NaiveDate::from_ymd_opt(
            s.get(0..4)?.parse().ok()?,
            s.get(4..6)?.parse().ok()?,
            s.get(6..8)?.parse().ok()?,
        )?)
    })();
    match parsed {
        Some(d) => format_date(d),
        None => s,
    }
}

#[tauri::command]
pub async fn send_result_mail(
    court_name: String,
    slots: Vec<AvailableSlot>,
) -> Result<String, String> {
    let smtp_user = std::env::var("SMTP_USER")
        .map_err(|_| "SMTP_USER 환경변수가 설정되지 않았습니다.".to_string())?;
    let smtp_pass = std::env::var("SMTP_PASS")
        .map_err(|_| "SMTP_PASS 환경변수가 설정되지 않았습니다.".to_string())?;

    let (period_start, period_end) = current_search_period();
    let period_label = format!("{} 〜 {}", format_date(period_start), format_date(period_end));

    let count = slots.len();
    let count_color = if count > 0 { "#16a34a" } else { "#dc2626" };

    let day_groups: String = if slots.is_empty() {
        "<p style=\"color:#6b7280; margin:0;\">빈 자리가 없습니다.</p>".to_string()
    } else {
        let mut grouped: BTreeMap<i64, Vec<&AvailableSlot>> = BTreeMap::new();
        for s in &slots {
            grouped.entry(s.use_day).or_default().push(s);
        }

        grouped
            .iter()
            .map(|(use_day, day_slots)| {
                let rows: String = day_slots
                    .iter()
                    .map(|s| {
                        format!(
                            "<tr><td style=\"padding:6px 12px; color:#374151;\">{}</td><td style=\"padding:6px 12px; text-align:right; font-weight:600; color:#111827;\">{}区画</td></tr>",
                            s.tzone_name, s.rsv_num
                        )
                    })
                    .collect();
                format!(
                    r#"<div style="border:1px solid #e5e7eb; border-radius:8px; overflow:hidden; margin-bottom:12px;">
  <div style="background:#f3f4f6; padding:8px 12px; font-weight:700; color:#111827;">{}</div>
  <table style="width:100%; border-collapse:collapse;">{}</table>
</div>"#,
                    format_use_day(*use_day),
                    rows
                )
            })
            .collect()
    };

    let html = format!(
        r#"<div style="font-family:-apple-system,sans-serif; max-width:480px; margin:0 auto; padding:24px; border:1px solid #e5e7eb; border-radius:12px;">
  <h2 style="margin:0 0 12px; color:#111827;">🎾 tennis-navi 検索結果 🔍</h2>
  <p style="color:#374151; margin:0 0 16px;">設定した条件で検索した結果を送ります.</p>
  <table style="width:100%; border-collapse:collapse; margin-bottom:16px;">
    <tr><td style="padding:4px 8px; color:#6b7280;">期間</td><td style="padding:4px 8px; font-weight:600;">{period_label}</td></tr>
    <tr><td style="padding:4px 8px; color:#6b7280;">コート</td><td style="padding:4px 8px; font-weight:600;">{court_name}</td></tr>
    <tr><td style="padding:4px 8px; color:#6b7280;">空席</td><td style="padding:4px 8px; font-weight:700; color:{count_color};">{count}件</td></tr>
  </table>
  <div style="margin-bottom:8px;">
    {day_groups}
  </div>
  <a href="{RESERVE_SITE_URL}" style="display:inline-block; padding:10px 20px; background:#2563eb; color:#ffffff; text-decoration:none; border-radius:8px; font-weight:600;">予約サイトへ移動</a>
</div>"#
    );

    let email = Message::builder()
        .from(smtp_user.parse().map_err(|e| format!("보내는 주소 오류: {}", e))?)
        .to(MAIL_TO.parse().map_err(|e| format!("받는 주소 오류: {}", e))?)
        .subject(format!("[tennis-navi] {} 빈 코트 검색 결과", court_name))
        .header(ContentType::TEXT_HTML)
        .body(html)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    mailer.send(email).await.map_err(|e| e.to_string())?;

    Ok("메일 발송 성공".to_string())
}
