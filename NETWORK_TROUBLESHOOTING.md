# Network Connectivity Troubleshooting

## Common 404 Errors and Solutions

### Issue: Getting 404 when connecting from another machine

---

## Quick Diagnosis

### Step 1: Verify Server is Running

On the **server machine**, check if the server is actually listening:

```bash
# Check if port 8080 is listening on all interfaces
ss -tlnp | grep :8080

# Expected output:
# LISTEN 0      128          0.0.0.0:8080       0.0.0.0:*
#                           ^^^^^^^
#                           This should be 0.0.0.0, NOT 127.0.0.1
```

If you see `127.0.0.1:8080` instead of `0.0.0.0:8080`, the server is only listening on localhost.

---

### Step 2: Test from Server Machine (Localhost)

On the **server machine**, test if the API works locally:

```bash
# Test health endpoint
curl http://localhost:8080/api/v1/health

# Expected:
# {"status":"ok","version":"0.1.0"}

# Test root page
curl http://localhost:8080/

# Should return HTML page
```

**If this fails**, the server isn't running correctly. Check the logs.

**If this works**, proceed to Step 3.

---

### Step 3: Get Server's IP Address

On the **server machine**, find its LAN IP:

```bash
# Linux
hostname -I
# or
ip addr show | grep "inet " | grep -v "127.0.0.1"

# macOS
ifconfig | grep "inet " | grep -v "127.0.0.1"

# Windows
ipconfig
```

Example output: `192.168.1.100`

---

### Step 4: Test from Server Machine (Using LAN IP)

Still on the **server machine**, test using the LAN IP:

```bash
# Replace with your actual IP
curl http://192.168.1.100:8080/api/v1/health
```

**If this fails but localhost works**, you have a firewall issue (see Step 5).

**If this works**, proceed to Step 6.

---

### Step 5: Check Firewall

The firewall might be blocking port 8080.

#### Linux (UFW)
```bash
# Check status
sudo ufw status

# Allow port 8080
sudo ufw allow 8080/tcp

# Or allow from specific subnet only
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

#### Linux (firewalld)
```bash
# Check status
sudo firewall-cmd --state

# Allow port 8080
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload
```

#### macOS
```bash
# Disable firewall temporarily for testing
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off

# Re-enable after testing
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on
```

#### Windows
```powershell
# Add firewall rule
New-NetFirewallRule -DisplayName "RZEM Server" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```

---

### Step 6: Test from Client Machine

On the **client machine** (different computer):

```bash
# Test connection to server
ping 192.168.1.100

# Test if port is reachable
nc -zv 192.168.1.100 8080
# or
telnet 192.168.1.100 8080

# Test API endpoint
curl http://192.168.1.100:8080/api/v1/health
```

---

## Common Mistakes

### 1. Wrong URL Path

❌ **Wrong:**
```bash
curl http://192.168.1.100:8080/health          # Missing /api/v1/
curl http://192.168.1.100:8080/api/health      # Missing /v1/
```

✅ **Correct:**
```bash
curl http://192.168.1.100:8080/api/v1/health
```

### 2. Using Wrong IP

❌ **Wrong:**
```bash
# Using localhost from client machine
./rzem-ai-inference --client --server-url http://localhost:8080

# Using server's public IP instead of LAN IP
./rzem-ai-inference --client --server-url http://203.0.113.1:8080
```

✅ **Correct:**
```bash
# Using server's LAN IP
./rzem-ai-inference --client --server-url http://192.168.1.100:8080
```

### 3. Firewall Blocking

Most common issue! The server binds to `0.0.0.0:8080` but the firewall blocks external connections.

**Test:** Can you ping the server from the client? If yes but curl fails, it's likely the firewall.

---

## Debugging Checklist

From the **server machine**:

- [ ] Server process is running
- [ ] `ss -tlnp | grep :8080` shows `0.0.0.0:8080`
- [ ] `curl http://localhost:8080/api/v1/health` works
- [ ] `curl http://<LAN_IP>:8080/api/v1/health` works
- [ ] Firewall allows port 8080
- [ ] Server logs show "Server listening on 0.0.0.0:8080"

