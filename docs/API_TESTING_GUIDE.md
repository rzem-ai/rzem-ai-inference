# API Testing Guide

Quick reference for testing the RZEM AI Inference server API.

## Starting the Server

```bash
# Development build
cargo run -- --server --port 8080

# Release build (faster)
./src-tauri/target/release/rzem-ai-inference --server --port 8080
```

## Environment Variables

```bash
# Set base URL for convenience
export API_BASE="http://localhost:8080/api/v1"
```

## API Endpoints

### 1. Health Check

```bash
curl $API_BASE/health
```

Expected response:
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### 2. List Available Models

```bash
curl $API_BASE/models
```

Expected response:
```json
[
  {
    "id": "schnell",
    "name": "FLUX Schnell",
    "is_downloaded": true
  },
  {
    "id": "dev",
    "name": "FLUX Dev",
    "is_downloaded": false
  }
]
```

### 3. Get System Stats

```bash
curl $API_BASE/system/stats
```

Expected response:
```json
{
  "cpu_usage": 15.3,
  "memory_used": 8589934592,
  "memory_total": 17179869184,
  "memory_percent": 50.0,
  "gpu_memory_used": 4294967296,
  "gpu_memory_total": 8589934592,
  "gpu_usage_percent": 25.5,
  "gpu_name": "NVIDIA GeForce RTX 4090",
  "is_generating": false
}
```

### 4. Submit Generation Job

```bash
curl -X POST $API_BASE/generate \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "a serene mountain landscape at sunset",
    "steps": 20,
    "cfg_scale": 7.5,
    "width": 1024,
    "height": 1024,
    "seed": -1,
    "model": "schnell"
  }'
```

Expected response:
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "queue_position": 0
}
```

**Save the job_id for subsequent requests!**

### 5. List All Queue Jobs

```bash
curl $API_BASE/queue
```

Expected response:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "params": {
      "prompt": "a serene mountain landscape at sunset",
      "negative_prompt": null,
      "steps": 20,
      "cfg_scale": 7.5,
      "width": 1024,
      "height": 1024,
      "seed": -1,
      "model": "schnell",
      "sampler": null,
      "scheduler": null
    },
    "status": "running",
    "progress": 0.45,
    "created_at": 1737648000,
    "started_at": 1737648001,
    "completed_at": null,
    "result_path": null,
    "error": null
  }
]
```

### 6. Get Specific Job Status

```bash
# Replace {job_id} with actual job ID
curl $API_BASE/queue/{job_id}
```

Example:
```bash
curl $API_BASE/queue/550e8400-e29b-41d4-a716-446655440000
```

### 7. Cancel a Job

```bash
# Replace {job_id} with actual job ID
curl -X DELETE $API_BASE/queue/{job_id}
```

Expected response:
```json
{
  "success": true,
  "message": "Job cancelled"
}
```

### 8. Download Generated Image

```bash
# Replace {filename} with actual filename from completed job
curl $API_BASE/files/{filename} -o output.png
```

Example:
```bash
curl $API_BASE/files/flux_1737648123_42.png -o my_image.png
```

## WebSocket Testing

### Using websocat (CLI tool)

Install websocat:
```bash
# Ubuntu/Debian
sudo apt install websocat

# macOS
brew install websocat

# Or download from: https://github.com/vi/websocat
```

Connect to WebSocket:
```bash
websocat ws://localhost:8080/api/v1/ws
```

Send messages:
```json
{"type": "Ping"}
{"type": "Subscribe", "job_id": "550e8400-e29b-41d4-a716-446655440000"}
{"type": "Unsubscribe", "job_id": "550e8400-e29b-41d4-a716-446655440000"}
```

### Using JavaScript (Browser Console)

```javascript
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');

ws.onopen = () => {
  console.log('Connected!');
  ws.send(JSON.stringify({ type: 'Ping' }));
};

ws.onmessage = (event) => {
  console.log('Received:', JSON.parse(event.data));
};

// Subscribe to job updates
ws.send(JSON.stringify({
  type: 'Subscribe',
  job_id: '550e8400-e29b-41d4-a716-446655440000'
}));
```

## Full Workflow Example

```bash
#!/bin/bash
API_BASE="http://localhost:8080/api/v1"

# 1. Check server is running
echo "1. Health check..."
curl $API_BASE/health
echo -e "\n"

# 2. Check available models
echo "2. Available models..."
curl $API_BASE/models
echo -e "\n"

# 3. Submit generation job
echo "3. Submitting job..."
RESPONSE=$(curl -s -X POST $API_BASE/generate \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "a cat on a windowsill",
    "steps": 4,
    "cfg_scale": 7.5,
    "width": 512,
    "height": 512,
    "seed": 42,
    "model": "schnell"
  }')

echo $RESPONSE
JOB_ID=$(echo $RESPONSE | jq -r '.job_id')
echo "Job ID: $JOB_ID"
echo -e "\n"

# 4. Poll job status
echo "4. Checking job status..."
while true; do
  STATUS=$(curl -s $API_BASE/queue/$JOB_ID | jq -r '.status')
  PROGRESS=$(curl -s $API_BASE/queue/$JOB_ID | jq -r '.progress')

  echo "Status: $STATUS, Progress: $PROGRESS"

  if [ "$STATUS" = "completed" ] || [ "$STATUS" = "failed" ]; then
    break
  fi

  sleep 2
done
echo -e "\n"

# 5. Download result
if [ "$STATUS" = "completed" ]; then
  echo "5. Downloading result..."
  FILENAME=$(curl -s $API_BASE/queue/$JOB_ID | jq -r '.result_path' | xargs basename)
  curl $API_BASE/files/$FILENAME -o result.png
  echo "Saved to result.png"
fi
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid request parameters"
}
```

### 404 Not Found
```json
{
  "error": "Job not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```

## Performance Testing

### Load Testing with Apache Bench

```bash
# 100 requests, 10 concurrent
ab -n 100 -c 10 http://localhost:8080/api/v1/health

# POST requests with JSON payload
ab -n 10 -c 2 -p generate.json -T application/json \
  http://localhost:8080/api/v1/generate
```

### Load Testing with wrk

```bash
# 10 second test, 2 threads, 10 connections
wrk -t2 -c10 -d10s http://localhost:8080/api/v1/health
```

## Monitoring

### Watch Queue Status

```bash
watch -n 1 'curl -s http://localhost:8080/api/v1/queue | jq'
```

### Monitor System Stats

```bash
watch -n 2 'curl -s http://localhost:8080/api/v1/system/stats | jq'
```

## Troubleshooting

### Server not starting?
```bash
# Check if port is already in use
lsof -i :8080

# Try a different port
./rzem-ai-inference --server --port 8081
```

### Connection refused?
```bash
# Verify server is running
ps aux | grep rzem-ai-inference

# Check server logs
# (Logs go to stdout when running from terminal)
```

### File not found when downloading?
```bash
# Check the result_path field from job status
curl http://localhost:8080/api/v1/queue/{job_id} | jq '.result_path'

# List files in outputs directory
ls ~/.rzem-ai-inference/outputs/
```

## Next Steps

- **Phase 2**: Implement full WebSocket subscriptions for real-time updates
- **Phase 3**: Add authentication and rate limiting
- **Phase 4**: Build client mode for remote inference

---

For more information, see `PHASE1_COMPLETE.md`
