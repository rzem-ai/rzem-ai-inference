/**
 * Batch operations: CSV parsing and template rendering.
 * Ported from src/bun/services/batch.ts — no changes needed (pure logic).
 */

const VAR_RE = /\{\{\s*(\w+)\s*\}\}/g;

interface BatchParseResult {
	status: "success" | "error";
	message?: string;
	columns?: string[];
	rows?: Record<string, string>[];
}

interface BatchRenderResult {
	status: "success" | "error";
	message?: string;
	rendered?: string[];
	errors?: Array<{ row: number; error: string }>;
}

export function batchParseData(content: string): BatchParseResult {
	const trimmed = content.trim();
	if (!trimmed) {
		return { status: "error", message: "No data provided" };
	}

	const lines = trimmed.split("\n").map((l) => l.trim());
	if (lines.length < 2) {
		return { status: "error", message: "No data rows found" };
	}

	// Parse CSV header
	const columns = parseCSVLine(lines[0]!);
	if (columns.length === 0) {
		return { status: "error", message: "No columns detected in CSV" };
	}

	// Parse data rows
	const rows: Record<string, string>[] = [];
	for (let i = 1; i < lines.length; i++) {
		const line = lines[i]!;
		if (!line) continue;
		const values = parseCSVLine(line);
		const row: Record<string, string> = {};
		for (let j = 0; j < columns.length; j++) {
			row[columns[j]!] = values[j] ?? "";
		}
		rows.push(row);
	}

	if (rows.length === 0) {
		return { status: "error", message: "No data rows found" };
	}

	return { status: "success", columns, rows };
}

export function batchRenderTemplate(
	template: string,
	rows: Record<string, string>[],
): BatchRenderResult {
	const rendered: string[] = [];
	const errors: Array<{ row: number; error: string }> = [];

	for (let i = 0; i < rows.length; i++) {
		try {
			const row = rows[i]!;
			const result = template.replace(
				VAR_RE,
				(_match, varName: string) => row[varName] ?? `{{${varName}}}`,
			);
			rendered.push(result);
		} catch (err) {
			rendered.push("");
			errors.push({ row: i, error: String(err) });
		}
	}

	return { status: "success", rendered, errors };
}

function parseCSVLine(line: string): string[] {
	const result: string[] = [];
	let current = "";
	let inQuotes = false;

	for (let i = 0; i < line.length; i++) {
		const char = line[i]!;

		if (inQuotes) {
			if (char === '"') {
				if (i + 1 < line.length && line[i + 1] === '"') {
					current += '"';
					i++;
				} else {
					inQuotes = false;
				}
			} else {
				current += char;
			}
		} else {
			if (char === '"') {
				inQuotes = true;
			} else if (char === ",") {
				result.push(current.trim());
				current = "";
			} else {
				current += char;
			}
		}
	}

	result.push(current.trim());
	return result;
}
