/**
 * Freedesktop thumbnail cache writer.
 *
 * GNOME's file manager (Nautilus) reads pre-generated thumbnails from
 * ~/.cache/thumbnails/large/{md5(uri)}.png per the freedesktop.org
 * thumbnail managing spec. If a thumbnail is already there with a
 * matching Thumb::MTime, Nautilus uses it directly and never invokes
 * its own thumbnailer — which in our case had been recording failures
 * and marking files as un-thumbnailable.
 *
 * Spec: https://specifications.freedesktop.org/thumbnail-spec/thumbnail-spec-latest.html
 */

import { createHash } from "crypto";
import { promises as fs, statSync } from "fs";
import { homedir } from "os";
import { join } from "path";

type ThumbSize = "normal" | "large" | "x-large";

const SIZE_PX: Record<ThumbSize, number> = {
	normal: 128,
	large: 256,
	"x-large": 512,
};

function fileUri(absPath: string): string {
	// Percent-encode per RFC 3986, but keep "/" as path separator.
	const encoded = absPath.split("/").map((seg) => encodeURIComponent(seg)).join("/");
	return `file://${encoded}`;
}

function md5Hex(input: string): string {
	return createHash("md5").update(input).digest("hex");
}

function cacheRoot(): string {
	return process.env.XDG_CACHE_HOME ?? join(homedir(), ".cache");
}

function thumbDir(size: ThumbSize): string {
	return join(cacheRoot(), "thumbnails", size);
}

function failDir(): string {
	return join(cacheRoot(), "thumbnails", "fail", "gnome-thumbnail-factory");
}

// CRC32 (IEEE 802.3) — table-based impl for injecting PNG chunks.
const CRC_TABLE: number[] = (() => {
	const t = new Array<number>(256);
	for (let n = 0; n < 256; n++) {
		let c = n;
		for (let k = 0; k < 8; k++) {
			c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
		}
		t[n] = c >>> 0;
	}
	return t;
})();

function crc32(buf: Buffer): number {
	let c = 0xffffffff;
	for (let i = 0; i < buf.length; i++) {
		c = CRC_TABLE[(c ^ buf[i]!) & 0xff]! ^ (c >>> 8);
	}
	return (c ^ 0xffffffff) >>> 0;
}

function makeTextChunk(keyword: string, text: string): Buffer {
	const data = Buffer.concat([
		Buffer.from(keyword, "latin1"),
		Buffer.from([0]),
		Buffer.from(text, "latin1"),
	]);
	const type = Buffer.from("tEXt", "ascii");
	const length = Buffer.alloc(4);
	length.writeUInt32BE(data.length, 0);
	const crc = Buffer.alloc(4);
	crc.writeUInt32BE(crc32(Buffer.concat([type, data])), 0);
	return Buffer.concat([length, type, data, crc]);
}

/**
 * Splice tEXt chunks into an existing PNG buffer, right before IEND.
 */
function injectTextChunks(png: Buffer, chunks: Buffer[]): Buffer {
	// Locate IEND: last 12 bytes of a PNG (length=0, type="IEND", crc).
	// Be defensive and scan instead of assuming position.
	let i = 8; // skip PNG signature
	while (i < png.length) {
		const len = png.readUInt32BE(i);
		const type = png.slice(i + 4, i + 8).toString("ascii");
		if (type === "IEND") {
			return Buffer.concat([png.slice(0, i), ...chunks, png.slice(i)]);
		}
		i += 12 + len;
	}
	// Fallback: append before a synthetic IEND (shouldn't happen for valid PNGs).
	return Buffer.concat([png, ...chunks]);
}

async function writeAtomic(path: string, data: Buffer): Promise<void> {
	const tmp = `${path}.${process.pid}.tmp`;
	await fs.writeFile(tmp, data, { mode: 0o600 });
	await fs.rename(tmp, path);
}

export interface ThumbnailOptions {
	size?: ThumbSize;
}

/**
 * Generate and cache a freedesktop-spec thumbnail for `imagePath` so
 * Nautilus/Files can display it without invoking its own thumbnailer.
 *
 * No-ops on non-Linux platforms and silently swallows all errors —
 * thumbnail generation must never break image persistence.
 */
export async function writeFreedesktopThumbnail(
	imagePath: string,
	options: ThumbnailOptions = {},
): Promise<void> {
	if (process.platform !== "linux") return;
	const size: ThumbSize = options.size ?? "large";
	try {
		const uri = fileUri(imagePath);
		const hash = md5Hex(uri);
		const dir = thumbDir(size);
		await fs.mkdir(dir, { recursive: true, mode: 0o700 });

		const stat = statSync(imagePath);
		const mtime = Math.floor(stat.mtimeMs / 1000).toString();

		// Skip if an up-to-date thumbnail already exists.
		const outPath = join(dir, `${hash}.png`);
		try {
			const existing = statSync(outPath);
			if (existing.mtimeMs >= stat.mtimeMs) return;
		} catch { /* not present, fall through */ }

		const sharp = (await import("sharp")).default;
		const pngBuffer = await sharp(imagePath)
			.resize({ width: SIZE_PX[size], height: SIZE_PX[size], fit: "inside", withoutEnlargement: true })
			.png({ compressionLevel: 9 })
			.toBuffer();

		const annotated = injectTextChunks(pngBuffer, [
			makeTextChunk("Thumb::URI", uri),
			makeTextChunk("Thumb::MTime", mtime),
		]);

		await writeAtomic(outPath, annotated);

		// Clear any stale fail marker so file managers trust the new thumb.
		try {
			await fs.unlink(join(failDir(), `${hash}.png`));
		} catch { /* no fail entry — fine */ }
	} catch (err) {
		console.warn(`[thumbnail] failed for ${imagePath}:`, err instanceof Error ? err.message : err);
	}
}

/**
 * Walk `outputDir` and generate missing freedesktop thumbnails for
 * every .png in it. Intended to run once on startup so previously-
 * generated images pick up thumbnails retroactively. Throttled to
 * avoid pegging CPU.
 */
export async function backfillOutputThumbnails(outputDir: string): Promise<void> {
	if (process.platform !== "linux") return;
	let entries: string[];
	try {
		entries = await fs.readdir(outputDir);
	} catch {
		return;
	}
	const pngs = entries.filter((f) => f.toLowerCase().endsWith(".png"));
	for (const name of pngs) {
		await writeFreedesktopThumbnail(join(outputDir, name));
	}
}
