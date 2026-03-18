/**
 * Manages the Python inference engine as a subprocess (sidecar).
 * Replaces src/bun/sidecar.ts — uses Node.js child_process instead of Bun.spawn.
 *
 * The engine exposes a FastAPI server with REST + WebSocket endpoints.
 * This manager handles spawning, health checks, restart, and shutdown.
 */

import { spawn, type ChildProcess } from "child_process";
import WebSocket from "ws";
import axios, { type AxiosInstance } from "axios";

export interface SidecarConfig {
	outputDir: string;
	port: number;
	host?: string;
	engineDir?: string;
	device?: string;
	vramLimitGb?: number;
	previewInterval?: number;
}

export interface SidecarStatus {
	running: boolean;
	port: number;
	pid: number | null;
	url: string;
}

export class SidecarManager {
	private config: Required<Pick<SidecarConfig, "outputDir" | "port" | "host">> &
		SidecarConfig;
	private process: ChildProcess | null = null;
	private ws: WebSocket | null = null;
	private healthCheckInterval: ReturnType<typeof setInterval> | null = null;
	private _ready = false;
	private http: AxiosInstance;

	constructor(config: SidecarConfig) {
		this.config = {
			host: "127.0.0.1",
			...config,
		};
		this.http = axios.create({
			baseURL: this.baseUrl,
		});
	}

	get baseUrl(): string {
		return `http://${this.config.host}:${this.config.port}`;
	}

	get wsUrl(): string {
		return `ws://${this.config.host}:${this.config.port}/ws`;
	}

	get ready(): boolean {
		return this._ready;
	}

	get status(): SidecarStatus {
		return {
			running: this.process !== null && !this.process.killed,
			port: this.config.port,
			pid: this.process?.pid ?? null,
			url: this.baseUrl,
		};
	}

	/**
	 * Start the Python inference engine sidecar.
	 * Waits for the health endpoint to respond before resolving.
	 */
	async start(): Promise<void> {
		if (this.process && !this.process.killed) {
			console.log("Sidecar already running");
			return;
		}

		const args = [
			"run",
			"python",
			"-m",
			"rzem_ai_inference_engine",
			"serve",
			"--host",
			this.config.host,
			"--port",
			String(this.config.port),
			"--output-dir",
			this.config.outputDir,
			"--no-announce",
		];

		if (this.config.device) {
			args.push("--device", this.config.device);
		}
		if (this.config.vramLimitGb) {
			args.push("--vram-limit", String(this.config.vramLimitGb));
		}
		if (this.config.previewInterval) {
			args.push("--preview-interval", String(this.config.previewInterval));
		}

		console.log(
			`Starting sidecar: uv ${args.join(" ")}${this.config.engineDir ? ` (cwd: ${this.config.engineDir})` : ""}`,
		);

		this.process = spawn("uv", args, {
			cwd: this.config.engineDir,
			stdio: ["ignore", "inherit", "inherit"],
		});

		this.process.on("exit", (code, signal) => {
			console.log(`Sidecar exited (code=${code}, signal=${signal})`);
			this._ready = false;
			this.process = null;
		});

		this.process.on("error", (err) => {
			console.error("Sidecar spawn error:", err);
			this._ready = false;
			this.process = null;
		});

		// Wait for the server to become ready
		await this.waitForHealth(30_000);
		this._ready = true;

		// Connect WebSocket for real-time events
		this.connectWebSocket();

		// Start periodic health checks
		this.healthCheckInterval = setInterval(() => this.checkHealth(), 5_000);

		console.log(
			`Sidecar ready at ${this.baseUrl} (PID ${this.process?.pid})`,
		);
	}

	/**
	 * Stop the sidecar process.
	 */
	stop(): void {
		if (this.healthCheckInterval) {
			clearInterval(this.healthCheckInterval);
			this.healthCheckInterval = null;
		}

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		if (this.process && !this.process.killed) {
			console.log("Stopping sidecar...");
			this.process.kill("SIGTERM");
			this.process = null;
		}

		this._ready = false;
	}

	/**
	 * Restart the sidecar process.
	 */
	async restart(): Promise<void> {
		this.stop();
		await new Promise((resolve) => setTimeout(resolve, 1000));
		await this.start();
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
				`[sidecar] ${options?.method ?? "GET"} ${path} → ${status}`,
				body,
			);
			throw new Error(
				`Sidecar ${options?.method ?? "GET"} ${path} failed: ${status}`,
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
			`Sidecar health check timed out after ${timeoutMs}ms`,
		);
	}

	private async checkHealth(): Promise<void> {
		try {
			await this.http.get("/health", { timeout: 3000 });
			this._ready = true;
		} catch {
			console.warn("Sidecar health check failed — process may have died");
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
				console.error("Failed to parse sidecar WS message:", err);
			}
		});

		ws.on("close", () => {
			console.log("Sidecar WebSocket closed");
			this.ws = null;
			// Reconnect after a delay if process is still running
			if (this.process && !this.process.killed) {
				setTimeout(() => this.connectWebSocket(), 2000);
			}
		});

		ws.on("error", (err) => {
			console.error("Sidecar WebSocket error:", err);
		});

		this.ws = ws;
	}
}
