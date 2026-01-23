# Phase 3 Client Mode - Testing Guide

This guide covers testing the client mode implementation, including connection setup, job submission, and real-time updates.

---

## Prerequisites

- Completed build of the application
- Two machines or one machine with two instances (recommended: use different ports)
- Network connectivity between server and client

---

## Test Setup

### Option 1: Single Machine Testing (Recommended for Development)

#### Terminal 1: Start Server
```bash
cd src-tauri
cargo build --release

# Start in server mode on port 8080
../target/release/rzem-ai-inference --server --port 8080
```

The desktop UI should open normally. The server is now listening on `http://localhost:8080`.

#### Terminal 2: Start Client
```bash
# Start in client mode, connecting to the local server
../target/release/rzem-ai-inference --client --server-url http://localhost:8080
```

The client desktop UI should open, connected to the server running in Terminal 1.

### Option 2: Two Machine Testing

#### Machine A (GPU Server)
```bash
# Find your IP address
ip addr show  # Linux
ipconfig      # Windows

# Start server (replace 192.168.1.100 with your IP)
./rzem-ai-inference --server --port 8080

# Or headless mode (no desktop UI)
./rzem-ai-inference --headless --server --port 8080
```

#### Machine B (Client)
```bash
# Connect to server (replace with actual server IP)
./rzem-ai-inference --client --server-url http://192.168.1.100:8080
```

---

## Test Cases

### 1. Connection Verification

**Goal:** Verify client can connect to server

**Steps:**
1. Start server
2. Start client
3. Check client UI for connection status indicator

**Expected Results:**
- ✅ Connection status shows "Client Mode" with green indicator
- ✅ Server URL displayed in status bar
- ✅ Console logs show "WebSocket initialized for client mode"
- ✅ No connection errors in console

**Verify Backend Connection:**
```bash
# From server machine, check WebSocket connections
curl http://localhost:8080/api/v1/health

# Should return:
{
  "status": "ok",
  "version": "0.1.0"
}
```

---

### 2. Job Submission (Client → Server)

**Goal:** Verify client can submit generation jobs to server

**Steps:**
1. In client UI, navigate to Generation view
2. Enter prompt: "a beautiful sunset over mountains"
3. Select model: "Schnell"
4. Set steps: 4
5. Click "Generate"

**Expected Results:**
- ✅ Job appears in queue immediately
- ✅ Job shows "Pending" status
- ✅ No errors in console
- ✅ Job ID is displayed

**Verify on Server:**
```bash
# Check queue via REST API
curl http://localhost:8080/api/v1/queue | jq

# Should show the submitted job
```

---

### 3. Real-time Progress Updates

**Goal:** Verify client receives WebSocket updates during generation

**Steps:**
1. Submit a job from client (see Test 2)
2. Watch the job in the queue view
3. Observe status changes

**Expected Results:**
- ✅ Status changes: Pending → Running → Completed
- ✅ Progress bar updates in real-time (0% → 100%)
- ✅ Stage messages displayed ("Loading models", "Encoding", "Denoising", etc.)
- ✅ Current step / total steps shown during denoising
- ✅ Completion notification appears
- ✅ Result image displayed in gallery

