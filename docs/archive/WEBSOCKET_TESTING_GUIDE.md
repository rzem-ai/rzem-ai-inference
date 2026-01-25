# WebSocket Testing Guide

Complete guide for testing real-time WebSocket functionality in RZEM AI Inference.

## Prerequisites

```bash
# Install websocat (WebSocket CLI client)
# Ubuntu/Debian
sudo apt install websocat

# macOS
brew install websocat

# Or download from: https://github.com/vi/websocat

# For Node.js testing
npm install -g wscat
# or
npm install ws
```

## Starting the Server

```bash
# Start server mode
./src-tauri/target/release/rzem-ai-inference --server --port 8080

# You should see:
# Starting in server mode on port 8080...
# API will be available at: http://localhost:8080/api/v1
# WebSocket at: ws://localhost:8080/api/v1/ws
```

## Quick Test

### 1. Connect with websocat

```bash
# Open terminal and connect
websocat ws://localhost:8080/api/v1/ws

# You should immediately receive:
{"type":"Connected","connection_id":"abc-123-..."}
```

### 2. Send a Ping

```json
{"type":"Ping"}
```

**Expected Response:**
```json
{"type":"Pong"}
```

### 3. Subscribe to a Job

First, submit a job via REST API (in another terminal):

```bash
JOB_ID=$(curl -s -X POST http://localhost:8080/api/v1/generate \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "a serene mountain landscape",
    "steps": 4,
    "cfg_scale": 7.5,
    "width": 512,
    "height": 512,
    "seed": 42,
    "model": "schnell"
  }' | jq -r '.job_id')

echo "Job ID: $JOB_ID"
```

Then subscribe in websocat:

```json
{"type":"Subscribe","job_id":"YOUR_JOB_ID_HERE"}
```

**Expected Responses:**
```json
{"type":"Subscribed","job_id":"..."}
{"type":"JobProgress","job_id":"...","status":"pending","progress":0.0,...}
{"type":"JobProgress","job_id":"...","status":"running","progress":0.35,...}
{"type":"JobProgress","job_id":"...","status":"running","progress":0.70,...}
{"type":"JobComplete","job_id":"...","result_url":"/api/v1/files/flux_123_42.png"}
```

## Testing with JavaScript/Node.js

### Browser Console

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');

ws.onopen = () => {
  console.log('✓ Connected');
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('📨', msg);

  // Auto-subscribe to job after connection
  if (msg.type === 'Connected') {
    console.log('Connection ID:', msg.connection_id);
  }
};

ws.onerror = (error) => {
  console.error('❌ Error:', error);
};

ws.onclose = () => {
  console.log('🔌 Disconnected');
};

// Helper to send messages
function send(type, data = {}) {
  ws.send(JSON.stringify({ type, ...data }));
}

// Usage examples:
send('Ping');
send('Subscribe', { job_id: 'your-job-id' });
send('Unsubscribe', { job_id: 'your-job-id' });
```

### Node.js Script

Create `test-ws.js`:

```javascript
const WebSocket = require('ws');
const fetch = require('node-fetch');

const API_BASE = 'http://localhost:8080/api/v1';
const WS_URL = 'ws://localhost:8080/api/v1/ws';

