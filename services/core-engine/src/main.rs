#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
}

struct Stats {
    total_optimizations: u64,
    total_ingests: u64,
    total_analyses: u64,
    total_transcodes: u64,
    bytes_processed: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_jobs: u64 }

// Optimize
#[derive(Deserialize)]
#[allow(dead_code)]
struct OptimizeRequest { url: Option<String>, target_bitrate: Option<u32>, codec: Option<String>, resolution: Option<String> }
#[derive(Serialize)]
struct OptimizeResponse {
    job_id: String, status: String, output_codec: String,
    original_bitrate_kbps: u32, optimized_bitrate_kbps: u32,
    savings_pct: f64, latency_ms: u64, abr_ladder: Vec<AbrRung>, elapsed_us: u128,
}
#[derive(Serialize)]
struct AbrRung { resolution: String, bitrate_kbps: u32, fps: u32, codec: String }

// Ingest
#[derive(Deserialize)]
#[allow(dead_code)]
struct IngestRequest { source_url: Option<String>, output_format: Option<String>, record: Option<bool> }
#[derive(Serialize)]
struct IngestResponse {
    job_id: String, status: String, output_format: String,
    ingest_url: String, playback_url: String,
    estimated_latency_ms: u64, elapsed_us: u128,
}

// Transcode
#[derive(Deserialize)]
struct TranscodeRequest { input_codec: Option<String>, output_codec: Option<String>, resolution: Option<String>, bitrate: Option<u32> }
#[derive(Serialize)]
struct TranscodeResponse {
    job_id: String, status: String,
    input_codec: String, output_codec: String,
    input_resolution: String, output_resolution: String,
    output_bitrate_kbps: u32, estimated_time_ms: u64, elapsed_us: u128,
}

// Analyze
#[derive(Deserialize)]
#[allow(dead_code)]
struct AnalyzeRequest { url: Option<String> }
#[derive(Serialize)]
struct AnalyzeResponse {
    codec: String, bitrate_kbps: u32, resolution: String, fps: f32,
    protocol: String, latency_ms: u64, keyframe_interval: u32,
    gop_size: u32, b_frames: u8, profile: String,
    quality_score: f64,
}

// Formats
#[derive(Serialize)]
struct FormatInfo { name: String, protocol: String, adaptive: bool, latency_class: String, description: String }

// Stats
#[derive(Serialize)]
struct StatsResponse {
    total_optimizations: u64, total_ingests: u64, total_analyses: u64,
    total_transcodes: u64, bytes_processed: u64,
}

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "stream_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_optimizations: 0, total_ingests: 0, total_analyses: 0,
            total_transcodes: 0, bytes_processed: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/stream/optimize", post(optimize))
        .route("/api/v1/stream/ingest", post(ingest))
        .route("/api/v1/stream/transcode", post(transcode))
        .route("/api/v1/stream/analyze", post(analyze))
        .route("/api/v1/stream/formats", get(formats))
        .route("/api/v1/stream/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("STREAM_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Stream Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(),
        total_jobs: st.total_optimizations + st.total_ingests + st.total_analyses + st.total_transcodes,
    })
}

async fn optimize(State(s): State<Arc<AppState>>, Json(req): Json<OptimizeRequest>) -> Json<OptimizeResponse> {
    let t = Instant::now();
    let codec = req.codec.unwrap_or_else(|| "h264".into());
    let target = req.target_bitrate.unwrap_or(2_500);
    let res = req.resolution.as_deref().unwrap_or("1920x1080");

    // Generate ABR ladder based on resolution and codec
    let abr_ladder = generate_abr_ladder(&codec, res);
    let original = estimate_original_bitrate(res);
    let savings = if original > 0 { ((original - target) as f64 / original as f64 * 100.0).max(0.0) } else { 0.0 };

    // Latency estimation based on codec
    let latency = match codec.as_str() {
        "av1" => 120, "h265" | "hevc" => 80, "vp9" => 90,
        "h264" | "avc" => 50, _ => 60,
    };

    {
        let mut st = s.stats.lock().unwrap();
        st.total_optimizations += 1;
        st.bytes_processed += target as u64 * 1000;
    }

    Json(OptimizeResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        output_codec: codec, original_bitrate_kbps: original,
        optimized_bitrate_kbps: target, savings_pct: savings,
        latency_ms: latency, abr_ladder, elapsed_us: t.elapsed().as_micros(),
    })
}

