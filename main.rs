use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

// --- Data Structures ---

#[derive(Serialize, Clone, Debug)]
struct ThaiLottoResult {
    #[serde(rename = "Draw Date")]
    draw_date: String,
    #[serde(rename = "First Prize")]
    first_prize: String,
    #[serde(rename = "Last 2 Digits")]
    last_2_digits: String,
}

#[derive(Serialize, Clone)]
struct TaskStatus {
    is_running: bool,
    lotto_type: Option<String>,
    progress: Vec<String>,
    results: Vec<ThaiLottoResult>,
}

impl TaskStatus {
    fn new() -> Self {
        TaskStatus {
            is_running: false,
            lotto_type: None,
            progress: Vec::new(),
            results: Vec::new(),
        }
    }
}

lazy_static! {
    static ref TASK_STATUS: Mutex<TaskStatus> = Mutex::new(TaskStatus::new());
}

// --- Web Scraper ---

async fn scrape_thai_lotto_page(
    client: &reqwest::Client,
    url: &str,
) -> Result<(Vec<ThaiLottoResult>, Option<String>), String> {
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()));
    }
    let body = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&body);

    let article_selector = Selector::parse("article.archive--lotto").unwrap();
    let date_selector = Selector::parse("time.archive--lotto__date").unwrap();
    let li_selector = Selector::parse("ul.archive--lotto__result-list li").unwrap();
    let label_selector = Selector::parse("em.archive--lotto__result-txt").unwrap();
    let number_selector = Selector::parse("strong.archive--lotto__result-number").unwrap();
    let next_button_selector = Selector::parse("a.pagination__item--next").unwrap();

    let mut page_results = Vec::new();
    for article in document.select(&article_selector) {
        let draw_date = article
            .select(&date_selector)
            .next()
            .and_then(|time| time.value().attr("datetime"))
            .unwrap_or("Unknown")
            .to_string();

        let mut first_prize = None;
        let mut last_2_digits = None;

        for li in article.select(&li_selector) {
            let label = li
                .select(&label_selector)
                .next()
                .map(|em| em.text().collect::<String>());
            let prize = li
                .select(&number_selector)
                .next()
                .map(|s| s.text().collect::<String>());

            if let (Some(label_text), Some(prize_text)) = (label, prize) {
                if label_text.contains("รางวัลที่ 1") {
                    first_prize = Some(prize_text.trim().to_string());
                } else if label_text.contains("เลขท้าย 2 ตัว") {
                    last_2_digits = Some(prize_text.trim().to_string());
                }
            }
        }

        if let (Some(fp), Some(l2d)) = (first_prize, last_2_digits) {
            page_results.push(ThaiLottoResult {
                draw_date,
                first_prize: fp,
                last_2_digits: l2d,
            });
        }
    }

    let next_page_url = document
        .select(&next_button_selector)
        .next()
        .and_then(|a| a.value().attr("href"))
        .map(|s| s.to_string());

    Ok((page_results, next_page_url))
}

