# Batch Scripting Examples

This directory contains example data files for the batch scripting feature.

## Overview

Batch scripting allows you to generate multiple images using template prompts and data from CSV or JSON files. All images in a batch use the **same seed** for consistent comparison of prompt variations.

## File Formats

### CSV Format

CSV files must have headers in the first row:

```csv
style,subject,lighting
watercolor,cat,soft
oil painting,dog,dramatic
digital art,bird,natural
```

**Example:** `example.csv`

### JSON Format (Array of Objects)

JSON files can be structured as an array of objects:

```json
[
  {"style": "watercolor", "subject": "cat", "lighting": "soft"},
  {"style": "oil painting", "subject": "dog", "lighting": "dramatic"}
]
```

**Example:** `example-array.json`

### JSON Format (Object with Arrays)

Alternatively, JSON can be structured as an object with parallel arrays:

```json
{
  "style": ["watercolor", "oil painting"],
  "subject": ["cat", "dog"],
  "lighting": ["soft", "dramatic"]
}
```

**Note:** All arrays must have the same length.

**Example:** `example-object.json`

## Template Syntax

Templates use Jinja2-style `{{ variable }}` syntax:

### Basic Variables

```
A {{ style }} painting of {{ subject }}
```

With `example.csv`, this generates:
- "A watercolor painting of cat"
- "A oil painting of dog"
- "A digital art painting of bird"

### Filters

Apply filters to transform variables:

```
{{ subject | upper }}, {{ style }} art, {{ lighting }} lighting
```

Supported filters:
- `upper` - Convert to uppercase
- `lower` - Convert to lowercase
- `trim` - Remove whitespace
- `title` - Title case
- `default("fallback")` - Use fallback if variable is empty

### Examples

**Simple substitution:**
```
A {{ style }} painting of {{ subject }}
```

**With filters:**
```
{{ subject | title }}, {{ style }} art style, {{ lighting | lower }} lighting
```

**With defaults:**
```
A {{ mood | default("calm") }} {{ style }} painting of {{ subject }}
```

## How to Use

1. **Open Batch Script Dialog:** Click the "Batch Script" button in the generation sidebar
2. **Load Data:** Choose a file, drag-and-drop, or paste CSV/JSON data
3. **Enter Template:** Write your template using `{{ variable }}` syntax
4. **Preview:** Review the generated prompts in the preview table
5. **Generate:** Click "Generate N Images" to queue all images

## Key Features

- **Same Seed:** All images in a batch use the same seed for consistent comparison
- **Parameter Inheritance:** Steps, CFG, size, sampler, scheduler, and LoRAs are inherited from the main generation form
- **Error Handling:** Template errors are highlighted in red; generation is blocked until fixed
- **Preview:** See all rendered prompts before generating

## Tips

- **Column Names:** Keep column names simple (no spaces or special characters work best)
- **Testing:** Start with a small file (5-10 rows) to test your template
- **Variables:** Click on available variable chips to insert them at cursor position
- **Debugging:** Use the preview table to verify prompts render correctly
- **Seed Strategy:** Since all rows use the same seed, you can directly compare how different prompts affect the same random state
