# Batch Generation Guide

**Version:** 1.0
**Last Updated:** 2026-01-27

## Overview

The Batch Generation feature allows you to generate multiple images from a single template by substituting variables with data from CSV or JSON files. This is perfect for creating variations of prompts, testing different combinations, or generating large sets of images with systematic variations.

## Key Features

- **3-Step Wizard Interface** - Guided process for loading data, creating templates, and confirming generation
- **Two Processing Modes:**
  - **As-Is Mode** - One image per row (N rows = N images)
  - **Combinatorial Mode** - All combinations (2×3×2 = 12 images)
- **Template History** - Reuse previously used templates
- **Live Preview** - See rendered prompts before generating
- **Same-Seed Generation** - All images use identical seed for direct comparison
- **Error Detection** - Highlights template errors before generation

---

## Quick Start

### 1. Prepare Your Data

Create a CSV or JSON file with your variable data:

**Example CSV (`styles.csv`):**
```csv
style,subject,mood
watercolor,cat,peaceful
oil painting,dog,energetic
digital art,bird,mysterious
```

**Example JSON (`styles.json`):**
```json
[
  {"style": "watercolor", "subject": "cat", "mood": "peaceful"},
  {"style": "oil painting", "subject": "dog", "mood": "energetic"},
  {"style": "digital art", "subject": "bird", "mood": "mysterious"}
]
```

### 2. Open Batch Script Dialog

Click the **"Batch Script"** button in the Generate view.

### 3. Follow the 3-Step Wizard

#### **Step 1: Load Data**
1. Choose how to load data:
   - **File Picker** - Click "Choose File" and select your CSV/JSON
   - **Drag & Drop** - Drag a file onto the drop zone
   - **Paste** - Paste CSV/JSON text directly

2. Select processing mode:
   - **Use data as-is** - Creates one image per row (3 images)
   - **Generate all combinations** - Creates all permutations (shows count)

3. Review the data preview table
4. Click **"Next: Template"**

#### **Step 2: Template**
1. (Optional) Click a recent template to load it
2. Write your template using `{{ variable }}` syntax:
   ```
   A {{ style }} painting of a {{ subject }} with a {{ mood }} atmosphere
   ```
3. See live preview of rendered prompts
4. Fix any errors shown in red
5. Click **"Next: Confirm"**

#### **Step 3: Confirm & Submit**
1. Review the summary:
   - Data source filename
   - Processing mode and image count
   - Template text
   - Generation settings (steps, CFG, size, seed, model)
2. Review final preview (first 5 prompts shown)
3. Click **"Generate X Images"**

All images will be queued with the same seed for consistent comparison.

---

## Template Syntax

### Basic Variables

Use `{{ variable_name }}` to insert data from your CSV/JSON columns:

```
A {{ style }} painting of {{ subject }}
```

**Result:**
- `A watercolor painting of cat`
- `A oil painting of dog`
- `A digital art of bird`

### Built-in Filters

Apply transformations using the pipe `|` operator:

#### **upper** - Convert to uppercase
```
{{ subject | upper }}
```
Result: `CAT`, `DOG`, `BIRD`

#### **lower** - Convert to lowercase
```
{{ style | lower }}
```
Result: `watercolor`, `oil painting`, `digital art`

#### **title** - Title case
```
{{ mood | title }}
```
Result: `Peaceful`, `Energetic`, `Mysterious`

#### **default** - Provide fallback value
```
{{ color | default("blue") }}
```
If `color` column doesn't exist, uses `blue`

### Conditionals

Use if/else logic in templates:

```
A {{ style }} painting of {{ subject }}{% if mood == "peaceful" %}, serene and calm{% else %}, dynamic and vibrant{% endif %}
```

---

## Processing Modes Explained

### As-Is Mode

**How it works:** Uses each row exactly as provided

**Input CSV (3 rows):**
```csv
style,subject
watercolor,cat
oil painting,dog
digital art,bird
```

**Output:** 3 images
1. `watercolor cat`
2. `oil painting dog`
3. `digital art bird`

**Use when:**
- You have pre-planned combinations
- Each row is a complete variation
- You want full control over pairings

---

### Combinatorial Mode

**How it works:** Generates Cartesian product of all unique values

**Input CSV (2 rows):**
```csv
style,subject
watercolor,cat
oil painting,dog
```

**Unique values extracted:**
- `style`: [watercolor, oil painting]
- `subject`: [cat, dog]

**Output:** 4 images (2×2)
1. `watercolor cat`
2. `watercolor dog`
3. `oil painting cat`
4. `oil painting dog`

**Calculation:** `style_count × subject_count = total_images`

