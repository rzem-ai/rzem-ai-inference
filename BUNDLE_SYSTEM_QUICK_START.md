# Model Bundle System - Quick Start Guide

## 🚀 Get Started in 5 Minutes

This guide will help you test the new model bundle system immediately.

## Prerequisites

- FLUX models downloaded (Schnell or Dev)
- HuggingFace cache at `~/.cache/huggingface/hub`
- App database at `~/.rzem-ai-inference/rzem.db`

## Quick Test Steps

### 1. Start the Application

```bash
cd /home/alex/Dev/Work/rzem-ai-inference
npm run tauri:dev
```

**Expected:** App launches without errors

### 2. Navigate to Models View

Click **Models** in the sidebar or navigation

**Expected:** Models view loads with Downloads tab active

### 3. Switch to Bundles Tab

Click **Bundles** button in the header (next to Downloads)

**Expected:**
- Sidebar shows bundle list area
- Right panel shows empty state or bundle details
- Green/yellow banner shows active bundle status (or "No bundle active")

### 4. Scan for Models

Click **"Scan Models"** button

**Expected:**
- Button shows loading spinner
- After 2-10 seconds, toast notification appears:
  - "Found X components, added Y new, created Z bundles"
- Bundles appear in sidebar list
- Active bundle banner may update

### 5. View Bundle Details

Click on any bundle in the sidebar

**Expected:**
- Right panel shows bundle information:
  - Bundle name and description
  - Model family (FLUX / Z-INDEX)
  - Total VRAM estimate
  - Component count
  - Default steps/guidance
- Component list shows 4-6 components:
  - Transformer ✅
  - T5 Encoder ✅
  - CLIP Encoder ✅
  - VAE Decoder ✅
  - (Tokenizers if detected)
- Green checkmarks indicate available components

### 6. Activate a Bundle

With bundle selected, click **"Activate"** button

**Expected:**
- Toast notification: "Bundle Activated"
- Green "Active" tag appears on bundle
- Active bundle banner updates at top
- Other bundles lose Active tag

### 7. Generate an Image

1. Go to **Generate** view
2. Enter any prompt (e.g., "a cat")
3. Click **Generate**

**Expected:**
- Image generates successfully
- Console shows: `INFO Loading models from active bundle`
- Check backend logs for bundle ID

### 8. Check Console Logs

Open browser DevTools console (F12)

**Look for:**
```
INFO Loading models from active bundle
bundle_id="black-forest-labs-flux-1-schnell-full"
```

**Or in terminal (backend logs):**
```
INFO Using active model bundle
bundle_id=Some("black-forest-labs-flux-1-schnell-full")
```

### 9. Create Custom Bundle (Optional)

1. Click **"Create Bundle"** button
2. Dialog opens
3. Enter name: "My Test Bundle"
4. Select model family: FLUX
5. Select components (use dropdown for each role)
6. Click **"Create Bundle"**

**Expected:**
- Toast: "Bundle Created"
- Dialog closes
- New bundle appears in list with "Custom" tag

### 10. Delete Custom Bundle (Optional)

1. Find your custom bundle in list
2. Click trash icon
3. Confirm deletion

**Expected:**
- Confirmation dialog appears
- After confirming, bundle removed
- Toast: "Bundle Deleted"

## Verification Commands

### Check Database Schema

```bash
sqlite3 ~/.rzem-ai-inference/rzem.db << 'EOF'
.tables
.schema model_bundles
SELECT COUNT(*) as total_bundles FROM model_bundles;
SELECT COUNT(*) as total_components FROM model_components;
EOF
```

**Expected Output:**
```
Total tables include: model_components, model_bundles, bundle_components
Schema shows all fields
total_bundles = N (number of bundles)
total_components = M (number of components)
```

### Check Active Bundle

```bash
sqlite3 ~/.rzem-ai-inference/rzem.db \
  "SELECT id, name, is_active FROM model_bundles WHERE is_active = 1;"
```

**Expected:** Shows the active bundle (or empty if none active)

### View Bundle Components

```bash
sqlite3 ~/.rzem-ai-inference/rzem.db << 'EOF'
SELECT
  b.name as bundle_name,
  bc.component_role,
  c.name as component_name,
  c.component_type,
  c.is_available
FROM bundle_components bc
JOIN model_bundles b ON bc.bundle_id = b.id
JOIN model_components c ON bc.component_id = c.id
WHERE b.is_active = 1;
EOF
```

**Expected:** Shows all components of the active bundle

## Troubleshooting

### Issue: "No bundles found" after scan

