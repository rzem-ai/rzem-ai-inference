<template>
  <Dialog
    v-model:visible="visible"
    :header="isEditing ? 'Edit Folder' : 'Create Folder'"
    modal
    :style="{ width: '400px' }"
    @hide="resetForm">
    <div class="folder-form">
      <div class="form-field">
        <label for="folder-name">Name</label>
        <InputText
          id="folder-name"
          v-model="formData.name"
          placeholder="Enter folder name"
          :class="{ 'p-invalid': errors.name }"
          @keyup.enter="handleSubmit" />
        <small v-if="errors.name" class="p-error">{{ errors.name }}</small>
      </div>

      <div v-if="!isEditing && availableParents.length > 0" class="form-field">
        <label for="parent-folder">Parent Folder</label>
        <Select
          id="parent-folder"
          v-model="formData.parentId"
          :options="parentOptions"
          optionLabel="label"
          optionValue="value"
          placeholder="None (root level)"
          showClear
          class="w-full" />
      </div>

      <div class="form-field">
        <label>Color</label>
        <div class="color-picker">
          <button
            v-for="color in folderColors"
            :key="color.value"
            class="color-option"
            :class="{ active: formData.color === color.value }"
            :style="{ backgroundColor: color.value }"
            :title="color.label"
            @click="formData.color = color.value"></button>
          <button
            class="color-option color-none"
            :class="{ active: !formData.color }"
            title="Default"
            @click="formData.color = undefined">
            <i class="pi pi-ban"></i>
          </button>
        </div>
      </div>
    </div>

    <template #footer>
      <Button label="Cancel" severity="secondary" @click="visible = false" />
      <Button :label="isEditing ? 'Save' : 'Create'" :loading="isSubmitting" @click="handleSubmit" />
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useFoldersStore, type FolderNode } from '@/stores/folders'
import Dialog from 'primevue/dialog'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Button from 'primevue/button'

interface Props {
  folder?: FolderNode | null
  defaultParentId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  folder: null,
  defaultParentId: null,
})

const visible = defineModel<boolean>('visible', { default: false })

// Emit events for parent coordination (used by parent component via v-model)
defineEmits<{
  created: [folder: FolderNode]
  updated: [folder: FolderNode]
}>()

const foldersStore = useFoldersStore()

const formData = ref({
  name: '',
  parentId: undefined as string | undefined,
  color: undefined as string | undefined,
})

const errors = ref({
  name: '',
})

const isSubmitting = ref(false)

const isEditing = computed(() => !!props.folder)

const folderColors = [
  { label: 'Red', value: '#ef4444' },
  { label: 'Orange', value: '#f97316' },
  { label: 'Amber', value: '#f59e0b' },
  { label: 'Yellow', value: '#eab308' },
  { label: 'Lime', value: '#84cc16' },
  { label: 'Green', value: '#22c55e' },
  { label: 'Teal', value: '#14b8a6' },
  { label: 'Cyan', value: '#06b6d4' },
  { label: 'Blue', value: '#3b82f6' },
  { label: 'Indigo', value: '#6366f1' },
  { label: 'Purple', value: '#a855f7' },
  { label: 'Pink', value: '#ec4899' },
]

// Get available parent folders (excluding current folder and its descendants when editing)
const availableParents = computed(() => {
  if (!isEditing.value) {
    return foldersStore.flatFolders
  }

  // When editing, exclude the folder itself and all its descendants
  const excludeIds = new Set<string>()
  const collectDescendants = (folder: FolderNode) => {
    excludeIds.add(folder.id)
    folder.children.forEach(collectDescendants)
  }
  if (props.folder) {
    collectDescendants(props.folder)
  }

  return foldersStore.flatFolders.filter((f) => !excludeIds.has(f.id))
})

const parentOptions = computed(() =>
  availableParents.value.map((f) => ({
    label: f.path.length > 0 ? `${f.path.join(' / ')} / ${f.name}` : f.name,
    value: f.id,
  }))
)

// Initialize form when folder prop changes
watch(
  () => [props.folder, props.defaultParentId, visible.value],
  () => {
    if (visible.value) {
      if (props.folder) {
        formData.value = {
          name: props.folder.name,
          parentId: props.folder.parentId || undefined,
          color: props.folder.color || undefined,
        }
      } else {
        formData.value = {
          name: '',
          parentId: props.defaultParentId || undefined,
          color: undefined,
        }
      }
      errors.value = { name: '' }
    }
  },
  { immediate: true }
)

const validateForm = (): boolean => {
  errors.value = { name: '' }

  if (!formData.value.name.trim()) {
    errors.value.name = 'Name is required'
    return false
  }

  if (formData.value.name.length > 100) {
    errors.value.name = 'Name must be 100 characters or less'
    return false
  }

  return true
}

const handleSubmit = async () => {
  if (!validateForm()) return

  isSubmitting.value = true

  try {
    if (isEditing.value && props.folder) {
      // Update existing folder
      const success = await foldersStore.updateFolder(props.folder.id, {
        name: formData.value.name.trim(),
        color: formData.value.color,
      })
      if (success) {
        visible.value = false
      }
    } else {
      // Create new folder
      const folder = await foldersStore.createFolder(
        formData.value.name.trim(),
        formData.value.parentId,
        formData.value.color
      )
      if (folder) {
        visible.value = false
      }
    }
  } finally {
    isSubmitting.value = false
  }
}

const resetForm = () => {
  formData.value = {
    name: '',
    parentId: undefined,
    color: undefined,
  }
  errors.value = { name: '' }
}
</script>

<style scoped>
@reference "tailwindcss";

.folder-form {
  @apply flex flex-col gap-4;
}

.form-field {
  @apply flex flex-col gap-2;

  label {
    @apply text-sm font-medium;
    color: var(--color-slate-300);
  }
}

.color-picker {
  @apply flex flex-wrap gap-2;
}

.color-option {
  @apply w-8 h-8 rounded-full border-2 border-transparent cursor-pointer transition-transform;

  &:hover {
    @apply scale-110;
  }

  &.active {
    @apply border-white scale-110;
    box-shadow: 0 0 0 2px var(--color-blue-500);
  }

  &.color-none {
    @apply flex items-center justify-center;
    background-color: var(--color-slate-700);
    color: var(--color-slate-400);
  }
}
</style>
