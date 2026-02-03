# Testing the Styles Management Feature

## Pre-Testing Setup

1. **Delete the database** (schema has changed):
   ```bash
   rm ~/.rzem-ai-inference/inference.db
   ```

2. **Start the application**:
   ```bash
   npm run tauri:dev
   ```

## Test Plan

### ✅ Test 1: LoRA Import & Auto-Style Creation

**Objective**: Verify that importing a LoRA automatically creates a default style.

**Steps**:
1. Navigate to Models view
2. Import a LoRA file (with trigger words if possible)
3. Navigate to Styles view (new menu item with palette icon)
4. Verify a new style appears named "[LoRA Name] (default)"
5. Click the style to view details
6. Verify:
   - Template contains trigger words + `{{prompt}}`
   - LoRA is linked with strength 1.0
   - Category is "lora"

**Expected**: Style automatically created with trigger words pre-configured.

---

### ✅ Test 2: Create Custom Style

**Objective**: Test manual style creation.

**Steps**:
1. In Styles view, click "New Style" button
2. Fill in the form:
   - Name: "Cinematic Portrait"
   - Description: "Professional cinematic look"
   - Template: `cinematic lighting, {{prompt}}, 8k, professional photography`
   - Category: "photography"
   - Mark as favorite: Yes
3. Preview with test prompt: "woman smiling"
4. Verify preview shows: `cinematic lighting, woman smiling, 8k, professional photography`
5. Click "Create"

**Expected**: Style appears in grid, shows in favorites filter, preview works correctly.

---

### ✅ Test 3: Apply Style in Generation

**Objective**: Verify style application and template rendering.

**Steps**:
1. Navigate to Generate view
2. Toggle "Style" section (middle button in toolbar)
3. Style dropdown should appear above prompt input
4. Select "Cinematic Portrait" style
5. Verify preview shows:
   - Template text
   - Any associated LoRAs (if added)
6. Click "Apply Style" button
7. Verify green checkmark appears
8. Enter prompt: `sunset`
9. Generate image
10. Check backend logs or queue to verify final prompt is:
    `cinematic lighting, sunset, 8k, professional photography`

**Expected**: Template correctly replaces `{{prompt}}` with user input.

---

### ✅ Test 4: Style with LoRAs

**Objective**: Test LoRA activation when applying a style.

**Steps**:
1. In Styles view, edit the auto-created LoRA style
2. Or create a new style and add LoRAs via detail panel
3. Go to Generate view
4. Apply the style
5. Check that the LoRAs are automatically activated
6. Verify their strengths match the style configuration

**Expected**: Applying style activates all associated LoRAs with correct strengths.

---

### ✅ Test 5: Style Management

**Objective**: Test CRUD operations.

**Steps**:
1. **Filter by category**: Click "lora" category, verify only LoRA styles show
2. **Filter by favorites**: Click favorites filter, verify only favorited styles show
3. **Search**: Type in search box, verify filtering works
4. **Edit**: Click edit on a style card, modify name, save
5. **Delete**: Delete a style, confirm deletion
6. **Usage tracking**: Use a style multiple times, verify usage count increments

**Expected**: All CRUD operations work, filters function correctly.

---

### ✅ Test 6: Clear Style

**Objective**: Verify style clearing functionality.

**Steps**:
1. Apply a style in Generate view
2. Click the X button next to "Manage styles" button
3. Verify:
   - Style dropdown resets to "Select a style..."
   - Template is cleared (prompt renders as-is)
   - Applied checkmark disappears

**Expected**: Style cleared successfully.

---

### ✅ Test 7: Navigation

**Objective**: Verify navigation works.

**Steps**:
1. Click "Styles" in left sidebar (palette icon)
2. Verify StylesView loads
3. Click "Manage styles" button in Generate view StyleDropdown
4. Verify navigates to /styles route

**Expected**: Navigation works from both entry points.

---

### ✅ Test 8: Edge Cases

**Objective**: Test error handling.

**Steps**:
1. Create style with empty name → Should not allow save
2. Create style with no `{{prompt}}` placeholder → Should still save and work
3. Template with multiple `{{prompt}}` → Should replace all occurrences
4. Delete database while app running → Should handle gracefully
5. Apply style then immediately delete it → Should handle gracefully

**Expected**: Graceful error handling, no crashes.

---

## Backend Verification

### Database Schema

Run SQLite to verify tables created:

```bash
sqlite3 ~/.rzem-ai-inference/inference.db
```

```sql
.tables  -- Should show: styles, style_loras, style_examples
.schema styles
.schema style_loras
.schema style_examples
PRAGMA table_info(loras);  -- Should show default_strength, strength_min, strength_max
```

### API Commands

Test Tauri commands via browser console (if using dev tools):

```javascript
// Get all styles
await invoke('get_all_styles')

// Get style detail
await invoke('get_style_detail', { styleId: 'some-uuid' })

// Create style
await invoke('create_style', { request: {
  name: 'Test',
  promptTemplate: '{{prompt}}',
  defaultStrength: 1.0,
  strengthMin: 0.5,
  strengthMax: 1.5,
  isFavorite: false
}})

// Render template
await invoke('render_style_template', {
  template: 'cinematic, {{prompt}}, 4k',
  userPrompt: 'sunset'
})
// Should return: "cinematic, sunset, 4k"
```

---

## Known Limitations

1. **Example Capture UI**: Backend ready but UI integration deferred
   - Can manually call: `stylesStore.addExample(styleId, 'prompt'|'image', content)`
   - Future enhancement: Context menus and dialogs

2. **LoRA Panel**: Removed and replaced with Styles system
   - Manual LoRA management now done via Styles
   - For one-off LoRA use without styles, users can create temporary styles

---

## Success Criteria

- ✅ All database tables created correctly
- ✅ LoRA import creates default style
- ✅ Style CRUD operations work
- ✅ Template rendering works ({{prompt}} replacement)
- ✅ Style application activates LoRAs
- ✅ Navigation works (sidebar + manage button)
- ✅ Filters (category, favorites, search) work
- ✅ Usage tracking increments
- ✅ No compilation errors (backend or frontend)
- ✅ No runtime errors in console

---

## Troubleshooting

**Issue**: Styles don't appear after creating
- **Fix**: Check browser console for errors, verify database initialized

**Issue**: Template not rendering
- **Fix**: Check `getFinalPrompt()` called in GenerateView line 311

**Issue**: LoRAs not activating
- **Fix**: Check `applyStyle()` in generation store, verify LoRA IDs match

**Issue**: Navigation not working
- **Fix**: Check router registration and WorkspaceNav component

**Issue**: Database errors
- **Fix**: Delete database and restart app (schema changed)
