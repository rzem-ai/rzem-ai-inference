<template>
  <div class="tag-manager">
    <!-- Header -->
    <div class="tag-header">
      <span class="section-title">Tags</span>
      <Button
        v-if="selectedTags.length > 0"
        label="Clear"
        text
        size="small"
        @click="clearSelection" />
    </div>

    <!-- Loading State -->
    <div v-if="tagsStore.isLoading" class="loading-state">
      <i class="pi pi-spin pi-spinner"></i>
    </div>

    <!-- Empty State -->
    <div v-else-if="tagsStore.tags.length === 0" class="empty-tags">
      <p>No tags yet</p>
      <small>Tags will appear as you add them to images</small>
    </div>

    <template v-else>
      <!-- Popular Tags -->
      <div v-if="tagsStore.popularTags.length > 0" class="tag-section">
        <div class="section-label">Popular</div>
        <div class="tag-list">
          <Chip
            v-for="tag in tagsStore.popularTags"
            :key="tag.id"
            :label="`${tag.name} (${tag.usageCount})`"
            :class="{ active: selectedTags.includes(tag.name) }"
            :style="tag.color ? { '--tag-color': tag.color } : {}"
            @click="toggleTag(tag.name)"
            @contextmenu.prevent="(e: MouseEvent) => handleContextMenu(e, tag)" />
        </div>
      </div>

      <!-- Tags by Category -->
      <div
        v-for="(categoryTags, category) in tagsStore.tagsByCategory"
        :key="category"
        class="tag-section">
        <div class="section-label">{{ formatCategoryName(category) }}</div>
        <div class="tag-list">
          <Chip
            v-for="tag in categoryTags"
            :key="tag.id"
            :label="tag.name"
            :class="{ active: selectedTags.includes(tag.name) }"
            :style="tag.color ? { '--tag-color': tag.color } : {}"
            @click="toggleTag(tag.name)"
            @contextmenu.prevent="(e: MouseEvent) => handleContextMenu(e, tag)" />
        </div>
      </div>
    </template>

    <!-- Context Menu -->
    <ContextMenu ref="contextMenuRef" :model="contextMenuItems" />

    <!-- Delete Confirmation -->
    <ConfirmDialog />

    <!-- Edit Tag Dialog -->
    <Dialog v-model:visible="editDialogVisible" header="Edit Tag" modal :style="{ width: '350px' }">
      <div v-if="editingTag" class="edit-tag-form">
        <div class="form-field">
          <label>Name</label>
          <InputText v-model="editFormData.name" />
        </div>
        <div class="form-field">
          <label>Category</label>
          <InputText v-model="editFormData.category" placeholder="Optional category" />
        </div>
        <div class="form-field">
          <label>Color</label>
          <div class="color-picker">
            <button
              v-for="color in tagColors"
              :key="color"
              class="color-option"
              :class="{ active: editFormData.color === color }"
              :style="{ backgroundColor: color }"
              @click="editFormData.color = color"></button>
            <button
              class="color-option color-none"
              :class="{ active: !editFormData.color }"
              @click="editFormData.color = undefined">
              <i class="pi pi-ban"></i>
            </button>
          </div>
        </div>
      </div>
      <template #footer>
        <Button label="Cancel" severity="secondary" @click="editDialogVisible = false" />
        <Button label="Save" @click="saveTagEdit" />
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConfirm } from 'primevue/useconfirm'
import { useTagsStore, type Tag } from '@/stores/tags'
import { useGalleryStore } from '@/stores/gallery'
import Chip from 'primevue/chip'
import Button from 'primevue/button'
import ContextMenu from 'primevue/contextmenu'
import ConfirmDialog from 'primevue/confirmdialog'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'

const tagsStore = useTagsStore()
const galleryStore = useGalleryStore()
const confirm = useConfirm()

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null)
const contextTag = ref<Tag | null>(null)
const editDialogVisible = ref(false)
const editingTag = ref<Tag | null>(null)
const editFormData = ref({
  name: '',
  color: undefined as string | undefined,
  category: undefined as string | undefined,
})

const selectedTags = computed(() => galleryStore.filters.tags)

const tagColors = [
  '#ef4444',
  '#f97316',
  '#eab308',
  '#22c55e',
  '#14b8a6',
  '#3b82f6',
  '#6366f1',
  '#a855f7',
  '#ec4899',
]