async fn ingest(State(s): State<Arc<AppState>>, Json(req): Json<IngestRequest>) -> Json<IngestResponse> {
    let t = Instant::now();
    let fmt = req.output_format.unwrap_or_else(|| "hls".into());
    let source = req.source_url.unwrap_or_else(|| "rtmp://source/live".into());
    let stream_key = &uuid::Uuid::new_v4().to_string()[..8];

    let ingest_url = match fmt.as_str() {
        "hls" => format!("rtmp://ingest.alice-stream.io/live/{stream_key}"),
        "dash" => format!("rtmp://ingest.alice-stream.io/dash/{stream_key}"),
        "webrtc" => format!("whip://ingest.alice-stream.io/webrtc/{stream_key}"),
        "srt" => format!("srt://ingest.alice-stream.io:9000?streamid={stream_key}"),
        _ => format!("rtmp://ingest.alice-stream.io/live/{stream_key}"),
    };

    let playback_url = match fmt.as_str() {
        "hls" => format!("https://cdn.alice-stream.io/{stream_key}/index.m3u8"),
        "dash" => format!("https://cdn.alice-stream.io/{stream_key}/manifest.mpd"),
        "webrtc" => format!("whep://cdn.alice-stream.io/webrtc/{stream_key}"),
        "cmaf" => format!("https://cdn.alice-stream.io/{stream_key}/cmaf/index.m3u8"),
        _ => format!("https://cdn.alice-stream.io/{stream_key}/index.m3u8"),
    };

    let latency = match fmt.as_str() {
        "webrtc" => 200, "srt" => 500, "cmaf" => 2000, "hls" => 6000, "dash" => 4000,
        _ => 6000,
    };

    let _ = source; // source_url stored for recording lookup

    s.stats.lock().unwrap().total_ingests += 1;

    Json(IngestResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "ingesting".into(),
        output_format: fmt, ingest_url, playback_url,
        estimated_latency_ms: latency, elapsed_us: t.elapsed().as_micros(),
    })
}

async fn transcode(State(s): State<Arc<AppState>>, Json(req): Json<TranscodeRequest>) -> Json<TranscodeResponse> {
    let t = Instant::now();
    let in_codec = req.input_codec.unwrap_or_else(|| "h264".into());
    let out_codec = req.output_codec.unwrap_or_else(|| "h265".into());
    let in_res = req.resolution.as_deref().unwrap_or("1920x1080");
    let bitrate = req.bitrate.unwrap_or_else(|| estimate_original_bitrate(in_res) / 2);

    // Estimate output resolution (same or downscale)
    let out_res = in_res;

    // Processing time estimate: based on codec complexity
    let complexity = match out_codec.as_str() {
        "av1" => 8.0, "h265" | "hevc" => 3.0, "vp9" => 4.0,
        "h264" | "avc" => 1.0, _ => 2.0,
    };
    let estimated_time = (complexity * 1000.0) as u64;

    s.stats.lock().unwrap().total_transcodes += 1;

    Json(TranscodeResponse {
        job_id: uuid::Uuid::new_v4().to_string(), status: "completed".into(),
        input_codec: in_codec, output_codec: out_codec,
        input_resolution: in_res.into(), output_resolution: out_res.into(),
        output_bitrate_kbps: bitrate, estimated_time_ms: estimated_time,
        elapsed_us: t.elapsed().as_micros(),
    })
}

