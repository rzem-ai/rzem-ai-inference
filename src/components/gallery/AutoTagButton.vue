<template>
  <div class="auto-tag-button">
    <!-- Main Tag Button -->
    <Button
      :disabled="!canTag"
      :loading="autoTagStore.isTagging"
      size="small"
      severity="secondary"
      @click="handleTagClick"
      v-tooltip.bottom="tooltipText">
      <div class="flex items-center gap-2">
        <fa :icon="['fal', 'sparkles']" size="sm" />
        <span>Auto-Tag</span>
      </div>
    </Button>

    <!-- Backend Selector Dropdown -->
    <Menu ref="backendMenuRef" :model="backendMenuItems" popup />

    <!-- Settings Button -->
    <Button
      size="small"
      severity="secondary"
      text
      @click="$emit('open-settings')"
      v-tooltip.bottom="'Auto-Tag Settings'">
      <fa :icon="['fal', 'gears']" size="sm" />
    </Button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAutoTagStore, type TaggingBackend } from '@/stores/autoTag'
import { useGalleryStore } from '@/stores/gallery'
import { useTagsStore } from '@/stores/tags'
import Button from 'primevue/button'
import Menu from 'primevue/menu'

const emit = defineEmits<{
  (e: 'open-settings'): void
  (e: 'tagging-complete', results: { total: number; success: number }): void
}>()

const autoTagStore = useAutoTagStore()
const galleryStore = useGalleryStore()
const tagsStore = useTagsStore()

const backendMenuRef = ref<InstanceType<typeof Menu> | null>(null)

const selectedCount = computed(() => galleryStore.selectedImages.size)

const canTag = computed(() => {
  return selectedCount.value > 0 && autoTagStore.effectiveBackend !== null
})

const tooltipText = computed(() => {
  if (selectedCount.value === 0) {
    return 'Select images to auto-tag'
  }
  if (autoTagStore.effectiveBackend === null) {
    return 'Configure auto-tag backend in settings'
  }
  const backend = autoTagStore.effectiveBackend === 'claude' ? 'Claude Vision' : 'Local (Moondream)'
  return `Tag ${selectedCount.value} image${selectedCount.value > 1 ? 's' : ''} using ${backend}`
})

const backendMenuItems = computed(() => {
  const items = []

  if (autoTagStore.isClaudeAvailable) {
    items.push({
      label: 'Tag with Claude Vision',
      icon: 'pi pi-cloud',
      command: () => tagWithBackend('claude'),
    })
  }

  if (autoTagStore.isLocalAvailable) {
    items.push({
      label: 'Tag with Local Model',
      icon: 'pi pi-desktop',
      command: () => tagWithBackend('local'),
    })
  }

  if (items.length === 0) {
    items.push({
      label: 'No backends available',
      icon: 'pi pi-exclamation-circle',
      disabled: true,
    })
    items.push({
      separator: true,
    })
    items.push({
      label: 'Configure in Settings',
      icon: 'pi pi-cog',
      command: () => emit('open-settings'),
    })
  }

  return items
})

const handleTagClick = (event: Event) => {
  // If multiple backends available, show menu
  const availableBackends = [
    autoTagStore.isClaudeAvailable,
    autoTagStore.isLocalAvailable,
  ].filter(Boolean).length

  if (availableBackends > 1) {
    backendMenuRef.value?.toggle(event)
  } else {
    // Use the effective backend directly
    tagWithBackend(autoTagStore.effectiveBackend!)
  }
}

const tagWithBackend = async (backend: TaggingBackend) => {
  const imageIds = Array.from(galleryStore.selectedImages)
  if (imageIds.length === 0) return

  const results = await autoTagStore.tagImages(imageIds, backend)

  // Count successes
  const successCount = results.filter((r) => r.success).length

  // Refresh tags store
  await tagsStore.loadTags()

  // Refresh gallery to show new tags
  await galleryStore.loadImages()

  emit('tagging-complete', {
    total: imageIds.length,
    success: successCount,
  })
}
</script>

<style scoped>
@reference "tailwindcss";

.auto-tag-button {
  @apply flex items-center gap-1;
}
</style>