async function main() {
  console.log('🚀 Starting WebSocket test...\n');

  // Connect to WebSocket
  const ws = new WebSocket(WS_URL);

  ws.on('open', async () => {
    console.log('✓ WebSocket connected\n');

    // Submit a generation job
    console.log('📝 Submitting generation job...');
    const response = await fetch(`${API_BASE}/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        prompt: 'a beautiful sunset over the ocean',
        steps: 4,
        cfg_scale: 7.5,
        width: 512,
        height: 512,
        seed: 42,
        model: 'schnell'
      })
    });

    const data = await response.json();
    const jobId = data.job_id;
    console.log(`✓ Job submitted: ${jobId}\n`);

    // Subscribe to job updates
    console.log('🔔 Subscribing to job updates...');
    ws.send(JSON.stringify({
      type: 'Subscribe',
      job_id: jobId
    }));
  });

  ws.on('message', (data) => {
    const msg = JSON.parse(data);

    switch (msg.type) {
      case 'Connected':
        console.log(`✓ Connected with ID: ${msg.connection_id}\n`);
        break;

      case 'Subscribed':
        console.log(`✓ Subscribed to job: ${msg.job_id}\n`);
        break;

      case 'JobProgress':
        const percent = (msg.progress * 100).toFixed(0);
        console.log(`📊 Progress: ${percent}% (${msg.status})`);
        break;

      case 'JobComplete':
        console.log(`\n✅ Job complete!`);
        console.log(`📁 Result URL: ${msg.result_url}`);
        console.log(`\nDownload with:`);
        console.log(`curl http://localhost:8080${msg.result_url} -o result.png`);
        ws.close();
        break;

      case 'JobFailed':
        console.error(`\n❌ Job failed: ${msg.error}`);
        ws.close();
        break;

      case 'Pong':
        console.log('🏓 Pong received');
        break;

      case 'Error':
        console.error(`❌ Error: ${msg.message}`);
        break;

      default:
        console.log('📨 Received:', msg);
    }
  });

  ws.on('error', (error) => {
    console.error('❌ WebSocket error:', error.message);
  });

  ws.on('close', () => {
    console.log('\n🔌 WebSocket closed');
    process.exit(0);
  });
}

main().catch(console.error);
```

Run with: `node test-ws.js`

### Python Script

Create `test_ws.py`:

```python
#!/usr/bin/env python3
import asyncio
import json
import websockets
import requests

API_BASE = 'http://localhost:8080/api/v1'
WS_URL = 'ws://localhost:8080/api/v1/ws'

async def test_websocket():
    print('🚀 Starting WebSocket test...\n')

    async with websockets.connect(WS_URL) as ws:
        print('✓ WebSocket connected\n')

        # Submit a generation job
        print('📝 Submitting generation job...')
        response = requests.post(f'{API_BASE}/generate', json={
            'prompt': 'a majestic lion in the savanna',
            'steps': 4,
            'cfg_scale': 7.5,
            'width': 512,
            'height': 512,
            'seed': 42,
            'model': 'schnell'
        })

        data = response.json()
        job_id = data['job_id']
        print(f'✓ Job submitted: {job_id}\n')

        # Subscribe to job updates
        print('🔔 Subscribing to job updates...')
        await ws.send(json.dumps({
            'type': 'Subscribe',
            'job_id': job_id
        }))

        # Receive messages
        async for message in ws:
            msg = json.loads(message)
            msg_type = msg['type']

            if msg_type == 'Connected':
                print(f"✓ Connected with ID: {msg['connection_id']}\n")

            elif msg_type == 'Subscribed':
                print(f"✓ Subscribed to job: {msg['job_id']}\n")

            elif msg_type == 'JobProgress':
                percent = int(msg['progress'] * 100)
                print(f"📊 Progress: {percent}% ({msg['status']})")

            elif msg_type == 'JobComplete':
                print(f"\n✅ Job complete!")
                print(f"📁 Result URL: {msg['result_url']}")
                print(f"\nDownload with:")
                print(f"curl http://localhost:8080{msg['result_url']} -o result.png")
                break

            elif msg_type == 'JobFailed':
                print(f"\n❌ Job failed: {msg['error']}")
                break

            elif msg_type == 'Error':
                print(f"❌ Error: {msg['message']}")

        print('\n🔌 WebSocket closed')

if __name__ == '__main__':
    asyncio.run(test_websocket())
```

Run with: `python3 test_ws.py`

## Testing Multiple Clients

### Terminal 1: Server
```bash
./rzem-ai-inference --server --port 8080
```

### Terminal 2: Submit Job
```bash
JOB_ID=$(curl -s -X POST http://localhost:8080/api/v1/generate \
  -H 'Content-Type: application/json' \
  -d '{"prompt": "a cat", "steps": 20, "cfg_scale": 7.5, "width": 1024, "height": 1024, "seed": -1, "model": "schnell"}' \
  | jq -r '.job_id')
