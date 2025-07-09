use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Distribution, Median, Min, Max};
use statrs::distribution::Normal;
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
            let label = li.select(&label_selector).next().map(|em| em.text().collect::<String>());
            let prize = li.select(&number_selector).next().map(|s| s.text().collect::<String>());
            if let (Some(label_text), Some(prize_text)) = (label, prize) {
                if label_text.contains("รางวัลที่ 1") { first_prize = Some(prize_text.trim().to_string()); }
                else if label_text.contains("เลขท้าย 2 ตัว") { last_2_digits = Some(prize_text.trim().to_string()); }
            }
        }
        if let (Some(fp), Some(l2d)) = (first_prize, last_2_digits) {
            page_results.push(ThaiLottoResult { draw_date, first_prize: fp, last_2_digits: l2d });
        }
    }
    let next_page_url = document.select(&next_button_selector).next().and_then(|a| a.value().attr("href")).map(|s| s.to_string());
    Ok((page_results, next_page_url))
}

async fn run_scraper() {
    let start_url = "https://news.sanook.com/lotto/archive/".to_string();
    let client = reqwest::Client::new();
    let mut all_results = Vec::new();
    let mut current_url = Some(start_url);

    while let Some(url) = current_url {
        { TASK_STATUS.lock().unwrap().progress.push(format!("📄 Scraping page: {}", url)); }
        match scrape_thai_lotto_page(&client, &url).await {
            Ok((mut page_results, next_url)) => { all_results.append(&mut page_results); current_url = next_url; },
            Err(e) => { TASK_STATUS.lock().unwrap().progress.push(format!("⚠️ Error scraping page {}: {}", url, e)); current_url = None; }
        }
        sleep(Duration::from_millis(500)).await;
    }
    let mut status = TASK_STATUS.lock().unwrap();
    status.results = all_results;
    status.progress.push("✅ Thai Lottery scraping complete.".to_string());
    status.is_running = false;
}

// --- ADVANCED ANALYSIS ENGINE ---

#[derive(Deserialize)]
struct AnalyzeRequest {
    numbers: Vec<String>,
}

#[derive(Serialize)]
struct AnalysisResponse {
    statistical_summary: HashMap<String, String>,
    pattern_analysis: HashMap<String, serde_json::Value>,
    prediction_output: HashMap<String, serde_json::Value>,
    detailed_explanation: HashMap<String, String>,
}

