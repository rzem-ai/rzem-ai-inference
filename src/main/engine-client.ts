/**
 * Network client for the Python inference engine.
 * Connects to an already-running engine over HTTP REST + WebSocket.
 * No process management — the engine is an external service.
 */

import WebSocket from "ws";
import axios, { type AxiosInstance } from "axios";

export interface EngineClientConfig {
	host: string;
	port: number;
}

export interface EngineClientStatus {
	connected: boolean;
	host: string;
	port: number;
	url: string;
}

export class EngineClient {
	private _host: string;
	private _port: number;
	private ws: WebSocket | null = null;
	private healthCheckInterval: ReturnType<typeof setInterval> | null = null;
	private _ready = false;
	private http: AxiosInstance;

	constructor(config: EngineClientConfig) {
		this._host = config.host;
		this._port = config.port;
		this.http = axios.create({
			baseURL: this.baseUrl,
		});
	}

	get host(): string {
		return this._host;
	}

	get port(): number {
		return this._port;
	}

	get baseUrl(): string {
		return `http://${this._host}:${this._port}`;
	}

	get wsUrl(): string {
		return `ws://${this._host}:${this._port}/ws`;
	}

	get ready(): boolean {
		return this._ready;
	}

	get status(): EngineClientStatus {
		return {
			connected: this._ready,
			host: this._host,
			port: this._port,
			url: this.baseUrl,
		};
	}

	/**
	 * Connect to the inference engine at the current (or new) host:port.
	 * Waits for the health endpoint to respond before resolving.
	 */
	async connect(host?: string, port?: number): Promise<void> {
		// Clean up any existing connection first
		this.disconnect();

		// Update target if new host/port provided
		if (host !== undefined) this._host = host;
		if (port !== undefined) this._port = port;

		// Recreate axios instance with updated baseURL
		this.http = axios.create({
			baseURL: this.baseUrl,
		});

		console.log(`Connecting to engine at ${this.baseUrl}...`);

		// Wait for the server to respond
		await this.waitForHealth(5_000);
		this._ready = true;

		// Connect WebSocket for real-time events
		this.connectWebSocket();

		// Start periodic health checks
		this.healthCheckInterval = setInterval(() => this.checkHealth(), 5_000);

		console.log(`Engine connected at ${this.baseUrl}`);
	}

	/**
	 * Disconnect from the engine (close WebSocket + stop health checks).
	 */
	disconnect(): void {
		if (this.healthCheckInterval) {
			clearInterval(this.healthCheckInterval);
			this.healthCheckInterval = null;
		}

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		this._ready = false;
	}

	/**
	 * Reconnect to the engine at the current host:port.
	 */
	async reconnect(): Promise<void> {
		this.disconnect();
		await new Promise((resolve) => setTimeout(resolve, 500));
		await this.connect();
	}

	// ── HTTP client methods for engine API ─────────────────────────

	async fetch(path: string, options?: { method?: string; body?: string; headers?: Record<string, string> }): Promise<{ ok: boolean; status: number; statusText: string; data: unknown }> {
		const res = await this.http.request({
			url: path,
			method: (options?.method as any) ?? "GET",
			data: options?.body,
			headers: options?.headers,
		});
		return { ok: true, status: res.status, statusText: res.statusText, data: res.data };
	}

	async fetchJson<T = unknown>(path: string, options?: { method?: string; body?: string; headers?: Record<string, string> }): Promise<T> {
		try {
			const res = await this.http.request<T>({
				url: path,
				method: (options?.method as any) ?? "GET",
				data: options?.body,
				headers: {
					"Content-Type": "application/json",
					...options?.headers,
				},
			});
			return res.data;
		} catch (err: any) {
			const status = err.response?.status ?? "unknown";
			const body = err.response?.data ?? "";
			console.error(
				`[engine] ${options?.method ?? "GET"} ${path} → ${status}`,
				body,
			);
			throw new Error(
				`Engine ${options?.method ?? "GET"} ${path} failed: ${status}`,
			);
		}
	}

	// ── Event listeners ────────────────────────────────────────────

	private eventListeners: Array<(event: Record<string, unknown>) => void> = [];

	/**
	 * Register a listener for engine events (received over WebSocket).
	 */
	onEvent(listener: (event: Record<string, unknown>) => void): () => void {
		this.eventListeners.push(listener);
		return () => {
			const idx = this.eventListeners.indexOf(listener);
			if (idx >= 0) this.eventListeners.splice(idx, 1);
		};
	}

	// ── Internal ───────────────────────────────────────────────────

	private async waitForHealth(timeoutMs: number): Promise<void> {
		const deadline = Date.now() + timeoutMs;
		while (Date.now() < deadline) {
			try {
				await this.http.get("/health", { timeout: 2000 });
				return;
			} catch {
				// Not ready yet
			}
			await new Promise((resolve) => setTimeout(resolve, 500));
		}
		throw new Error(
			`Engine health check timed out after ${timeoutMs}ms at ${this.baseUrl}`,
		);
	}

	private async checkHealth(): Promise<void> {
		try {
			await this.http.get("/health", { timeout: 3000 });
			this._ready = true;
		} catch {
			console.warn("Engine health check failed — server may be down");
			this._ready = false;
		}
	}

	private connectWebSocket(): void {
		if (this.ws) {
			this.ws.close();
		}

		const ws = new WebSocket(this.wsUrl);

		ws.on("message", (data) => {
			try {
				const parsed = JSON.parse(String(data));
				for (const listener of this.eventListeners) {
					listener(parsed);
				}
			} catch (err) {
				console.error("Failed to parse engine WS message:", err);
			}
		});

		ws.on("close", () => {
			console.log("Engine WebSocket closed");
			this.ws = null;
			// Reconnect after a delay if still in connected mode
			if (this._ready) {
				setTimeout(() => this.connectWebSocket(), 2000);
			}
		});

		ws.on("error", (err) => {
			console.error("Engine WebSocket error:", err);
		});

		this.ws = ws;
	}
}