async fn run_scraper() {
    let start_url = "https://news.sanook.com/lotto/archive/".to_string();
    let client = reqwest::Client::new();
    let mut all_results = Vec::new();
    let mut current_url = Some(start_url);

    while let Some(url) = current_url {
        {
            let mut status = TASK_STATUS.lock().unwrap_or_else(|e| e.into_inner());
            status.progress.push(format!("📄 Scraping page: {}", url));
        }

        match scrape_thai_lotto_page(&client, &url).await {
            Ok((mut page_results, next_url)) => {
                all_results.append(&mut page_results);
                current_url = next_url;
            }
            Err(e) => {
                let mut status = TASK_STATUS.lock().unwrap_or_else(|e| e.into_inner());
                status
                    .progress
                    .push(format!("⚠️ Error scraping page {}: {}", url, e));
                current_url = None;
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    let mut status = TASK_STATUS.lock().unwrap_or_else(|e| e.into_inner());
    status.results = all_results;
    status
        .progress
        .push("✅ Thai Lottery scraping complete.".to_string());
    status.is_running = false;
}

// --- Analysis Engine ---

#[derive(Deserialize)]
struct AnalyzeRequest {
    numbers: Vec<String>,
}

#[derive(Serialize)]
struct AnalysisResponse {
    statistical_summary: HashMap<String, String>,
    pattern_analysis: HashMap<String, serde_json::Value>,
    prediction_output: HashMap<String, serde_json::Value>,
    explanation: String,
}

fn analyze_sequence(numbers_str: &[String]) -> Result<AnalysisResponse, String> {
    if numbers_str.len() < 10 {
        return Err(format!(
            "ข้อมูลไม่เพียงพอ AI ต้องการชุดตัวเลขที่ใช้งานได้อย่างน้อย 10 ชุด แต่พบเพียง {} ชุด",
            numbers_str.len()
        ));
    }

    // Determine number length (assuming all numbers have the same length)
    let num_len = if let Some(first_num) = numbers_str.first() {
        first_num.len()
    } else {
        return Err("ข้อมูลว่างเปล่า ไม่สามารถวิเคราะห์ได้".to_string());
    };

    // Count frequency of each digit
    let mut digit_counts = HashMap::new();
    for num_str in numbers_str {
        for digit in num_str.chars() {
            if digit.is_ascii_digit() {
                *digit_counts.entry(digit).or_insert(0) += 1;
            }
        }
    }

    // Sort digits by frequency
    let mut sorted_digits: Vec<_> = digit_counts.iter().collect();
    sorted_digits.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    // Create the prediction from most frequent digits
    let mut final_prediction: String = sorted_digits
        .iter()
        .take(num_len)
        .map(|(digit, _)| **digit)
        .collect();

    // Pad prediction if not enough unique digits were found
    if final_prediction.len() < num_len {
        if let Some((most_frequent_digit, _)) = sorted_digits.first() {
            let padding =
                std::iter::repeat(**most_frequent_digit).take(num_len - final_prediction.len());
            final_prediction.extend(padding);
        } else {
            final_prediction = "0".repeat(num_len);
        }
    }

    // Create alternative predictions
    let mut alternatives = vec![];
    if sorted_digits.len() > num_len {
        let mut alt1_chars: Vec<char> = final_prediction.chars().collect();
        if let Some(swap_char) = sorted_digits.get(num_len).map(|(d, _)| **d) {
            if !alt1_chars.is_empty() {
                *alt1_chars.last_mut().unwrap() = swap_char;
                alternatives.push(alt1_chars.into_iter().collect::<String>());
            }
        }
    }
    let cold_prediction: String = sorted_digits
        .iter()
        .rev()
        .take(num_len)
        .map(|(digit, _)| **digit)
        .collect();
    if cold_prediction.len() == num_len && cold_prediction != final_prediction {
        alternatives.push(cold_prediction);
    }
    let reversed_prediction: String = final_prediction.chars().rev().collect();
    if reversed_prediction != final_prediction {
        alternatives.push(reversed_prediction);
    }
    if alternatives.is_empty() && final_prediction.len() > 1 {
        let mut chars: Vec<char> = final_prediction.chars().collect();
        chars.swap(0, 1);
        alternatives.push(chars.into_iter().collect());
    }
    alternatives.dedup();
    let final_alternatives = alternatives.into_iter().take(4).collect::<Vec<String>>();

    // Create statistical summary
    let statistical_summary = HashMap::from([
        ("Dataset Size".to_string(), numbers_str.len().to_string()),
        (
            "Unique Numbers Provided".to_string(),
            numbers_str
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                .to_string(),
        ),
        (
            "Analysis Type".to_string(),
            "Digit Frequency Analysis".to_string(),
        ),
    ]);

    // Create pattern analysis (digit frequencies)
    let most_frequent_digits_str: Vec<String> = sorted_digits
        .iter()
        .map(|(d, c)| format!("'{}' ({} times)", d, c))
        .collect();
    let pattern_analysis = HashMap::from([(
        "Digit Frequency (Most to Least)".to_string(),
        serde_json::json!(most_frequent_digits_str),
    )]);

    // Calculate confidence score
    let total_digits: i32 = digit_counts.values().sum();
    let top_digits_count: i32 = sorted_digits
        .iter()
        .take(num_len)
        .map(|(_, count)| *count)
        .sum();
    let confidence = if total_digits > 0 {
        (top_digits_count as f64 / total_digits as f64) * 100.0
    } else {
        0.0
    };
    let scaled_confidence = (50.0 + confidence / 2.0).min(95.0);

    // Create final prediction output
    let prediction_output = HashMap::from([
        ("PREDICTION".to_string(),serde_json::json!(final_prediction)),
        (
            "CONFIDENCE".to_string(),
            serde_json::json!(format!("{:.2}%", scaled_confidence)),
        ),
        (
            "METHOD".to_string(),
            serde_json::json!("Digit Frequency Combination"),
        ),
        (
            "ALTERNATIVE_PREDICTIONS".to_string(),
            serde_json::json!(final_alternatives),
        ),
    ]);

    let explanation = format!(
        "AI ได้ทำการวิเคราะห์ความถี่ของตัวเลขแต่ละหลัก (0-9) จากชุดข้อมูลทั้งหมด {} ชุด. ตัวเลขที่ทำนาย '{}' ถูกสร้างขึ้นโดยการรวมตัวเลขที่มีความถี่สูงสุด. ตัวเลขทางเลือกได้ถูกสร้างขึ้นจากรูปแบบความถี่ที่แตกต่างกัน เพื่อเป็นแนวทางเพิ่มเติม.",
        numbers_str.len(), final_prediction
    );

    Ok(AnalysisResponse {
        statistical_summary,
        pattern_analysis,
        prediction_output,
        explanation,
    })
}

// --- API Endpoints ---

#[derive(Deserialize)]
struct StartScrapeRequest {
    lotto_type: String,
}

async fn start_scrape(req: web::Json<StartScrapeRequest>) -> impl Responder {
    let mut status = TASK_STATUS.lock().unwrap_or_else(|e| e.into_inner());
    if status.is_running {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "A scraper is already running."
        }));
    }
    if req.lotto_type != "thai" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid lottery type. Only 'thai' is supported."
        }));
    }

    status.is_running = true;
    status.lotto_type = Some(req.lotto_type.clone());
    status.progress = vec!["🚀 Starting scraper for Thai Lottery...".to_string()];
    status.results.clear();

    tokio::spawn(run_scraper());

    HttpResponse::Accepted().json(serde_json::json!({
        "message": "Scraping process for Thai lottery started!"
    }))
}

async fn get_status() -> impl Responder {
    let status = TASK_STATUS.lock().unwrap_or_else(|e| e.into_inner());
    HttpResponse::Ok().json(&*status)
}

async fn analyze_handler(req: web::Json<AnalyzeRequest>) -> impl Responder {
    match analyze_sequence(&req.numbers) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

async fn index() -> impl Responder {
    if let Ok(content) = std::fs::read_to_string("templates/index.html") {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content)
    } else {
        HttpResponse::InternalServerError().body("Could not read index.html")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port_str.parse::<u16>().expect("PORT must be a valid number");

    if !std::path::Path::new("templates/index.html").exists() {
        eprintln!("❌ Error: templates/index.html not found.");
    }

    println!("🌍 Server starting at http://0.0.0.0:{}", port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/start-scrape", web::post().to(start_scrape))
            .route("/status", web::get().to(get_status))
            .route("/analyze", web::post().to(analyze_handler))
            .service(Files::new("/static", "static").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}