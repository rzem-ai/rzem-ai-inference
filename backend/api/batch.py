"""Batch API — CSV parsing and template rendering for batch generation."""

from __future__ import annotations

import csv
import io
import logging
import re
from typing import Any

logger = logging.getLogger(__name__)

_VAR_RE = re.compile(r"\{\{\s*(\w+)\s*\}\}")


class BatchAPI:
    """pywebview js_api mixin for batch operations."""

    def batch_parse_data(self, content: str, **kwargs) -> dict[str, Any]:
        """Parse CSV text into columns and rows.

        Returns ``{status, columns: [...], rows: [{col: val, ...}, ...]}``.
        """
        try:
            content = content.strip()
            if not content:
                return {"status": "error", "message": "No data provided"}

            reader = csv.DictReader(io.StringIO(content))
            columns = reader.fieldnames or []
            rows = [dict(row) for row in reader]

            if not columns:
                return {"status": "error", "message": "No columns detected in CSV"}
            if not rows:
                return {"status": "error", "message": "No data rows found"}

            return {
                "status": "success",
                "columns": list(columns),
                "rows": rows,
            }
        except Exception as e:
            logger.error("Failed to parse batch data: %s", e)
            return {"status": "error", "message": str(e)}

    def batch_render_template(
        self, template: str, rows: list[dict], **kwargs
    ) -> dict[str, Any]:
        """Render ``{{variable}}`` placeholders for each data row.

        Returns ``{status, rendered: [str, ...], errors: [{row, error}, ...]}``.
        """
        try:
            rendered: list[str] = []
            errors: list[dict[str, Any]] = []

            for i, row in enumerate(rows):
                try:
                    result = _VAR_RE.sub(
                        lambda m: row.get(m.group(1), m.group(0)), template
                    )
                    rendered.append(result)
                except Exception as e:
                    rendered.append("")
                    errors.append({"row": i, "error": str(e)})

            return {
                "status": "success",
                "rendered": rendered,
                "errors": errors,
            }
        except Exception as e:
            logger.error("Failed to render batch template: %s", e)
            return {"status": "error", "message": str(e)}