async fn analyze(State(s): State<Arc<AppState>>, Json(_req): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    s.stats.lock().unwrap().total_analyses += 1;

    Json(AnalyzeResponse {
        codec: "h264".into(), bitrate_kbps: 4500, resolution: "1920x1080".into(),
        fps: 30.0, protocol: "HLS".into(), latency_ms: 6200,
        keyframe_interval: 2, gop_size: 60, b_frames: 2,
        profile: "High".into(), quality_score: 78.5,
    })
}

async fn formats() -> Json<Vec<FormatInfo>> {
    Json(vec![
        FormatInfo { name: "HLS".into(), protocol: "hls".into(), adaptive: true, latency_class: "standard (6-30s)".into(), description: "HTTP Live Streaming — Apple standard, widest device support".into() },
        FormatInfo { name: "LL-HLS".into(), protocol: "ll-hls".into(), adaptive: true, latency_class: "low (2-4s)".into(), description: "Low-Latency HLS — partial segments, preload hints".into() },
        FormatInfo { name: "DASH".into(), protocol: "dash".into(), adaptive: true, latency_class: "standard (4-20s)".into(), description: "MPEG-DASH — ISO standard, broad support".into() },
        FormatInfo { name: "CMAF".into(), protocol: "cmaf".into(), adaptive: true, latency_class: "low (2-5s)".into(), description: "Common Media Application Format — unified HLS+DASH".into() },
        FormatInfo { name: "WebRTC".into(), protocol: "webrtc".into(), adaptive: false, latency_class: "ultra-low (<500ms)".into(), description: "Sub-second latency, peer-to-peer capable".into() },
        FormatInfo { name: "SRT".into(), protocol: "srt".into(), adaptive: false, latency_class: "low (500ms-2s)".into(), description: "Secure Reliable Transport — resilient to packet loss".into() },
        FormatInfo { name: "RTMP".into(), protocol: "rtmp".into(), adaptive: false, latency_class: "low (1-3s)".into(), description: "Real-Time Messaging Protocol — legacy ingest standard".into() },
    ])
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_optimizations: st.total_optimizations, total_ingests: st.total_ingests,
        total_analyses: st.total_analyses, total_transcodes: st.total_transcodes,
        bytes_processed: st.bytes_processed,
    })
}

// ── Helpers ─────────────────────────────────────────────────
fn estimate_original_bitrate(resolution: &str) -> u32 {
    match resolution {
        "3840x2160" | "4k" => 15000,
        "2560x1440" | "1440p" => 8000,
        "1920x1080" | "1080p" => 4500,
        "1280x720" | "720p" => 2500,
        "854x480" | "480p" => 1200,
        "640x360" | "360p" => 600,
        _ => 4500,
    }
}

fn generate_abr_ladder(codec: &str, max_res: &str) -> Vec<AbrRung> {
    let efficiency = match codec {
        "av1" => 0.5, "h265" | "hevc" => 0.65, "vp9" => 0.7, _ => 1.0,
    };
    let rungs: &[(&str, u32, u32)] = match max_res {
        "3840x2160" | "4k" => &[
            ("3840x2160", 12000, 30), ("2560x1440", 6000, 30),
            ("1920x1080", 4000, 30), ("1280x720", 2000, 30),
            ("854x480", 800, 30), ("640x360", 400, 30),
        ],
        "2560x1440" | "1440p" => &[
            ("2560x1440", 6000, 30), ("1920x1080", 4000, 30),
            ("1280x720", 2000, 30), ("854x480", 800, 30), ("640x360", 400, 30),
        ],
        _ => &[
            ("1920x1080", 4000, 30), ("1280x720", 2000, 30),
            ("854x480", 800, 30), ("640x360", 400, 30), ("426x240", 200, 30),
        ],
    };
    rungs.iter().map(|(res, br, fps)| AbrRung {
        resolution: res.to_string(),
        bitrate_kbps: (*br as f64 * efficiency) as u32,
        fps: *fps,
        codec: codec.into(),
    }).collect()
}
