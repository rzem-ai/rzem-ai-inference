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
		// Vite builds the Vue frontend to src/mainview/dist/, we copy from there
		copy: {
			"src/mainview/dist/index.html": "views/mainview/index.html",
			"src/mainview/dist/assets": "views/mainview/assets",
		},
		// Ignore Vite output in watch mode — HMR handles view rebuilds
		watchIgnore: ["src/mainview/dist/**"],
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