const contextMenuItems = computed(() => [
  {
    label: 'Edit Tag',
    icon: 'pi pi-pencil',
    command: () => {
      if (contextTag.value) {
        openEditDialog(contextTag.value)
      }
    },
  },
  {
    label: 'Filter by this tag',
    icon: 'pi pi-filter',
    command: () => {
      if (contextTag.value) {
        toggleTag(contextTag.value.name)
      }
    },
  },
  { separator: true },
  {
    label: 'Delete Tag',
    icon: 'pi pi-trash',
    class: 'p-menuitem-danger',
    command: () => {
      if (contextTag.value) {
        confirmDeleteTag(contextTag.value)
      }
    },
  },
])

const formatCategoryName = (category: string): string => {
  if (category === 'uncategorized') return 'Other'
  return category.charAt(0).toUpperCase() + category.slice(1)
}

const toggleTag = (tagName: string) => {
  const currentTags = [...galleryStore.filters.tags]
  const index = currentTags.indexOf(tagName)

  if (index >= 0) {
    currentTags.splice(index, 1)
  } else {
    currentTags.push(tagName)
  }

  galleryStore.setFilter({ tags: currentTags })
}

const clearSelection = () => {
  galleryStore.setFilter({ tags: [] })
}

const handleContextMenu = (event: MouseEvent, tag: Tag) => {
  contextTag.value = tag
  contextMenuRef.value?.show(event)
}

const openEditDialog = (tag: Tag) => {
  editingTag.value = tag
  editFormData.value = {
    name: tag.name,
    color: tag.color,
    category: tag.category,
  }
  editDialogVisible.value = true
}

const saveTagEdit = async () => {
  if (!editingTag.value) return

  await tagsStore.updateTag(editingTag.value.id, {
    name: editFormData.value.name,
    color: editFormData.value.color,
    category: editFormData.value.category,
  })

  editDialogVisible.value = false
  editingTag.value = null
}

const confirmDeleteTag = (tag: Tag) => {
  confirm.require({
    message: `Are you sure you want to delete the tag "${tag.name}"?${
      tag.usageCount > 0 ? ` It is used by ${tag.usageCount} image${tag.usageCount > 1 ? 's' : ''}.` : ''
    }`,
    header: 'Delete Tag',
    icon: 'pi pi-exclamation-triangle',
    rejectClass: 'p-button-secondary',
    acceptClass: 'p-button-danger',
    accept: async () => {
      await tagsStore.deleteTag(tag.id)
    },
  })
}
</script>

<style scoped>
@reference "tailwindcss";

.tag-manager {
  @apply flex flex-col gap-3 overflow-hidden;
}

.tag-header {
  @apply flex items-center justify-between px-3;
}

.section-title {
  @apply text-xs font-semibold uppercase tracking-wide;
  color: var(--color-gray-500);
}

.loading-state {
  @apply flex items-center justify-center p-4;
  color: var(--color-gray-500);
}

.empty-tags {
  @apply flex flex-col items-center gap-1 p-4 text-center;

  p {
    @apply text-sm;
    color: var(--color-gray-500);
  }

  small {
    @apply text-xs;
    color: var(--color-gray-600);
  }
}

.tag-section {
  @apply flex flex-col gap-2 px-3;
}

.section-label {
  @apply text-xs font-medium;
  color: var(--color-gray-400);
}

.tag-list {
  @apply flex flex-wrap gap-1.5;
}

.tag-list :deep(.p-chip) {
  @apply cursor-pointer transition-all text-xs;
  background-color: var(--color-gray-700);
  color: var(--color-gray-300);
  border: 1px solid transparent;

  &:hover {
    background-color: var(--color-gray-600);
  }

  &.active {
    background-color: var(--color-blue-900);
    border-color: var(--color-blue-500);
    color: var(--color-blue-200);
  }

  /* Custom color indicator */
  &[style*="--tag-color"]::before {
    content: '';
    @apply w-2 h-2 rounded-full mr-1.5;
    background-color: var(--tag-color);
  }
}

.edit-tag-form {
  @apply flex flex-col gap-4;
}

.form-field {
  @apply flex flex-col gap-2;

  label {
    @apply text-sm font-medium;
    color: var(--color-gray-300);
  }
}

.color-picker {
  @apply flex flex-wrap gap-2;
}

.color-option {
  @apply w-7 h-7 rounded-full border-2 border-transparent cursor-pointer transition-transform;

  &:hover {
    @apply scale-110;
  }

  &.active {
    @apply border-white scale-110;
    box-shadow: 0 0 0 2px var(--color-blue-500);
  }

  &.color-none {
    @apply flex items-center justify-center text-xs;
    background-color: var(--color-gray-700);
    color: var(--color-gray-400);
  }
}
</style>
