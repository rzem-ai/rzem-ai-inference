import type { ElectrobunConfig } from "electrobun";

// Enable code signing and notarization when Apple credentials are available
const hasSigningCredentials = !!(
	process.env.ELECTROBUN_DEVELOPER_ID &&
	process.env.ELECTROBUN_TEAMID &&
	process.env.ELECTROBUN_APPLEID &&
	process.env.ELECTROBUN_APPLEIDPASS
);

export default {
	app: {
		name: "Inference",
		identifier: "ai.rzem.inference",
		version: "0.5.0",
	},
	build: {
		bun: {
			entrypoint: "src/bun/index.ts",
		},
		// Vite builds to dist/, we copy from there
		copy: {
			"dist/index.html": "views/mainview/index.html",
			"dist/assets": "views/mainview/assets",
		},
		// Ignore Vite output in watch mode — HMR handles view rebuilds
		watchIgnore: ["dist/**"],
		mac: {
			bundleCEF: false,
			codesign: hasSigningCredentials,
			notarize: hasSigningCredentials,
		},
		linux: {
			bundleCEF: false,
		},
		win: {
			bundleCEF: false,
		},
	},
} satisfies ElectrobunConfig;
