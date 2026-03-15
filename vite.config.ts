import { fileURLToPath, URL } from "node:url";

import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [vue(), tailwindcss()],
	root: "src/mainview",
	base: "./",
	build: {
		outDir: "../../dist/renderer",
		emptyOutDir: true,
	},
	resolve: {
		alias: {
			"@": fileURLToPath(new URL("./src/mainview", import.meta.url)),
		},
	},
	server: {
		port: 1978,
		strictPort: true,
		headers: {
			"Access-Control-Allow-Origin": "*",
		},
	},
});
