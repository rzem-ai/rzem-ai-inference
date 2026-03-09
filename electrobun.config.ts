import type { ElectrobunConfig } from "electrobun";

export default {
	app: {
		name: "RZEM AI Inference",
		identifier: "ai.rzem.inference",
		version: "0.5.0",
	},
	build: {
		bun: {
			entrypoint: "src/bun/index.ts",
		},
		// Vite builds the Vue frontend to frontend/dist/, we copy from there
		copy: {
			"frontend/dist/index.html": "views/mainview/index.html",
			"frontend/dist/assets": "views/mainview/assets",
		},
		// Ignore Vite output in watch mode — HMR handles view rebuilds
		watchIgnore: ["frontend/dist/**"],
		mac: {
			bundleCEF: false,
		},
		linux: {
			bundleCEF: false,
		},
		win: {
			bundleCEF: false,
		},
	},
} satisfies ElectrobunConfig;
