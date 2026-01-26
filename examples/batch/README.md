# Batch Generation Examples

This directory contains example CSV and JSON files for batch generation with rzem-ai-inference.

## Files

### Basic Examples

- **`styles.csv`** / **`styles.json`** - Multi-column example with style, subject, and mood
  - Template: `A {{ style }} painting of a {{ subject }} with a {{ mood }} atmosphere`
  - Mode: As-is (5 images)

### Use Case Examples

- **`style-transfer.csv`** - Single-column example for style variations
  - Template: `A beautiful landscape, {{ style }} style`
  - Mode: As-is (8 images, same subject, different styles)

- **`character-env.csv`** - Two-column example for character/environment combinations
  - Template: `A {{ character }} standing in a {{ environment }}`
  - Mode: Combinatorial (6×6 = 36 combinations)

- **`lighting.csv`** - Two-column example for lighting studies
  - Template: `A portrait with {{ lighting }} lighting during {{ time }}`
  - Mode: Combinatorial (8×8 = 64 combinations)

## Usage

1. Open rzem-ai-inference
2. Click "Batch Script" button
3. In Step 1, drag one of these files into the dialog
4. In Step 2, enter the suggested template
5. Review the preview and generate

## Creating Your Own

See the [Batch Generation Guide](../../docs/BATCH_GENERATION_GUIDE.md) for detailed documentation on:
- CSV/JSON format specifications
- Template syntax (variables, filters, conditionals)
- Processing modes (as-is vs combinatorial)
- Advanced use cases
