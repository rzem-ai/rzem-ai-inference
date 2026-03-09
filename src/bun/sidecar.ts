import type { Subprocess } from "bun";

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

/**
 * Manages the Python inference engine as a subprocess (sidecar).
 *
 * The engine exposes a FastAPI server with REST + WebSocket endpoints.
 * This manager handles spawning, health checks, restart, and shutdown.
 */
export class SidecarManager {
	private config: Required<
		Pick<SidecarConfig, "outputDir" | "port" | "host">
	> &
		SidecarConfig;
	private process: Subprocess | null = null;
	private ws: WebSocket | null = null;
	private healthCheckInterval: ReturnType<typeof setInterval> | null = null;
	private _ready = false;

	constructor(config: SidecarConfig) {
		this.config = {
			host: "127.0.0.1",
			...config,
		};
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
			"uv",
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

		console.log(`Starting sidecar: ${args.join(" ")}${this.config.engineDir ? ` (cwd: ${this.config.engineDir})` : ""}`);

		this.process = Bun.spawn(args, {
			cwd: this.config.engineDir,
			stdout: "inherit",
			stderr: "inherit",
			onExit: (_proc, exitCode, signal) => {
				console.log(
					`Sidecar exited (code=${exitCode}, signal=${signal})`,
				);
				this._ready = false;
				this.process = null;
			},
		});

		// Wait for the server to become ready
		await this.waitForHealth(30_000);
		this._ready = true;

		// Connect WebSocket for real-time events
		this.connectWebSocket();

		// Start periodic health checks
		this.healthCheckInterval = setInterval(() => this.checkHealth(), 5_000);

		console.log(`Sidecar ready at ${this.baseUrl} (PID ${this.process.pid})`);
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

	async fetch(
		path: string,
		options?: RequestInit,
	): Promise<Response> {
		return fetch(`${this.baseUrl}${path}`, options);
	}

	async fetchJson<T = unknown>(
		path: string,
		options?: RequestInit,
	): Promise<T> {
		const res = await this.fetch(path, {
			...options,
			headers: {
				"Content-Type": "application/json",
				...options?.headers,
			},
		});
		if (!res.ok) {
			throw new Error(
				`Sidecar ${options?.method ?? "GET"} ${path} failed: ${res.status} ${res.statusText}`,
			);
		}
		return res.json() as Promise<T>;
	}

	// ── Event listeners ────────────────────────────────────────────

	private eventListeners: Array<(event: Record<string, unknown>) => void> =
		[];

	/**
	 * Register a listener for engine events (received over WebSocket).
	 */
	onEvent(
		listener: (event: Record<string, unknown>) => void,
	): () => void {
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
				const res = await fetch(`${this.baseUrl}/health`, {
					signal: AbortSignal.timeout(2000),
				});
				if (res.ok) return;
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
			const res = await fetch(`${this.baseUrl}/health`, {
				signal: AbortSignal.timeout(3000),
			});
			if (!res.ok) {
				console.warn(`Sidecar health check failed: ${res.status}`);
				this._ready = false;
			} else {
				this._ready = true;
			}
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

		ws.addEventListener("message", (event) => {
			try {
				const data = JSON.parse(String(event.data));
				for (const listener of this.eventListeners) {
					listener(data);
				}
			} catch (err) {
				console.error("Failed to parse sidecar WS message:", err);
			}
		});

		ws.addEventListener("close", () => {
			console.log("Sidecar WebSocket closed");
			this.ws = null;
			// Reconnect after a delay if process is still running
			if (this.process && !this.process.killed) {
				setTimeout(() => this.connectWebSocket(), 2000);
			}
		});

		ws.addEventListener("error", (err) => {
			console.error("Sidecar WebSocket error:", err);
		});

		this.ws = ws;
	}
}