**Use when:**
- Testing all possible combinations
- Exploring parameter spaces
- Systematic variation testing

**Example with 3 columns:**
```csv
style,subject,lighting
watercolor,cat,soft
oil painting,dog,dramatic
```

Unique values:
- `style`: [watercolor, oil painting] = 2
- `subject`: [cat, dog] = 2
- `lighting`: [soft, dramatic] = 2

**Output:** 8 images (2×2×2)

---

## Data Format Specifications

### CSV Format

**Requirements:**
- First row must be headers (column names)
- No empty header names
- Values can contain spaces and special characters
- Quoted values supported for commas in data

**Example:**
```csv
style,subject,description
watercolor,cat,"A fluffy, orange tabby"
oil painting,dog,"A loyal, golden retriever"
```

**Column Names:**
- Use only letters, numbers, underscores
- No spaces in column names (use `snake_case`)
- Case-sensitive (`Style` ≠ `style`)

---

### JSON Format (Array of Objects)

**Preferred format:**
```json
[
  {"style": "watercolor", "subject": "cat", "mood": "calm"},
  {"style": "oil painting", "subject": "dog", "mood": "energetic"}
]
```

**Requirements:**
- Array of objects `[{...}, {...}]`
- All objects should have same keys
- String values recommended

---

### JSON Format (Object with Arrays)

**Alternative format:**
```json
{
  "style": ["watercolor", "oil painting"],
  "subject": ["cat", "dog"],
  "mood": ["calm", "energetic"]
}
```

**Requirements:**
- All arrays must have same length
- Arrays are transposed to rows

**This becomes:**
| style | subject | mood |
|-------|---------|----------|
| watercolor | cat | calm |
| oil painting | dog | energetic |

---

## Template History

### How It Works

- Templates are **automatically saved** after successful batch generation
- Stores: template text, timestamp, image count
- Shows **5 most recent** templates in Step 2
- Persists across app restarts

### Using Recent Templates

1. Open batch dialog and load data
2. In Step 2, click any recent template card
3. Template loads into editor
4. Modify if needed or use as-is

### Template Card Format

```
A {{ style }} painting of... (18 images)
Jan 26, 2024
```

Shows:
- First 40 characters of template
- Number of images generated
- Date last used

---

## Generation Settings

### Inherited Parameters

All images in a batch inherit settings from the main Generate view:

- **Steps** - Number of diffusion steps
- **CFG Scale** - Prompt adherence strength
- **Image Size** - Width × Height
- **Model** - FLUX.1 Schnell or Dev
- **Sampler** - Euler, DPM++, etc.
- **Scheduler** - Normal, Karras, etc.
- **LoRAs** - Active LoRA configurations

### Seed Strategy: Same for All

**Behavior:**
- If seed is set (≥0): Uses that seed for all images
- If seed is random (-1): Generates ONE random seed, uses for all images

**Why?**
- Enables direct comparison of prompt variations
- Same random noise = only prompt affects result
- Perfect for A/B testing and systematic studies

**Example:**
```
Seed: 42 (or randomly frozen to 8675309)

Image 1: "watercolor cat" with seed 42
Image 2: "oil painting cat" with seed 42
Image 3: "watercolor dog" with seed 42
Image 4: "oil painting dog" with seed 42
```

All images share the same latent noise pattern, showing **pure prompt effect**.

---

## Error Handling

### Data Loading Errors

**Empty file:**
```
Error: File is empty. Please provide data.
```
→ Ensure file has content

**Invalid format:**
```
Error: Failed to parse CSV: missing header row
```
→ Add column names in first row

**JSON parse error:**
```
Error: Invalid JSON: unexpected token at line 5
```
→ Validate JSON syntax (use JSONLint.com)

---

### Template Errors

**Missing variable:**
```
Row 1: Error: undefined variable 'color'
```
→ Use `{{ color | default("blue") }}` or add column to data

**Syntax error:**
```
Error: unexpected token '}'
```
→ Check template syntax, ensure balanced braces

**Preview shows errors:**
- Red error banner displays error count
- Preview table highlights error rows in red
- "Next: Confirm" button disabled until fixed

---

### Validation Errors

**Step 1 → 2 blocked:**
```
Toast: "Data Required - Please load data before proceeding"
```
→ Load a CSV/JSON file first

**Step 2 → 3 blocked:**
```
Toast: "Template cannot be empty"
```
→ Enter a template

```
Toast: "Template has rendering errors. Please fix them before proceeding."
```
→ Fix errors shown in preview

---

## Advanced Use Cases

### 1. Testing Prompt Variations