From the **client machine**:

- [ ] Can ping server IP
- [ ] `nc -zv <SERVER_IP> 8080` succeeds
- [ ] `curl http://<SERVER_IP>:8080/api/v1/health` works
- [ ] Using correct server URL in client command

---

## Verbose Testing

### Server Side

Start the server with debug logging:

```bash
RUST_LOG=debug ./rzem-ai-inference --server --port 8080
```

Look for:
```
Server listening on 0.0.0.0:8080
```

### Client Side

Test with verbose curl:

```bash
curl -v http://192.168.1.100:8080/api/v1/health
```

Look for:
```
* Connected to 192.168.1.100 (192.168.1.100) port 8080
> GET /api/v1/health HTTP/1.1
< HTTP/1.1 200 OK
```

If you see:
```
< HTTP/1.1 404 Not Found
```

Check the URL path - you're hitting a non-existent route.

---

## Network Capture

If all else fails, capture the actual network traffic:

### Server Side
```bash
sudo tcpdump -i any -n port 8080 -A
```

### Make request from client

### Look for

- Do packets arrive at the server?
- What's the HTTP request path?
- What's the response status code?

---

## Server Verification Script

Save this as `test_server.sh` and run it on the **server machine**:

```bash
#!/bin/bash
echo "=== RZEM Server Diagnostics ==="
echo ""

echo "1. Checking if port 8080 is listening..."
ss -tlnp 2>/dev/null | grep :8080 || echo "❌ Port 8080 not listening!"
echo ""

echo "2. Testing localhost..."
curl -s http://localhost:8080/api/v1/health && echo "✅ Localhost works" || echo "❌ Localhost failed"
echo ""

echo "3. Server IP addresses:"
hostname -I 2>/dev/null || ip addr show | grep "inet " | grep -v "127.0.0.1"
echo ""

echo "4. Testing LAN IP..."
SERVER_IP=$(hostname -I | awk '{print $1}')
curl -s http://$SERVER_IP:8080/api/v1/health && echo "✅ LAN IP works" || echo "❌ LAN IP failed (check firewall)"
echo ""

echo "5. Firewall status:"
sudo ufw status 2>/dev/null || sudo firewall-cmd --list-all 2>/dev/null || echo "Cannot detect firewall"
echo ""

echo "=== Diagnostics Complete ==="
```

Run: `bash test_server.sh`

---

## Client Verification Script

Save this as `test_client.sh` and run it on the **client machine**:

```bash
#!/bin/bash
SERVER_IP=$1

if [ -z "$SERVER_IP" ]; then
    echo "Usage: $0 <server-ip>"
    echo "Example: $0 192.168.1.100"
    exit 1
fi

echo "=== Testing connection to $SERVER_IP ==="
echo ""

echo "1. Pinging server..."
ping -c 3 $SERVER_IP && echo "✅ Server is reachable" || echo "❌ Cannot reach server"
echo ""

echo "2. Testing port 8080..."
nc -zv $SERVER_IP 8080 2>&1 && echo "✅ Port 8080 is open" || echo "❌ Port 8080 blocked"
echo ""

echo "3. Testing API health endpoint..."
curl -v http://$SERVER_IP:8080/api/v1/health
echo ""

echo "=== Test Complete ==="
```

Run: `bash test_client.sh 192.168.1.100`

---

## Still Having Issues?

If none of the above works, provide:

1. **Server logs** (start with `RUST_LOG=debug`)
2. **Output of:** `ss -tlnp | grep :8080` from server
3. **Output of:** `curl -v http://<SERVER_IP>:8080/api/v1/health` from client
4. **Firewall status** from server
5. **Network topology** (same subnet? VPN? Docker?)

---

## Quick Fix Summary

**Most common fix:**
```bash
# On server, allow port in firewall
sudo ufw allow 8080/tcp

# Restart server
./rzem-ai-inference --server --port 8080

# On client, use correct URL with /api/v1/ prefix
curl http://192.168.1.100:8080/api/v1/health
```
