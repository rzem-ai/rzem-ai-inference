# Phase 4: Gallery & Compare Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build smart gallery with search/filter, tagging system, and compare workspace for reviewing generated images.

**Architecture:** Create gallery and compare stores, build image grid UI with FTS5 search integration, implement tagging and filtering, add side-by-side comparison view with synchronized zoom, implement export functionality.

**Tech Stack:** Vue 3 + PrimeVue for UI, SQLite FTS5 for search, Pinia for state management, existing Rust backend with gallery database.

**Dependencies from Phase 3:**
- SQLite database with images, tags, and FTS5 tables
- Generation workflow saving images to database
- File system structure for image storage

---

## Task 1: Create Gallery Store

**Files:**
- Create: `src/stores/gallery.ts`
- Test: Manual store test

**Step 1: Create gallery store**

Create `src/stores/gallery.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface GalleryImage {
  id: string
  filePath: string
  thumbnailPath?: string
  createdAt: number
  width: number
  height: number
  fileSize: number
  isFavorite: boolean
  prompt: string
  negativePrompt?: string
  modelName: string
  steps?: number
  cfgScale?: number
  seed?: number
  sampler?: string
  tags: string[]
}

export interface GalleryFilters {
  searchQuery: string
  modelName?: string
  isFavorite?: boolean
  tags: string[]
  dateFrom?: number
  dateTo?: number
}

export const useGalleryStore = defineStore('gallery', () => {
  // State
  const images = ref<GalleryImage[]>([])
  const selectedImages = ref<Set<string>>(new Set())
  const filters = ref<GalleryFilters>({
    searchQuery: '',
    tags: [],
  })
  const isLoading = ref(false)

  // Getters
  const filteredImages = computed(() => {
    let result = images.value

    // Search query filter
    if (filters.value.searchQuery) {
      const query = filters.value.searchQuery.toLowerCase()
      result = result.filter(
        (img) =>
          img.prompt.toLowerCase().includes(query) ||
          img.negativePrompt?.toLowerCase().includes(query)
      )
    }

    // Model filter
    if (filters.value.modelName) {
      result = result.filter((img) => img.modelName === filters.value.modelName)
    }

    // Favorite filter
    if (filters.value.isFavorite !== undefined) {
      result = result.filter((img) => img.isFavorite === filters.value.isFavorite)
    }

    // Tags filter
    if (filters.value.tags.length > 0) {
      result = result.filter((img) =>
        filters.value.tags.every((tag) => img.tags.includes(tag))
      )
    }

    // Date range filter
    if (filters.value.dateFrom) {
      result = result.filter((img) => img.createdAt >= filters.value.dateFrom!)
    }
    if (filters.value.dateTo) {
      result = result.filter((img) => img.createdAt <= filters.value.dateTo!)
    }

    return result
  })

  const selectedImagesList = computed(() =>
    Array.from(selectedImages.value)
      .map((id) => images.value.find((img) => img.id === id))
      .filter((img): img is GalleryImage => img !== undefined)
  )

  // Actions
  async function loadImages(): Promise<void> {
    isLoading.value = true
    try {
      const result = await invoke<GalleryImage[]>('get_gallery_images', {
        limit: 100,
      })
      images.value = result
    } catch (error) {
      console.error('Failed to load gallery images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function searchImages(query: string): Promise<void> {
    if (!query.trim()) {
      await loadImages()
      return
    }

    isLoading.value = true
    try {
      const result = await invoke<GalleryImage[]>('search_gallery_images', {
        query: query.trim(),
      })
      images.value = result
    } catch (error) {
      console.error('Failed to search images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function toggleFavorite(imageId: string): Promise<boolean> {
    try {
      await invoke('toggle_favorite', { imageId })
      const image = images.value.find((img) => img.id === imageId)
      if (image) {
        image.isFavorite = !image.isFavorite
      }
      return true
    } catch (error) {
      console.error('Failed to toggle favorite:', error)
      return false
    }
  }

  async function addTag(imageId: string, tag: string): Promise<boolean> {
    try {
      await invoke('add_image_tag', { imageId, tag })
      const image = images.value.find((img) => img.id === imageId)
      if (image && !image.tags.includes(tag)) {
        image.tags.push(tag)
      }
      return true
    } catch (error) {
      console.error('Failed to add tag:', error)
      return false
    }
  }

  async function removeTag(imageId: string, tag: string): Promise<boolean> {
    try {
      await invoke('remove_image_tag', { imageId, tag })
      const image = images.value.find((img) => img.id === imageId)
      if (image) {
        image.tags = image.tags.filter((t) => t !== tag)
      }
      return true
    } catch (error) {
      console.error('Failed to remove tag:', error)
      return false
    }
  }

  async function deleteImage(imageId: string): Promise<boolean> {
    try {
      await invoke('delete_gallery_image', { imageId })
      images.value = images.value.filter((img) => img.id !== imageId)
      selectedImages.value.delete(imageId)
      return true
    } catch (error) {
      console.error('Failed to delete image:', error)
      return false
    }
  }

  function selectImage(imageId: string): void {
    selectedImages.value.add(imageId)
  }

  function deselectImage(imageId: string): void {
    selectedImages.value.delete(imageId)
  }

  function toggleSelectImage(imageId: string): void {
    if (selectedImages.value.has(imageId)) {
      selectedImages.value.delete(imageId)
    } else {
      selectedImages.value.add(imageId)
    }
  }

  function clearSelection(): void {
    selectedImages.value.clear()
  }

  function selectAll(): void {
    filteredImages.value.forEach((img) => selectedImages.value.add(img.id))
  }

  function updateFilters(newFilters: Partial<GalleryFilters>): void {
    filters.value = { ...filters.value, ...newFilters }
  }

  function clearFilters(): void {
    filters.value = {
      searchQuery: '',
      tags: [],
    }
  }

  return {
    // State
    images,
    selectedImages,
    filters,
    isLoading,
    // Getters
    filteredImages,
    selectedImagesList,
    // Actions
    loadImages,
    searchImages,
    toggleFavorite,
    addTag,
    removeTag,
    deleteImage,
    selectImage,
    deselectImage,
    toggleSelectImage,
    clearSelection,
    selectAll,
    updateFilters,
    clearFilters,
  }
})
```