**Goal:** Compare different artistic styles for same subject

**Data (as-is mode):**
```csv
style,subject
photorealistic,mountain landscape
anime,mountain landscape
watercolor,mountain landscape
oil painting,mountain landscape
```

**Template:**
```
{{ style }} of {{ subject }}, highly detailed, 8k
```

**Result:** 4 images, same seed, only style differs

---

### 2. Systematic Parameter Exploration

**Goal:** Test all combinations of 3 parameters

**Data (combinatorial mode):**
```csv
lighting,time,weather
golden hour,sunset,clear
blue hour,sunrise,cloudy
```

**Unique values:**
- lighting: [golden hour, blue hour] = 2
- time: [sunset, sunrise] = 2
- weather: [clear, cloudy] = 2

**Total:** 8 combinations (2×2×2)

**Template:**
```
A landscape during {{ time }} with {{ lighting }} lighting and {{ weather }} sky
```

---

### 3. Character Variations

**Goal:** Generate same character in different poses

**Data (as-is mode):**
```csv
character,pose,expression
wizard,casting spell,focused
wizard,reading book,thoughtful
wizard,walking,determined
wizard,sitting,relaxed
```

**Template:**
```
A {{ character }} {{ pose }}, {{ expression }} expression, fantasy art style
```

**Result:** 4 poses of the same wizard archetype (thanks to same seed)

---

### 4. Large-Scale Combinatorial Testing

**Data (combinatorial mode):**
```csv
subject,style,lighting,composition
portrait,anime,natural,close-up
portrait,realistic,dramatic,wide-shot
```

**Unique values:**
- subject: [portrait] = 1
- style: [anime, realistic] = 2
- lighting: [natural, dramatic] = 2
- composition: [close-up, wide-shot] = 2

**Total:** 8 combinations (1×2×2×2)

---

## Best Practices

### Data Preparation

✅ **DO:**
- Use clear, descriptive column names
- Keep values concise (avoid very long text)
- Use snake_case for column names
- Test with small datasets first (2-3 rows)
- Validate CSV/JSON syntax before loading

❌ **DON'T:**
- Use spaces in column names
- Mix data types in same column
- Leave columns empty in some rows
- Create massive combinatorial explosions (1000+ images)

---

### Template Writing

✅ **DO:**
- Start simple, add complexity gradually
- Use `default()` filter for optional columns
- Test template with 1-2 rows before full batch
- Use descriptive variable names
- Keep templates readable

❌ **DON'T:**
- Use undefined variables without `default()`
- Create overly complex conditional logic
- Hardcode values that should be variables
- Forget to check preview before generating

---

### Batch Size

**Recommended limits:**
- **Small batches:** 1-10 images - Quick tests
- **Medium batches:** 10-50 images - Systematic exploration
- **Large batches:** 50-200 images - Production datasets
- **Very large:** 200+ images - Use with caution (memory/time)

**Warning threshold:** 1000+ combinations
- System shows warning toast
- Still allows generation
- Consider breaking into smaller batches

---

## Troubleshooting

### Issue: Dialog shows empty content

**Solution:**
- Refresh the page
- Check browser console for errors
- Ensure dev server is running

---

### Issue: Combinatorial mode shows same count as as-is

**Cause:** All values in each column are unique (no duplicates)

**Example:**
```csv
style,subject
watercolor,cat
oil painting,dog
digital art,bird
```

Each column has 3 unique values: 3×3 = 9 combinations
But original data only has 3 rows with unique pairings.

**Solution:** Intended behavior if testing specific combinations.

---

### Issue: Template preview shows "undefined variable"

**Cause:** Column name in template doesn't match CSV/JSON

**Check:**
- Spelling: `{{ colour }}` vs data column `color`
- Case: `{{ Style }}` vs data column `style`
- Underscores: `{{ art_style }}` vs data column `artstyle`

**Solution:** Match variable names exactly to column names

---

### Issue: Generated images look identical despite different prompts

**Cause:** Seed freezing working as designed - same seed = same base image

**Check:**
- Prompts ARE different (check preview)
- Model is actually using prompt (check CFG scale)
- Prompt differences are significant enough

**Solution:**
- Increase CFG scale for stronger prompt adherence
- Make prompt variations more distinct
- Check that model supports the prompt features you're testing

---

## Example Workflows

### Example 1: Style Transfer Testing

**Files needed:** `examples/batch/style-transfer.csv`

```csv
style,subject,mood
watercolor,sunset over ocean,peaceful
oil painting,sunset over ocean,peaceful
digital art,sunset over ocean,peaceful
anime,sunset over ocean,peaceful
photorealistic,sunset over ocean,peaceful
```

