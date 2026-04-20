/**
 * mDNS discovery service for finding inference engines on the local network.
 * Browses for `_rzem-ai._tcp` services advertised by the Python engine.
 */

import Bonjour, { type Browser, type Service } from "bonjour-service";

export interface DiscoveredServer {
	name: string;
	host: string;
	port: number;
	device: string;
	version: string;
}

export interface DiscoveryService {
	start(): void;
	stop(): void;
	getServers(): DiscoveredServer[];
	onServerUp(cb: (server: DiscoveredServer) => void): () => void;
	onServerDown(cb: (server: DiscoveredServer) => void): () => void;
}

function getIPv4Address(service: Service): string {
	if (service.addresses?.length) {
		const ipv4 = service.addresses.find(
			(a) => !a.includes(":"), // exclude IPv6
		);
		if (ipv4) return ipv4;
	}
	if (service.referer?.family === "IPv4") {
		return service.referer.address;
	}
	return service.host;
}

function parseTxt(txt: Record<string, any> | undefined): { version: string; device: string } {
	if (!txt) return { version: "unknown", device: "unknown" };
	const toString = (v: unknown) =>
		v instanceof Buffer ? v.toString() : String(v ?? "unknown");
	return {
		version: toString(txt.version),
		device: toString(txt.device),
	};
}

export function createDiscoveryService(): DiscoveryService {
	const servers = new Map<string, DiscoveredServer>();
	const upCallbacks: Array<(server: DiscoveredServer) => void> = [];
	const downCallbacks: Array<(server: DiscoveredServer) => void> = [];

	let bonjour: Bonjour | null = null;
	let browser: Browser | null = null;

	function handleUp(service: Service) {
		const host = getIPv4Address(service);
		const port = service.port;
		const key = `${host}:${port}`;
		const { version, device } = parseTxt(service.txt);

		const server: DiscoveredServer = {
			name: service.name,
			host,
			port,
			device,
			version,
		};

		servers.set(key, server);
		console.log(`mDNS: engine discovered — ${server.name} at ${host}:${port} (${device})`);

		for (const cb of upCallbacks) {
			try {
				cb(server);
			} catch (err) {
				console.error("mDNS onServerUp callback error:", err);
			}
		}
	}

	function handleDown(service: Service) {
		const host = getIPv4Address(service);
		const port = service.port;
		const key = `${host}:${port}`;
		const server = servers.get(key);

		if (server) {
			servers.delete(key);
			console.log(`mDNS: engine gone — ${server.name} at ${host}:${port}`);

			for (const cb of downCallbacks) {
				try {
					cb(server);
				} catch (err) {
					console.error("mDNS onServerDown callback error:", err);
				}
			}
		}
	}

	return {
		start() {
			if (browser) return;

			bonjour = new Bonjour();
			browser = bonjour.find({ type: "rzem-ai" });
			browser.on("up", handleUp);
			browser.on("down", handleDown);

			console.log("mDNS: browsing for _rzem-ai._tcp services...");
		},

		stop() {
			if (browser) {
				browser.stop();
				browser = null;
			}
			if (bonjour) {
				bonjour.destroy();
				bonjour = null;
			}
			servers.clear();
		},

		getServers(): DiscoveredServer[] {
			return Array.from(servers.values());
		},

		onServerUp(cb) {
			upCallbacks.push(cb);
			return () => {
				const idx = upCallbacks.indexOf(cb);
				if (idx >= 0) upCallbacks.splice(idx, 1);
			};
		},

		onServerDown(cb) {
			downCallbacks.push(cb);
			return () => {
				const idx = downCallbacks.indexOf(cb);
				if (idx >= 0) downCallbacks.splice(idx, 1);
			};
		},
	};
}
