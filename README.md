# ALICE Stream SaaS

Adaptive streaming optimization powered by ALICE-Streaming-Protocol. Optimize, ingest, and analyze video streams across HLS, DASH, CMAF, WebRTC, SRT, and RTMP via a simple REST API.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Status

| Check | Status |
|-------|--------|
| `cargo check` | passing |
| `tsc --noEmit` | passing |
| API health | `/health` |

## Quick Start

```bash
docker compose up -d
```

Frontend: http://localhost:3000
API Gateway: http://localhost:8080
Stream Engine: http://localhost:8081

## Architecture

```
Browser / Client
      |
      v
Frontend (Next.js)   :3000
      |
      v
API Gateway          :8080
      |
      v
Stream Engine        :8081
(ALICE-Streaming-Protocol core)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/stream/optimize` | Optimize stream to target codec and bitrate |
| `POST` | `/api/v1/stream/ingest` | Ingest stream and transcode to output format |
| `POST` | `/api/v1/stream/analyze` | Analyze stream metadata and quality |
| `GET` | `/api/v1/stream/formats` | List supported streaming formats |
| `GET` | `/health` | Service health check |

### optimize

```json
POST /api/v1/stream/optimize
{
  "url": "rtmp://source/live/stream",
  "target_bitrate": 2500000,
  "codec": "h264"
}
```

Response:
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "output_codec": "h264",
  "optimized_bitrate": 2500000,
  "latency_ms": 42
}
```

### ingest

```json
POST /api/v1/stream/ingest
{
  "source_url": "rtmp://source/live/stream",
  "output_format": "hls"
}
```

Response:
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "ingesting",
  "output_format": "hls",
  "ingest_url": "https://cdn.example.com/hls/42/index.m3u8"
}
```

### analyze

```json
POST /api/v1/stream/analyze
{
  "url": "https://cdn.example.com/hls/stream/index.m3u8"
}
```

Response:
```json
{
  "codec": "h264",
  "bitrate": 2500000,
  "resolution": "1920x1080",
  "fps": 30.0,
  "protocol": "HLS",
  "latency_ms": 3200
}
```

## Supported Formats

| Format | Protocol | Adaptive Bitrate |
|--------|----------|-----------------|
| HLS | `hls` | Yes |
| DASH | `dash` | Yes |
| CMAF | `cmaf` | Yes |
| WebRTC | `webrtc` | No |
| SRT | `srt` | No |
| RTMP | `rtmp` | No |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STREAM_ADDR` | `0.0.0.0:8081` | Stream engine bind address |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | API gateway URL for frontend |

## License

AGPL-3.0. Commercial dual-license available — contact for pricing.
