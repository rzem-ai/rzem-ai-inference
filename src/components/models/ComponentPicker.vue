<template>
  <div class="component-picker">
    <div class="flex items-center gap-2 mb-2">
      <label class="text-sm font-medium">
        {{ label }}
        <span v-if="required" class="text-red-500">*</span>
      </label>
      <Tag
        v-if="selectedComponent?.quantization"
        :value="selectedComponent.quantization"
        severity="info"
        class="text-xs"
      />
      <Tag
        v-if="selectedComponent && !selectedComponent.isAvailable"
        value="Unavailable"
        severity="danger"
        class="text-xs"
      />
    </div>

    <Select
      :model-value="selectedId"
      :options="components"
      option-label="name"
      option-value="id"
      placeholder="Select component..."
      class="w-full"
      :invalid="required && !selectedId"
      @update:model-value="handleSelect"
    >
      <template #value="slotProps">
        <div v-if="slotProps.value && selectedComponent" class="flex items-center gap-2">
          <i
            :class="selectedComponent.isAvailable ? 'pi pi-check-circle text-green-500' : 'pi pi-times-circle text-red-500'"
          ></i>
          <span>{{ selectedComponent.name }}</span>
        </div>
        <span v-else class="text-surface-500">Select component...</span>
      </template>

      <template #option="slotProps">
        <div class="flex flex-col">
          <div class="flex items-center gap-2 mb-1">
            <i
              :class="slotProps.option.isAvailable ? 'pi pi-check-circle text-green-500' : 'pi pi-times-circle text-red-500'"
            ></i>
            <span class="font-medium">{{ slotProps.option.name }}</span>
            <Tag
              v-if="slotProps.option.quantization"
              :value="slotProps.option.quantization"
              severity="info"
              class="text-xs"
            />
          </div>
          <div class="flex gap-3 text-xs text-surface-500">
            <span>{{ slotProps.option.format }}</span>
            <span v-if="slotProps.option.vramMb">{{ formatVram(slotProps.option.vramMb) }} VRAM</span>
            <span v-if="slotProps.option.isSharded">{{ slotProps.option.shardCount }} shards</span>
            <span v-if="slotProps.option.repoId" class="truncate">{{ slotProps.option.repoId }}</span>
          </div>
        </div>
      </template>

      <template #empty>
        <div class="text-center py-3 text-surface-500">
          <i class="pi pi-inbox text-2xl mb-2"></i>
          <p class="text-sm">No {{ label.toLowerCase() }} components found</p>
          <p class="text-xs">Run "Scan Models" to detect components</p>
        </div>
      </template>
    </Select>

    <!-- Component Details -->
    <div v-if="selectedComponent" class="mt-2 p-3 bg-surface-50 rounded-lg text-xs space-y-1">
      <div class="flex justify-between">
        <span class="text-surface-600">Repository:</span>
        <span class="font-mono">{{ selectedComponent.repoId || 'Unknown' }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-surface-600">Format:</span>
        <span>{{ selectedComponent.format }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-surface-600">Size:</span>
        <span>{{ formatFileSize(selectedComponent.fileSize) }}</span>
      </div>
      <div v-if="selectedComponent.vramMb" class="flex justify-between">
        <span class="text-surface-600">VRAM:</span>
        <span>{{ formatVram(selectedComponent.vramMb) }}</span>
      </div>
      <div v-if="selectedComponent.architecture" class="flex justify-between">
        <span class="text-surface-600">Architecture:</span>
        <span>{{ selectedComponent.architecture }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-surface-600">Status:</span>
        <span :class="selectedComponent.isAvailable ? 'text-green-600' : 'text-red-600'">
          {{ selectedComponent.isAvailable ? 'Available' : 'Missing' }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ComponentRecord } from '@/stores/bundles'
import Select from 'primevue/select'
import Tag from 'primevue/tag'

interface Props {
  label: string
  role: string
  components: ComponentRecord[]
  selectedId?: string
  required?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: false,
})

const emit = defineEmits<{
  'update:selectedId': [value: string | undefined]
}>()

const selectedComponent = computed(() => {
  if (!props.selectedId) return null
  return props.components.find(c => c.id === props.selectedId) || null
})

function handleSelect(componentId: string) {
  emit('update:selectedId', componentId)
}

function formatVram(vramMb: number): string {
  if (vramMb >= 1000) {
    return `${(vramMb / 1000).toFixed(1)} GB`
  }
  return `${vramMb} MB`
}

function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(1)} ${units[unitIndex]}`
}
</script>

<style scoped>
.component-picker :deep(.p-select) {
  width: 100%;
}
</style>