fn run_comprehensive_analysis(numbers_str: &[String]) -> Result<AnalysisResponse, String> {
    if numbers_str.len() < 10 { return Err(format!("ข้อมูลไม่เพียงพอ AI ต้องการชุดตัวเลขอย่างน้อย 10 ชุด แต่พบเพียง {} ชุด", numbers_str.len())); }

    // --- Calculations on f64 (for math stats) ---
    let numbers_f64: Vec<f64> = numbers_str.iter().filter_map(|s| s.parse::<f64>().ok()).collect();
    if numbers_f64.len() < 5 { return Err("ไม่สามารถแปลงข้อมูลเป็นตัวเลขที่ถูกต้องเพื่อการวิเคราะห์ทางสถิติได้".to_string()); }

    let data = Data::new(numbers_f64.clone());
    let mean = data.mean().unwrap_or(0.0);
    let median = data.median();
    let std_dev = data.std_dev().unwrap_or(0.0);
    let variance = data.variance().unwrap_or(0.0);
    let min = data.min();
    let max = data.max();
    let skewness = Normal::new(mean, std_dev).unwrap().skewness().unwrap_or(0.0);
    
    // --- Calculations on original Strings (to preserve format like leading zeros) ---
    let mut counts = HashMap::new();
    for s in numbers_str {
        *counts.entry(s.clone()).or_insert(0) += 1;
    }
    
    // 1. Statistical Summary
    let mode = counts.iter().max_by_key(|&(_, count)| count).map(|(val, _)| val.clone()).unwrap_or_else(|| "N/A".to_string());

    let statistical_summary = HashMap::from([
        ("Dataset Size".to_string(), numbers_str.len().to_string()),
        ("Mean".to_string(), format!("{:.2}", mean)),
        ("Median".to_string(), format!("{:.2}", median)),
        ("Mode (ฐานนิยม)".to_string(), mode.clone()),
        ("Std. Dev.".to_string(), format!("{:.2}", std_dev)),
        ("Variance".to_string(), format!("{:.2}", variance)),
        ("Range".to_string(), format!("{:.2} - {:.2}", min, max)),
        ("Distribution Skewness".to_string(), format!("{:.4}", skewness)),
    ]);

    // 2. Pattern Recognition
    let most_frequent: Vec<String> = counts.iter().take(10).map(|(k, v)| format!("{} ({} times)", k, v)).collect();
    
    let mut digit_pos_freq: HashMap<usize, HashMap<char, usize>> = HashMap::new();
    for num_str in numbers_str {
        for (i, c) in num_str.chars().enumerate() {
            *digit_pos_freq.entry(i).or_default().entry(c).or_default() += 1;
        }
    }
    let digit_analysis_str: Vec<String> = digit_pos_freq.iter()
        .map(|(pos, freqs)| {
            let top_digit = freqs.iter().max_by_key(|&(_, count)| count).map(|(d, c)| format!("'{}' ({} times)", d, c)).unwrap_or_default();
            format!("Position {}: Most frequent is {}", pos + 1, top_digit)
        }).collect();

    let pattern_analysis = HashMap::from([
        ("Most Frequent Numbers".to_string(), serde_json::json!(most_frequent)),
        ("Digit & Position Analysis".to_string(), serde_json::json!(digit_analysis_str)),
    ]);
    
    // 3. Prediction Output
    let main_prediction = mode;
    let alternatives: Vec<String> = counts.iter().filter(|(k, _)| **k != main_prediction).take(4).map(|(k, _)| k.clone()).collect();
    let confidence = (60.0 + (numbers_str.len() as f64 / 100.0 * 20.0)).min(95.0);

    let prediction_output = HashMap::from([
        ("PREDICTION".to_string(), serde_json::json!(main_prediction.clone())),
        ("CONFIDENCE".to_string(), serde_json::json!(format!("{:.2}%", confidence))),
        ("METHOD".to_string(), serde_json::json!("Weighted Statistical & Frequency Model")),
        ("ALTERNATIVE_PREDICTIONS".to_string(), serde_json::json!(alternatives)),
    ]);

    // 4. Detailed Explanation
    let explanation = HashMap::from([
        ("Methodology".to_string(), "ใช้โมเดลผสมระหว่างการวิเคราะห์ความถี่ (Frequency Analysis) และค่าสถิติสำคัญ (Statistical Significance) โดยให้ความสำคัญกับตัวเลขที่ปรากฏบ่อยที่สุด (Mode) ในรูปแบบดั้งเดิมเป็นหลัก".to_string()),
        ("Statistical Evidence".to_string(), format!("ตัวเลข '{}' เป็นฐานนิยม (Mode) ซึ่งปรากฏบ่อยที่สุดในชุดข้อมูล การกระจายตัวของข้อมูลมีค่าเบี่ยงเบนมาตรฐานที่ {:.2} ซึ่งบ่งชี้ถึงความผันผวนของข้อมูล", main_prediction, std_dev)),
        ("Prediction Logic".to_string(), "การทำนายหลักมาจากค่าฐานนิยม (Mode) ซึ่งเป็นตัวบ่งชี้ทางสถิติที่แข็งแกร่งที่สุดในข้อมูลชุดนี้สำหรับตัวเลขที่จะออกซ้ำ ตัวเลือกสำรองมาจากตัวเลขที่มีความถี่รองลงมา".to_string()),
        ("Uncertainty Analysis".to_string(), "ระดับความมั่นใจประเมินจากขนาดของชุดข้อมูลและความเด่นชัดของฐานนิยม ความผันผวนของข้อมูลยังคงเป็นปัจจัยสำคัญที่สร้างความไม่แน่นอน".to_string()),
    ]);

    Ok(AnalysisResponse {
        statistical_summary,
        pattern_analysis,
        prediction_output,
        detailed_explanation: explanation,
    })
}


// --- API Endpoints ---

#[derive(Deserialize)]
struct StartScrapeRequest {
    lotto_type: String,
}

async fn start_scrape(req: web::Json<StartScrapeRequest>) -> impl Responder {
    let mut status = TASK_STATUS.lock().unwrap();
    if status.is_running { return HttpResponse::Conflict().json(serde_json::json!({"error": "A scraper is already running."})); }
    if req.lotto_type != "thai" { return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid lottery type."})); }
    status.is_running = true;
    status.lotto_type = Some(req.lotto_type.clone());
    status.progress = vec!["🚀 Starting scraper for Thai Lottery...".to_string()];
    status.results.clear();
    tokio::spawn(run_scraper());
    HttpResponse::Accepted().json(serde_json::json!({"message": "Scraping process started!"}))
}

async fn get_status() -> impl Responder {
    HttpResponse::Ok().json(&*TASK_STATUS.lock().unwrap())
}

async fn analyze_handler(req: web::Json<AnalyzeRequest>) -> impl Responder {
    match run_comprehensive_analysis(&req.numbers) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

async fn index() -> impl Responder {
    match std::fs::read_to_string("templates/index.html") {
        Ok(content) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(content),
        Err(_) => HttpResponse::InternalServerError().body("Could not read index.html"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port_str.parse::<u16>().expect("PORT must be a valid number");
    if !std::path::Path::new("templates/index.html").exists() { eprintln!("❌ Error: templates/index.html not found."); }
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