**Diagnosis:**
```bash
# Check if models exist
ls -la ~/.cache/huggingface/hub/models--black-forest-labs--FLUX.1-schnell/

# Check scan errors in console
# Look for "Failed to scan repository" messages
```

**Solutions:**
1. Verify FLUX models downloaded
2. Check HuggingFace cache location
3. Re-download models if missing

### Issue: "Bundle has missing components"

**Diagnosis:**
```bash
# Check component availability in database
sqlite3 ~/.rzem-ai-inference/rzem.db \
  "SELECT name, file_path, is_available FROM model_components WHERE is_available = 0;"
```

**Solutions:**
1. Rescan models
2. Verify files exist at listed paths
3. Download missing components
4. Choose different bundle

### Issue: Generation fails with active bundle

**Diagnosis:**
```bash
# Check backend logs
RUST_LOG=debug npm run tauri:dev

# Look for path resolution errors
```

**Solutions:**
1. Deactivate bundle (falls back to legacy mode)
2. Check bundle completeness
3. Verify component file permissions
4. Rescan models

### Issue: TypeScript errors on build

**Diagnosis:**
```bash
npm run build 2>&1 | grep "error TS" | grep -v "TS6133"
```

**Expected:** Only TS6133 warnings (unused variables - cosmetic only)

**If other errors:**
1. Check if pre-existing (not from bundle system)
2. Verify all imports present
3. Check TypeScript version compatibility

## Success Indicators

✅ **All working correctly if:**
- Bundles tab loads without errors
- Scan detects your models
- Bundles can be activated
- Active bundle shows in banner
- Image generation works with bundle
- Console logs show bundle usage
- No runtime errors in console

## Performance Benchmarks

Run these to verify performance:

### Database Operations
```bash
sqlite3 ~/.rzem-ai-inference/rzem.db << 'EOF'
.timer on
SELECT * FROM model_bundles WHERE is_active = 1;
EOF
```
**Expected:** <1ms

### Component Scan
```bash
# Time the scan operation
# Click "Scan Models" and measure time
```
**Expected:** 2-10 seconds depending on cache size

### Bundle Activation
```bash
# Time from clicking Activate to seeing Active tag
```
**Expected:** <500ms

## Next Steps After Testing

### If Everything Works ✅
1. Document any new findings
2. Add screenshots to user guide
3. Create demo video (optional)
4. Prepare release notes
5. Consider additional features from wishlist

### If Issues Found ❌
1. Check issue against Known Issues section
2. Review troubleshooting steps
3. Check console for detailed errors
4. Report bugs with reproduction steps
5. Rollback to legacy mode if critical

## CLI Testing (Bonus)

If you have a CLI tool:

```bash
# Scan models
rzem-cli models scan

# List bundles
rzem-cli bundles list

# Activate bundle
rzem-cli bundles activate <bundle-id>

# Show active
rzem-cli bundles status
```

Note: CLI commands not yet implemented, but easy to add using existing backend functions.

## Database Reset (If Needed)

To start fresh:

```bash
# Backup first
cp ~/.rzem-ai-inference/rzem.db ~/.rzem-ai-inference/rzem.db.backup

# Reset bundle tables only
sqlite3 ~/.rzem-ai-inference/rzem.db << 'EOF'
DELETE FROM bundle_components;
DELETE FROM model_bundles;
DELETE FROM model_components;
EOF

# Or delete entire database (will recreate on next run)
rm ~/.rzem-ai-inference/rzem.db
```

**Warning:** This will delete all bundles. Model files in HuggingFace cache are not affected.

## Support

### Getting Help

1. **Check logs**: Console (F12) and terminal output
2. **Review docs**: This guide and user guide
3. **Database queries**: Use SQL commands above to inspect state
4. **Fallback**: Deactivate all bundles to use legacy mode

### Reporting Issues

Include:
1. Steps to reproduce
2. Expected vs actual behavior
3. Console errors (frontend and backend)
4. Database state (bundle list, active bundle)
5. System info (OS, VRAM, model versions)

## Summary

The bundle system is ready to use! Follow these steps to test:

1. ✅ Start app
2. ✅ Go to Models → Bundles
3. ✅ Click "Scan Models"
4. ✅ Click a bundle to view
5. ✅ Click "Activate"
6. ✅ Generate an image
7. ✅ Verify bundle usage in logs

**Total time:** ~5 minutes

**Result:** Flexible, powerful model management! 🎉

---

*For detailed documentation, see BUNDLE_SYSTEM_USER_GUIDE.md and BUNDLE_SYSTEM_FINAL_SUMMARY.md*