echo "Job ID: $JOB_ID"
```

### Terminal 3: Client 1
```bash
websocat ws://localhost:8080/api/v1/ws
# Then send:
{"type":"Subscribe","job_id":"PASTE_JOB_ID_HERE"}
```

### Terminal 4: Client 2
```bash
websocat ws://localhost:8080/api/v1/ws
# Then send:
{"type":"Subscribe","job_id":"PASTE_JOB_ID_HERE"}
```

### Terminal 5: Client 3
```bash
websocat ws://localhost:8080/api/v1/ws
# Then send:
{"type":"Subscribe","job_id":"PASTE_JOB_ID_HERE"}
```

**Expected:** All three clients receive identical progress updates.

## Advanced Testing

### Test Connection Cleanup

```bash
# Connect
websocat ws://localhost:8080/api/v1/ws

# Subscribe to a job
{"type":"Subscribe","job_id":"some-job-id"}

# Press Ctrl+C to disconnect

# In server logs, you should see:
# INFO: Connection cleaned up
```

### Test Heartbeat

```bash
# Connect and wait 30+ seconds
websocat ws://localhost:8080/api/v1/ws

# You'll receive automatic ping frames (not visible in websocat)
# Connection stays alive
```

### Test Unsubscribe

```bash
websocat ws://localhost:8080/api/v1/ws

# Subscribe
{"type":"Subscribe","job_id":"job-123"}
# Response: {"type":"Subscribed","job_id":"job-123"}

# Unsubscribe
{"type":"Unsubscribe","job_id":"job-123"}
# Response: {"type":"Unsubscribed","job_id":"job-123"}

# You'll no longer receive updates for this job
```

### Load Testing

```bash
# Use wscat for load testing
npm install -g wscat

# Connect multiple clients
for i in {1..10}; do
  wscat -c ws://localhost:8080/api/v1/ws &
done

# Check server logs for connection count
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug ./rzem-ai-inference --server --port 8080
```

### Common Issues

**Connection Refused:**
```bash
# Check if server is running
ps aux | grep rzem-ai-inference

# Check if port is in use
lsof -i :8080
```

**No Messages Received:**
- Verify you subscribed to a valid job_id
- Check job status via REST API: `curl http://localhost:8080/api/v1/queue/{job_id}`
- Ensure job is not already completed

**Connection Closes Immediately:**
- Check server logs for errors
- Verify WebSocket endpoint: `ws://localhost:8080/api/v1/ws` (not http://)

## Monitoring

### Watch Active Connections

```bash
# In server logs (with RUST_LOG=info)
# You'll see:
# INFO: New WebSocket connection established
# INFO: Connection registered connection_id=abc123
# INFO: Subscribed to job connection_id=abc123 job_id=uuid
# INFO: Connection cleaned up connection_id=abc123
```

### Count Active Connections

No built-in endpoint yet. Future enhancement:
```
GET /api/v1/system/ws-stats
{
  "active_connections": 5,
  "active_subscriptions": 12
}
```

## Performance Testing

### Latency Test

```bash
# Measure round-trip time
time websocat -1 ws://localhost:8080/api/v1/ws <<< '{"type":"Ping"}'

# Typical: < 10ms on localhost
```

### Throughput Test

```javascript
// throughput-test.js
const WebSocket = require('ws');
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');

let start, pingsReceived = 0;

ws.on('open', () => {
  start = Date.now();
  for (let i = 0; i < 1000; i++) {
    ws.send(JSON.stringify({ type: 'Ping' }));
  }
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);
  if (msg.type === 'Pong') {
    pingsReceived++;
    if (pingsReceived === 1000) {
      const elapsed = Date.now() - start;
      console.log(`1000 pings in ${elapsed}ms`);
      console.log(`Rate: ${(1000 / elapsed * 1000).toFixed(0)} msgs/sec`);
      ws.close();
    }
  }
});
```

## Next Steps

After verifying WebSocket functionality:
1. Test Phase 3: Client mode implementation
2. Integrate WebSocket into Vue frontend
3. Build status indicator component
4. Implement reconnection logic

For REST API testing, see `API_TESTING_GUIDE.md`

---

**Status**: Phase 2 Complete
**Date**: 2026-01-23