**Template:**
```
{{ style }} of {{ subject }}, {{ mood }} atmosphere, highly detailed
```

**Settings:**
- Mode: As-is
- Images: 5
- Seed: Fixed (same for all)

**Result:** 5 identical subjects in different artistic styles

---

### Example 2: Character + Environment Combinations

**Files needed:** `examples/batch/character-env.csv`

```csv
character,environment
knight,forest
knight,castle
wizard,forest
wizard,castle
```

**Template:**
```
A {{ character }} in a {{ environment }}, fantasy art, detailed background
```

**Settings:**
- Mode: Combinatorial
- Images: 4 (2×2)
- Seed: Random (frozen for batch)

**Result:** All character-environment combinations

---

### Example 3: Lighting Conditions Study

**Files needed:** `examples/batch/lighting.csv`

```csv
scene,lighting,time
mountain landscape,golden hour,sunset
mountain landscape,blue hour,twilight
mountain landscape,harsh,noon
mountain landscape,soft,overcast
```

**Template:**
```
{{ scene }} during {{ time }} with {{ lighting }} lighting, photorealistic
```

**Settings:**
- Mode: As-is
- Images: 4
- Seed: 12345 (specific seed)

**Result:** Same landscape under different lighting

---

## Technical Details

### Template Rendering Engine

**Backend:** Rust `minijinja` crate (Jinja2-compatible)

**Processing:**
1. Frontend sends template + data rows to backend
2. Rust renders each row independently
3. Returns array of rendered prompts + errors
4. Frontend displays in preview table

**Error isolation:** One row's error doesn't block others

---

### Database Schema

**Table:** `batch_template_history`

```sql
CREATE TABLE batch_template_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template TEXT NOT NULL,
    used_at TEXT NOT NULL,  -- ISO 8601 timestamp
    image_count INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_batch_template_history_used_at
ON batch_template_history(used_at DESC);
```

**Cleanup:** Automatic (keeps most recent 100 entries)

---

### Tauri Commands

**Backend API:**

```rust
// Parse CSV/JSON to BatchData
batch_parse_data(content: String, format: String) -> BatchData

// Render template with data
batch_render_template(template: String, rows: Vec<Row>) -> RenderResult

// Generate Cartesian product
batch_generate_combinations(data: BatchData) -> BatchData

// Get recent templates (limit 5)
batch_get_recent_templates() -> Vec<TemplateHistoryEntry>

// Save template to history
batch_save_template(template: String, image_count: i64) -> ()
```

---

## Keyboard Shortcuts

- **Esc** - Close dialog
- **Enter** (in template editor) - New line (use "Next" button to proceed)
- **Tab** - Navigate between form fields

---

## Limits and Performance

### File Size Limits

- **CSV:** ~10MB recommended max
- **JSON:** ~5MB recommended max
- **Rows:** No hard limit, but 1000+ may be slow

### Memory Considerations

**Combinatorial explosions:**
- 5 columns × 10 values each = 100,000 combinations
- System will warn at 1000+ combinations
- Each combination uses minimal memory (~1KB)

**Rendering performance:**
- 100 rows: Instant
- 1,000 rows: <1 second
- 10,000 rows: 2-5 seconds

### Generation Queue

- All batch jobs added to standard queue
- Processed in order with other generations
- Progress tracked in Queue panel

---

## FAQ

**Q: Can I use batch generation with LoRAs?**
A: Yes! Active LoRAs from the main panel apply to all batch images.

**Q: Can I change the seed for each image?**
A: Not currently. Same seed is intentional for comparison purposes.

**Q: Can I generate with different models in one batch?**
A: No. All images use the model selected in the main panel.

**Q: Can I use random seeds for different images?**
A: No. The seed is frozen for the entire batch. This is by design for controlled comparison.

**Q: How do I export combinations without generating images?**
A: Not supported yet. Planned for future release.

**Q: Can I use database queries as data source?**
A: Not currently. CSV/JSON files only. Database support planned for future.

**Q: Can I exclude certain combinations in combinatorial mode?**
A: Not currently. Use as-is mode with pre-filtered data.

**Q: Will template history sync across devices?**
A: No. History is stored locally in SQLite database.

---

## Support

For issues, questions, or feature requests:
- Check existing documentation
- Review error messages in preview
- Test with minimal examples (2-3 rows)
- Report bugs with example data files

---

## Version History

**v1.0 (2026-01-27)**
- Initial release
- 3-step wizard interface
- As-is and combinatorial modes
- Template history
- Same-seed generation strategy
- Live preview with error detection