**Console Verification:**
Open browser devtools (if using Tauri's webview) and look for:
```
WebSocket connected
Subscribed to job: <job-id>
JobProgress: { progress: 0.25, stage: "denoising", ... }
JobComplete: { job_id: "...", result_url: "/api/v1/files/..." }
```

---

### 4. Image Display and Download

**Goal:** Verify images are properly displayed and accessible

**Steps:**
1. After job completes (Test 3), navigate to Gallery
2. Find the generated image
3. Click on image to view full size
4. Right-click and "Save As"

**Expected Results:**
- ✅ Thumbnail loads correctly
- ✅ Full image loads when clicked
- ✅ Image URL points to server: `http://server:8080/api/v1/files/flux_xxx.png`
- ✅ Image can be saved locally
- ✅ No CORS errors

**Manual URL Test:**
```bash
# Get job result
curl http://localhost:8080/api/v1/queue/<job-id> | jq .result_path

# Download image directly
curl http://localhost:8080/api/v1/files/flux_xxx.png -o test.png

# Verify file size
ls -lh test.png
```

---

### 5. Job Cancellation

**Goal:** Verify client can cancel pending/running jobs

**Steps:**
1. Submit multiple jobs to create a queue
2. While a job is running, click "Cancel" on a pending job
3. Try to cancel the currently running job

**Expected Results:**
- ✅ Pending job cancels immediately
- ✅ Running job cannot be cancelled (returns false)
- ✅ Cancelled job disappears from queue
- ✅ Next pending job starts processing

**API Verification:**
```bash
# Submit job
JOB_ID=$(curl -X POST http://localhost:8080/api/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt":"test","steps":4,"model":"schnell","width":1024,"height":1024,"cfg_scale":7.5,"seed":-1}' \
  | jq -r .job_id)

# Cancel job
curl -X DELETE http://localhost:8080/api/v1/queue/$JOB_ID

# Verify cancelled
curl http://localhost:8080/api/v1/queue/$JOB_ID
# Should return 404
```

---

### 6. Multiple Clients

**Goal:** Verify multiple clients can connect simultaneously

**Steps:**
1. Keep server running
2. Start client 1
3. Start client 2 (on same or different machine)
4. Submit job from client 1
5. Observe both clients

**Expected Results:**
- ✅ Both clients show connection indicator
- ✅ Job submitted from client 1 appears in both queues
- ✅ Both clients receive progress updates
- ✅ Both clients can view generated images

---

### 7. Connection Loss and Recovery

**Goal:** Verify client handles connection interruptions

**Steps:**
1. Start server and client
2. Submit a job
3. Kill the server (Ctrl+C)
4. Observe client UI
5. Restart server
6. Observe client reconnection

**Expected Results:**
- ✅ Client shows disconnected state (red indicator)
- ✅ Console shows "WebSocket closed"
- ✅ Console shows reconnection attempts
- ✅ After server restart, client reconnects automatically
- ✅ Connection indicator turns green
- ✅ Queue state refreshes

**Note:** Current MVP has basic reconnection. Full implementation would resubscribe to all active jobs.

---

### 8. Local Mode Compatibility

**Goal:** Verify local mode still works unchanged

**Steps:**
1. Start app without any flags: `./rzem-ai-inference`
2. Submit a generation job
3. Verify all features work

**Expected Results:**
- ✅ No connection status indicator shown (local mode)
- ✅ Generation works as before
- ✅ All UI features functional
- ✅ No WebSocket connection attempts
- ✅ Images displayed via Tauri file serving

---

### 9. Server Mode with Desktop UI

**Goal:** Verify server mode works with local desktop UI

**Steps:**
1. Start in server mode: `./rzem-ai-inference --server --port 8080`
2. Use desktop UI to generate images
3. Also submit via REST API from another terminal
4. Observe both UI and API jobs

**Expected Results:**
- ✅ Connection status shows "Server Mode" with blue indicator
- ✅ Desktop UI works normally
- ✅ REST API jobs appear in desktop queue
- ✅ Desktop jobs appear in REST API
- ✅ All jobs process correctly

**API Test:**
```bash
# Submit via API while desktop is running
curl -X POST http://localhost:8080/api/v1/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "test from API",
    "steps": 4,
    "model": "schnell",
    "width": 1024,
    "height": 1024,
    "cfg_scale": 7.5,
    "seed": -1
  }'
```

---

## Performance Tests

### 10. Network Latency

**Goal:** Measure performance over network

**Steps:**
1. Use server on LAN (not localhost)
2. Submit jobs from client
3. Monitor response times

**Benchmarks:**
- Health check: < 50ms
- Job submission: < 100ms
- Queue refresh: < 200ms
- WebSocket ping/pong: < 50ms
- Image download (5MB): < 1s on LAN

**Measurement:**
```bash
# Measure API latency
time curl http://192.168.1.100:8080/api/v1/health

# Measure WebSocket connection
# (use browser devtools or wscat)
wscat -c ws://192.168.1.100:8080/api/v1/ws
# Send: {"type":"Ping"}
# Measure time to Pong response
```

---

### 11. Concurrent Jobs

**Goal:** Verify multiple concurrent jobs

**Steps:**
1. Submit 5 jobs rapidly from client
2. Observe queue processing
3. Verify all complete successfully

**Expected Results:**
- ✅ All jobs enter queue
- ✅ Jobs process sequentially (MVP limitation)
- ✅ Progress updates for each job
- ✅ All complete without errors
- ✅ All images downloadable

---

## Error Scenarios

### 12. Invalid Server URL

**Goal:** Verify graceful handling of connection errors

**Steps:**
```bash
# Try to connect to non-existent server
./rzem-ai-inference --client --server-url http://invalid:9999
```

**Expected Results:**
- ✅ Clear error message about connection failure
- ✅ App doesn't crash
- ✅ Retry logic attempts reconnection
- ✅ User notified of problem

---

### 13. Network Interruption During Generation

**Goal:** Verify job state after network failure

**Steps:**
1. Submit long-running job (20+ steps with Dev model)
2. Disconnect network (unplug ethernet / disable WiFi)
3. Wait for job to continue on server
4. Reconnect network

**Expected Results:**
- ✅ Client shows disconnected state
- ✅ Server continues processing job
- ✅ After reconnection, client receives updated state
- ✅ Image becomes available

---

## Debugging Tips

### Check WebSocket Connection
```bash
# Install wscat if needed
npm install -g wscat

# Connect to server WebSocket
wscat -c ws://localhost:8080/api/v1/ws

# Send ping
{"type":"Ping"}

# Should receive
{"type":"Pong"}

# Subscribe to job
{"type":"Subscribe","job_id":"<job-id>"}

# Should receive
{"type":"Subscribed","job_id":"<job-id>"}
```

### Monitor Server Logs
```bash
# Run server with debug logging
RUST_LOG=debug ./rzem-ai-inference --server --port 8080
```

### Check Network Traffic
```bash
# Use tcpdump to monitor HTTP/WebSocket
sudo tcpdump -i any -A 'port 8080'

# Or use Wireshark with filter: tcp.port == 8080
```

### Browser DevTools
- Open Tauri webview devtools: Right-click → Inspect Element
- Check Console tab for JavaScript errors
- Check Network tab for failed requests
- Check Application → Frames → WebSocket for WS messages

---

## Success Criteria

Phase 3 is considered complete when:

- ✅ Client can connect to remote server
- ✅ Jobs submitted from client appear in server queue
- ✅ Client receives real-time WebSocket updates
- ✅ Generated images display correctly from server URLs
- ✅ Multiple clients can connect simultaneously
- ✅ Connection status indicator shows accurate state
- ✅ Local mode remains unchanged
- ✅ Server mode works with desktop UI
- ✅ Basic reconnection logic works

---

## Known Limitations (MVP)

1. **No Authentication**: Anyone on network can access API
2. **No Image Caching**: Client downloads images every view
3. **Gallery Operations**: Limited to local gallery in MVP
4. **Single Queue**: One job at a time (sequential processing)
5. **No Persistence**: Subscriptions lost on restart
6. **Basic Reconnection**: Only attempts reconnect, doesn't restore state

These will be addressed in post-MVP phases.

---

## Troubleshooting

### "Failed to connect to server"
- Verify server is running: `curl http://server:8080/api/v1/health`
- Check firewall allows port 8080
- Verify server URL in client command is correct

### "WebSocket connection failed"
- Check server supports WebSocket upgrade
- Verify no proxy blocking WebSocket
- Check server logs for errors

### "Images not loading"
- Verify file serving works: `curl http://server:8080/api/v1/files/<filename>`
- Check CORS headers allow your client origin
- Verify images exist in server's output directory

### "Job updates not appearing"
- Check WebSocket connection status
- Verify subscription message sent
- Check server logs for broadcast events
- Monitor browser Network tab for WS messages

---

**Last Updated:** 2026-01-23
**Phase:** 3 - Client Mode Implementation
**Status:** Ready for Testing