**Step 2: Verify TypeScript compilation**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 3: Commit gallery store**

```bash
git add src/stores/gallery.ts
git commit -m "feat: add gallery store for image management

- Create gallery store with Pinia
- Add image loading and FTS5 search
- Add favorite, tag, and delete operations
- Add selection and filtering functionality

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Rust Backend Commands for Gallery

**Files:**
- Modify: `src-tauri/src/gallery/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test`

**Step 1: Add gallery query methods to Rust**

Update `src-tauri/src/gallery/mod.rs` to add query methods:

```rust
// Add after existing methods

pub fn get_gallery_images(&self, limit: usize) -> Result<Vec<ImageMetadata>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, file_path, prompt, created_at
         FROM images
         ORDER BY created_at DESC
         LIMIT ?1"
    )?;

    let images = stmt.query_map(params![limit], |row| {
        Ok(ImageMetadata {
            id: row.get(0)?,
            file_path: row.get(1)?,
            prompt: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(images)
}

pub fn search_gallery_images(&self, query: &str) -> Result<Vec<ImageMetadata>> {
    let mut stmt = self.conn.prepare(
        "SELECT i.id, i.file_path, i.prompt, i.created_at
         FROM images i
         JOIN images_fts fts ON i.id = fts.image_id
         WHERE images_fts MATCH ?1
         ORDER BY i.created_at DESC
         LIMIT 100"
    )?;

    let images = stmt.query_map(params![query], |row| {
        Ok(ImageMetadata {
            id: row.get(0)?,
            file_path: row.get(1)?,
            prompt: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(images)
}

pub fn toggle_favorite(&self, image_id: &str) -> Result<()> {
    self.conn.execute(
        "UPDATE images SET is_favorite = NOT is_favorite WHERE id = ?1",
        params![image_id],
    )?;
    Ok(())
}

pub fn add_image_tag(&self, image_id: &str, tag: &str) -> Result<()> {
    // Insert tag if it doesn't exist
    self.conn.execute(
        "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
        params![tag],
    )?;

    // Get tag ID
    let tag_id: i64 = self.conn.query_row(
        "SELECT id FROM tags WHERE name = ?1",
        params![tag],
        |row| row.get(0),
    )?;

    // Link image to tag
    self.conn.execute(
        "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
        params![image_id, tag_id],
    )?;

    Ok(())
}

pub fn remove_image_tag(&self, image_id: &str, tag: &str) -> Result<()> {
    self.conn.execute(
        "DELETE FROM image_tags
         WHERE image_id = ?1
         AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
        params![image_id, tag],
    )?;
    Ok(())
}

pub fn delete_gallery_image(&self, image_id: &str) -> Result<()> {
    self.conn.execute(
        "DELETE FROM images WHERE id = ?1",
        params![image_id],
    )?;
    Ok(())
}
```

**Step 2: Add Tauri commands**

Update `src-tauri/src/lib.rs` to add gallery commands:

```rust
#[command]
fn get_gallery_images(
    app_state: State<AppState>,
    limit: usize,
) -> Result<Vec<ImageMetadata>, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        db.get_gallery_images(limit)
            .map_err(|e| format!("Failed to get images: {}", e))
    } else {
        Err("Database not initialized".to_string())
    }
}

#[command]
fn search_gallery_images(
    app_state: State<AppState>,
    query: String,
) -> Result<Vec<ImageMetadata>, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        db.search_gallery_images(&query)
            .map_err(|e| format!("Failed to search images: {}", e))
    } else {
        Err("Database not initialized".to_string())
    }
}

#[command]
fn toggle_favorite(
    app_state: State<AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        db.toggle_favorite(&image_id)
            .map_err(|e| format!("Failed to toggle favorite: {}", e))?;
        Ok("Favorite toggled".to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[command]
fn add_image_tag(
    app_state: State<AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        db.add_image_tag(&image_id, &tag)
            .map_err(|e| format!("Failed to add tag: {}", e))?;
        Ok("Tag added".to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[command]
fn remove_image_tag(
    app_state: State<AppState>,
    image_id: String,
    tag: String,
) -> Result<String, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        db.remove_image_tag(&image_id, &tag)
            .map_err(|e| format!("Failed to remove tag: {}", e))?;
        Ok("Tag removed".to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[command]
fn delete_gallery_image(
    app_state: State<AppState>,
    image_id: String,
) -> Result<String, String> {
    let db = app_state
        .gallery_db
        .lock()
        .unwrap();

    if let Some(db) = db.as_ref() {
        // Get file path before deleting from database
        let images = db.get_gallery_images(1000)
            .map_err(|e| format!("Failed to get images: {}", e))?;

        if let Some(image) = images.iter().find(|img| img.id == image_id) {
            // Delete file from filesystem
            std::fs::remove_file(&image.file_path)
                .map_err(|e| format!("Failed to delete file: {}", e))?;
        }

        // Delete from database
        db.delete_gallery_image(&image_id)
            .map_err(|e| format!("Failed to delete from database: {}", e))?;

        Ok("Image deleted".to_string())
    } else {
        Err("Database not initialized".to_string())
    }
}
```

**Step 3: Register commands**

Update the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    health_check,
    init_database,
    generate_image,
    get_gallery_images,
    search_gallery_images,
    toggle_favorite,
    add_image_tag,
    remove_image_tag,
    delete_gallery_image,
])
```

**Step 4: Run tests**

Run: `cd src-tauri && cargo test`
Expected: Tests pass

**Step 5: Commit backend commands**

```bash
git add src-tauri/src/gallery/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for gallery operations

- Add get_gallery_images for retrieving images
- Add search_gallery_images using FTS5
- Add toggle_favorite, tag management, delete operations
- Register all commands in Tauri handler

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create GalleryView Component

**Files:**
- Create: `src/views/GalleryView.vue`
- Create: `src/components/gallery/ImageGrid.vue`
- Modify: `src/router/index.ts`
- Test: Manual browser test

**Step 1: Create ImageGrid component**

Create `src/components/gallery/ImageGrid.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { GalleryImage } from '@/stores/gallery'
import Image from 'primevue/image'
import Checkbox from 'primevue/checkbox'
import Button from 'primevue/button'

interface Props {
  images: GalleryImage[]
  selectedIds: Set<string>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [imageId: string]
  openDetail: [image: GalleryImage]
  toggleFavorite: [imageId: string]
}>()

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath)
}
</script>

<template>
  <div class="image-grid">
    <div
      v-for="image in images"
      :key="image.id"
      class="image-card"
      :class="{ selected: selectedIds.has(image.id) }"
    >
      <div class="image-checkbox">
        <Checkbox
          :model-value="selectedIds.has(image.id)"
          @change="emit('select', image.id)"
          binary
        />
      </div>

      <div class="image-container" @click="emit('openDetail', image)">
        <Image
          :src="getImageSrc(image.filePath)"
          :alt="image.prompt"
          preview
        />
      </div>

      <div class="image-actions">
        <Button
          icon="pi pi-heart"
          :severity="image.isFavorite ? 'danger' : 'secondary'"
          text
          rounded
          @click.stop="emit('toggleFavorite', image.id)"
          :title="image.isFavorite ? 'Remove from favorites' : 'Add to favorites'"
        />
        <span class="image-date">
          {{ new Date(image.createdAt).toLocaleDateString() }}
        </span>
      </div>

      <div class="image-info">
        <p class="image-prompt">{{ image.prompt.substring(0, 60) }}{{ image.prompt.length > 60 ? '...' : '' }}</p>
        <div class="image-meta">
          <span>{{ image.width }}×{{ image.height }}</span>
          <span>{{ image.modelName }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
  padding: 1rem;
}

.image-card {
  position: relative;
  border: 2px solid transparent;
  border-radius: 0.5rem;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
  overflow: hidden;
}

.image-card:hover {
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.image-card.selected {
  border-color: #3b82f6;
  background: #eff6ff;
}

.image-checkbox {
  position: absolute;
  top: 0.5rem;
  left: 0.5rem;
  z-index: 10;
  background: white;
  border-radius: 0.25rem;
  padding: 0.25rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.image-container {
  cursor: pointer;
  aspect-ratio: 1;
  overflow: hidden;
  background: #f3f4f6;
}

.image-container :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  border-top: 1px solid #e5e7eb;
}

.image-date {
  font-size: 0.75rem;
  color: #6b7280;
}

.image-info {
  padding: 0.75rem;
  background: #f9fafb;
}

.image-prompt {
  margin: 0 0 0.5rem 0;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: #374151;
}

.image-meta {
  display: flex;
  gap: 0.75rem;
  font-size: 0.75rem;
  color: #6b7280;
}

.image-meta span {
  display: flex;
  align-items: center;
}
</style>
```

**Step 2: Create GalleryView**

Create `src/views/GalleryView.vue`:

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useGalleryStore } from '@/stores/gallery'
import ImageGrid from '@/components/gallery/ImageGrid.vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'

const galleryStore = useGalleryStore()

onMounted(async () => {
  await galleryStore.loadImages()
})

const handleSearch = async () => {
  if (galleryStore.filters.searchQuery.trim()) {
    await galleryStore.searchImages(galleryStore.filters.searchQuery)
  } else {
    await galleryStore.loadImages()
  }
}

const handleToggleFavorite = async (imageId: string) => {
  await galleryStore.toggleFavorite(imageId)
}

const handleSelectImage = (imageId: string) => {
  galleryStore.toggleSelectImage(imageId)
}

const handleOpenDetail = (image: any) => {
  console.log('Open detail for:', image)
  // TODO: Open image detail modal (Task 4)
}
</script>

<template>
  <div class="workspace-content gallery-view">
    <div class="gallery-header">
      <h1>Gallery</h1>

      <div class="search-bar">
        <InputText
          v-model="galleryStore.filters.searchQuery"
          placeholder="Search prompts..."
          class="search-input"
          @keyup.enter="handleSearch"
        />
        <Button
          icon="pi pi-search"
          @click="handleSearch"
          :loading="galleryStore.isLoading"
        />
      </div>

      <div class="gallery-actions">
        <Button
          label="Select All"
          icon="pi pi-check-square"
          severity="secondary"
          @click="galleryStore.selectAll"
        />
        <Button
          label="Clear Selection"
          icon="pi pi-times"
          severity="secondary"
          @click="galleryStore.clearSelection"
          :disabled="galleryStore.selectedImages.size === 0"
        />
        <span class="selection-count">
          {{ galleryStore.selectedImages.size }} selected
        </span>
      </div>
    </div>

    <div v-if="galleryStore.isLoading" class="loading-state">
      <i class="pi pi-spin pi-spinner" style="font-size: 2rem"></i>
      <p>Loading images...</p>
    </div>

    <div v-else-if="galleryStore.filteredImages.length === 0" class="empty-state">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images found</p>
      <p class="empty-hint">Generate some images to see them here!</p>
    </div>

    <ImageGrid
      v-else
      :images="galleryStore.filteredImages"
      :selected-ids="galleryStore.selectedImages"
      @select="handleSelectImage"
      @open-detail="handleOpenDetail"
      @toggle-favorite="handleToggleFavorite"
    />
  </div>
</template>

<style scoped>
.gallery-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.gallery-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: white;
}

.gallery-header h1 {
  margin: 0 0 1rem 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.search-bar {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.search-input {
  flex: 1;
  max-width: 500px;
}

.gallery-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.selection-count {
  margin-left: auto;
  font-size: 0.875rem;
  color: #6b7280;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 1rem;
  color: #6b7280;
}

.empty-hint {
  font-size: 0.875rem;
  color: #9ca3af;
}
</style>
```

**Step 3: Add route**

Update `src/router/index.ts` to add gallery route:

```typescript
{
  path: '/gallery',
  name: 'gallery',
  component: () => import('../views/GalleryView.vue'),
}
```

**Step 4: Commit gallery UI**

```bash
git add src/views/GalleryView.vue src/components/gallery/ImageGrid.vue src/router/index.ts
git commit -m "feat: add gallery view with image grid

- Create GalleryView with search and selection
- Create ImageGrid component with thumbnails
- Add favorite toggle and selection UI
- Add route for gallery workspace

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Compare Store and View

**Files:**
- Create: `src/stores/compare.ts`
- Create: `src/views/CompareView.vue`
- Modify: `src/router/index.ts`
- Test: Manual browser test

**Step 1: Create compare store**

Create `src/stores/compare.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GalleryImage } from './gallery'

export const useCompareStore = defineStore('compare', () => {
  // State
  const compareImages = ref<GalleryImage[]>([])
  const maxCompareImages = 4

  // Getters
  const canAddMore = computed(() => compareImages.value.length < maxCompareImages)

  const compareCount = computed(() => compareImages.value.length)

  // Actions
  function addToCompare(image: GalleryImage): boolean {
    if (compareImages.value.length >= maxCompareImages) {
      return false
    }

    if (compareImages.value.some((img) => img.id === image.id)) {
      return false // Already in compare
    }

    compareImages.value.push(image)
    return true
  }

  function removeFromCompare(imageId: string): void {
    compareImages.value = compareImages.value.filter((img) => img.id !== imageId)
  }

  function clearCompare(): void {
    compareImages.value = []
  }

  return {
    // State
    compareImages,
    maxCompareImages,
    // Getters
    canAddMore,
    compareCount,
    // Actions
    addToCompare,
    removeFromCompare,
    clearCompare,
  }
})
```

**Step 2: Create CompareView**

Create `src/views/CompareView.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useCompareStore } from '@/stores/compare'
import { convertFileSrc } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import Image from 'primevue/image'

const compareStore = useCompareStore()

const getImageSrc = (filePath: string) => {
  return convertFileSrc(filePath)
}

const handleRemove = (imageId: string) => {
  compareStore.removeFromCompare(imageId)
}

const getParameterDiff = (image: any, compareToImage: any, param: string) => {
  if (!compareToImage) return null
  const value = image[param]
  const compareValue = compareToImage[param]
  if (value !== compareValue) {
    return 'different'
  }
  return 'same'
}
</script>

<template>
  <div class="workspace-content compare-view">
    <div class="compare-header">
      <h1>Compare Images</h1>
      <div class="compare-actions">
        <span class="compare-count">
          {{ compareStore.compareCount }} / {{ compareStore.maxCompareImages }} images
        </span>
        <Button
          label="Clear All"
          icon="pi pi-times"
          severity="secondary"
          @click="compareStore.clearCompare"
          :disabled="compareStore.compareCount === 0"
        />
      </div>
    </div>

    <div v-if="compareStore.compareCount === 0" class="empty-state">
      <i class="pi pi-images" style="font-size: 3rem; color: #9ca3af"></i>
      <p>No images to compare</p>
      <p class="empty-hint">Add images from the gallery to compare them side by side</p>
    </div>

    <div v-else class="compare-grid" :style="{ gridTemplateColumns: `repeat(${compareStore.compareCount}, 1fr)` }">
      <div
        v-for="(image, index) in compareStore.compareImages"
        :key="image.id"
        class="compare-item"
      >
        <div class="compare-image-header">
          <span class="compare-index">#{{ index + 1 }}</span>
          <Button
            icon="pi pi-times"
            severity="danger"
            text
            rounded
            size="small"
            @click="handleRemove(image.id)"
          />
        </div>

        <div class="compare-image-container">
          <Image :src="getImageSrc(image.filePath)" :alt="image.prompt" preview />
        </div>

        <div class="compare-metadata">
          <div class="metadata-item">
            <span class="metadata-label">Prompt:</span>
            <span class="metadata-value">{{ image.prompt }}</span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Model:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'modelName')"
            >
              {{ image.modelName }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Steps:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'steps')"
            >
              {{ image.steps ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">CFG:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'cfgScale')"
            >
              {{ image.cfgScale ?? 'N/A' }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Size:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'width')"
            >
              {{ image.width }}×{{ image.height }}
            </span>
          </div>

          <div class="metadata-item">
            <span class="metadata-label">Seed:</span>
            <span
              class="metadata-value"
              :class="getParameterDiff(image, compareStore.compareImages[0], 'seed')"
            >
              {{ image.seed ?? 'N/A' }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compare-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.compare-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.compare-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.compare-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.compare-count {
  font-size: 0.875rem;
  color: #6b7280;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 1rem;
  color: #6b7280;
}

.empty-hint {
  font-size: 0.875rem;
  color: #9ca3af;
}

.compare-grid {
  display: grid;
  gap: 1rem;
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.compare-item {
  display: flex;
  flex-direction: column;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 0.5rem;
  overflow: hidden;
}

.compare-image-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

.compare-index {
  font-weight: 600;
  color: #374151;
}

.compare-image-container {
  aspect-ratio: 1;
  overflow: hidden;
  background: #f3f4f6;
}

.compare-image-container :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.compare-metadata {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.metadata-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.metadata-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
}

.metadata-value {
  font-size: 0.875rem;
  color: #374151;
  word-break: break-word;
}

.metadata-value.different {
  color: #dc2626;
  font-weight: 600;
}

.metadata-value.same {
  color: #059669;
}
</style>
```

**Step 3: Add route**

Update `src/router/index.ts`:

```typescript
{
  path: '/compare',
  name: 'compare',
  component: () => import('../views/CompareView.vue'),
}
```

**Step 4: Add compare button to ImageGrid**

Update `src/components/gallery/ImageGrid.vue` to add compare emit:

```vue
const emit = defineEmits<{
  select: [imageId: string]
  openDetail: [image: GalleryImage]
  toggleFavorite: [imageId: string]
  addToCompare: [image: GalleryImage]  // Add this
}>()
```

And add button to image actions:

```vue
<Button
  icon="pi pi-clone"
  severity="secondary"
  text
  rounded
  @click.stop="emit('addToCompare', image)"
  title="Add to compare"
/>
```

**Step 5: Wire up compare in GalleryView**

Update `src/views/GalleryView.vue`:

```vue
import { useCompareStore } from '@/stores/compare'

const compareStore = useCompareStore()

const handleAddToCompare = (image: any) => {
  const success = compareStore.addToCompare(image)
  if (!success) {
    console.warn('Cannot add more images to compare')
  }
}
```

Add emit handler in template:

```vue
<ImageGrid
  :images="galleryStore.filteredImages"
  :selected-ids="galleryStore.selectedImages"
  @select="handleSelectImage"
  @open-detail="handleOpenDetail"
  @toggle-favorite="handleToggleFavorite"
  @add-to-compare="handleAddToCompare"
/>
```

**Step 6: Commit compare functionality**

```bash
git add src/stores/compare.ts src/views/CompareView.vue src/components/gallery/ImageGrid.vue src/views/GalleryView.vue src/router/index.ts
git commit -m "feat: add compare workspace for side-by-side comparison

- Create compare store with image selection
- Create CompareView with side-by-side layout
- Show parameter differences highlighting
- Add compare button to gallery images
- Wire up compare functionality

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 4 Complete!

**What We Built:**

✅ Gallery store with search, filtering, and image management
✅ Rust backend commands for gallery operations
✅ Gallery view with responsive image grid
✅ Search functionality using SQLite FTS5
✅ Favorite and selection system
✅ Compare store and workspace
✅ Side-by-side comparison with parameter diff highlighting

**What Works Now:**

1. Browse generated images in a gallery grid
2. Search images by prompt text
3. Select multiple images for batch operations
4. Mark images as favorites
5. Add images to compare workspace
6. View up to 4 images side-by-side with parameter comparison
7. See differences in generation parameters highlighted

**Not Yet Implemented (for future phases):**

- Tagging UI (needs dedicated task)
- Filter UI for dates, models, tags
- Image detail modal with full metadata
- Export functionality
- Bulk delete operations
- Metadata embedding in image files

The gallery and compare system provides excellent tools for reviewing and analyzing generated images, with a solid foundation for future enhancements!
