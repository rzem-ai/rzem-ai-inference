# Development Mode Guide

This guide covers running the application in different modes during development.

---

## NPM Scripts

### Local Mode (Default)
```bash
npm run tauri:dev
```

Runs the application in local mode with hot-reload.
- Desktop UI with local inference
- No network exposure
- Original functionality

### Server Mode
```bash
npm run tauri:dev:server
```

Runs the application in server mode with hot-reload.
- Desktop UI works normally
- REST API exposed on `http://localhost:8080/api/v1`
- WebSocket available at `ws://localhost:8080/api/v1/ws`
- Uses environment variables: `RZEM_SERVER_MODE=1` and `RZEM_PORT=8080`

**Test the API:**
```bash
curl http://localhost:8080/api/v1/health
```

### Client Mode
```bash
npm run tauri:dev:client
```

Runs the application in client mode (connects to a server).
- Desktop UI connects to remote server
- Default server: `http://localhost:8080`
- Uses environment variables: `RZEM_CLIENT_MODE=1` and `RZEM_SERVER_URL=http://localhost:8080`

**Prerequisites:**
Start a server first (in another terminal):
```bash
npm run tauri:dev:server
```

---

## Custom Configuration

### Different Port
```bash
cross-env RZEM_SERVER_MODE=1 RZEM_PORT=7181 npm run tauri:dev
```

### Different Server URL
```bash
cross-env RZEM_CLIENT_MODE=1 RZEM_SERVER_URL=http://192.168.1.100:8080 npm run tauri:dev
```

---

## Environment Variables

The application supports these environment variables for development:

| Variable | Description | Example |
|----------|-------------|---------|
| `RZEM_SERVER_MODE` | Enable server mode | `1` or `true` |
| `RZEM_CLIENT_MODE` | Enable client mode | `1` or `true` |
| `RZEM_SERVER_URL` | Server URL for client mode | `http://localhost:8080` |
| `RZEM_PORT` | Port for server mode | `8080` |

**Why environment variables?**

Tauri's dev mode doesn't support passing CLI arguments to the Rust binary easily. Environment variables provide a clean workaround that works on all platforms (via `cross-env`).

---

## Production Builds

For production, use CLI arguments (no environment variables needed):

```bash
# Build
npm run tauri:build

# Local mode
./target/release/rzem-ai-inference

# Server mode
./target/release/rzem-ai-inference --server --port 8080

# Client mode
./target/release/rzem-ai-inference --client --server-url http://192.168.1.100:8080
```

---

## Multi-Instance Development

### Scenario: Test client-server communication on one machine

**Terminal 1 - Server:**
```bash
npm run tauri:dev:server
```

Wait for "Server listening on 0.0.0.0:8080"

**Terminal 2 - Client:**
```bash
npm run tauri:dev:client
```

Now you have two instances running - one as server, one as client.

### Scenario: Different ports

**Terminal 1 - Server on 7181:**
```bash
cross-env RZEM_SERVER_MODE=1 RZEM_PORT=7181 npm run tauri:dev
```

**Terminal 2 - Client connecting to 7181:**
```bash
cross-env RZEM_CLIENT_MODE=1 RZEM_SERVER_URL=http://localhost:7181 npm run tauri:dev
```

---

## Troubleshooting

### "RZEM_SERVER_URL is required"

You're running in client mode without specifying a server URL.

**Fix:**
```bash
# Use the npm script (default server URL)
npm run tauri:dev:client

# Or specify custom URL
cross-env RZEM_CLIENT_MODE=1 RZEM_SERVER_URL=http://192.168.1.100:8080 npm run tauri:dev
```

### "Port already in use"

Another instance is using port 8080.

**Fix:**
```bash
# Use a different port
cross-env RZEM_SERVER_MODE=1 RZEM_PORT=7181 npm run tauri:dev
```

### Client can't connect to server

Make sure the server is actually running in server mode:

```bash
# Check server logs - should see:
# "Starting in server mode on port 8080..."
# "Server listening on 0.0.0.0:8080"

# Test API directly
curl http://localhost:8080/api/v1/health
```

---

## Platform Notes

### Windows

Environment variables are set using `cross-env` automatically:
```powershell
npm run tauri:dev:server
```

### macOS / Linux

Works the same way:
```bash
npm run tauri:dev:server
```

Alternatively, you can set variables manually:
```bash
RZEM_SERVER_MODE=1 RZEM_PORT=8080 npm run tauri:dev
```

---

## IDE Integration

### VS Code

Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Tauri Dev (Local)",
      "type": "node",
      "request": "launch",
      "cwd": "${workspaceFolder}",
      "runtimeExecutable": "npm",
      "runtimeArgs": ["run", "tauri:dev"]
    },
    {
      "name": "Tauri Dev (Server)",
      "type": "node",
      "request": "launch",
      "cwd": "${workspaceFolder}",
      "runtimeExecutable": "npm",
      "runtimeArgs": ["run", "tauri:dev:server"]
    },
    {
      "name": "Tauri Dev (Client)",
      "type": "node",
      "request": "launch",
      "cwd": "${workspaceFolder}",
      "runtimeExecutable": "npm",
      "runtimeArgs": ["run", "tauri:dev:client"]
    }
  ]
}
```

---

## Summary

| Mode | Dev Command | Production Command |
|------|-------------|-------------------|
| **Local** | `npm run tauri:dev` | `./rzem-ai-inference` |
| **Server** | `npm run tauri:dev:server` | `./rzem-ai-inference --server --port 8080` |
| **Client** | `npm run tauri:dev:client` | `./rzem-ai-inference --client --server-url http://SERVER:8080` |

For more details, see:
- `PHASE3_TESTING_GUIDE.md` - Complete testing guide
- `NETWORK_TROUBLESHOOTING.md` - Network connectivity issues
- `API_TESTING_GUIDE.md` - REST API examples
