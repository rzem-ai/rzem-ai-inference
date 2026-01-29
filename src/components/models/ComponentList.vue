<template>
  <div class="component-list">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <div>
        <h3 class="text-lg font-semibold text-surface-100">Available Components</h3>
        <p class="text-sm text-surface-400 mt-1">
          Individual model components detected in your HuggingFace cache
        </p>
      </div>
      <div class="text-sm text-surface-500">
        {{ bundlesStore.availableComponents.length }} components found
      </div>
    </div>

    <!-- Filter Buttons -->
    <div class="flex gap-2 mb-4">
      <Button
        :label="`All (${bundlesStore.availableComponents.length})`"
        size="small"
        :outlined="filterType !== 'all'"
        :severity="filterType === 'all' ? 'primary' : 'secondary'"
        @click="filterType = 'all'"
      />
      <Button
        :label="`Transformers (${bundlesStore.transformerComponents.length})`"
        size="small"
        :outlined="filterType !== 'transformer'"
        :severity="filterType === 'transformer' ? 'primary' : 'secondary'"
        @click="filterType = 'transformer'"
      />
      <Button
        :label="`T5 (${bundlesStore.t5Components.length})`"
        size="small"
        :outlined="filterType !== 't5_encoder'"
        :severity="filterType === 't5_encoder' ? 'primary' : 'secondary'"
        @click="filterType = 't5_encoder'"
      />
      <Button
        :label="`CLIP (${bundlesStore.clipComponents.length})`"
        size="small"
        :outlined="filterType !== 'clip_encoder'"
        :severity="filterType === 'clip_encoder' ? 'primary' : 'secondary'"
        @click="filterType = 'clip_encoder'"
      />
      <Button
        :label="`VAE (${bundlesStore.vaeComponents.length})`"
        size="small"
        :outlined="filterType !== 'vae'"
        :severity="filterType === 'vae' ? 'primary' : 'secondary'"
        @click="filterType = 'vae'"
      />
    </div>

    <!-- Loading State -->
    <div v-if="bundlesStore.isLoading" class="flex items-center justify-center py-12">
      <ProgressSpinner style="width: 50px; height: 50px" />
    </div>

    <!-- Empty State -->
    <div v-else-if="filteredComponents.length === 0" class="text-center py-12 text-surface-500">
      <i class="pi pi-inbox text-5xl mb-3"></i>
      <p class="mb-2">No components found</p>
      <p class="text-sm">Click "Scan Models" to detect installed model components</p>
    </div>

    <!-- Component Cards -->
    <div v-else class="grid grid-cols-1 gap-3">
      <div
        v-for="component in filteredComponents"
        :key="component.id"
        class="border border-surface-700 rounded-lg p-4 hover:border-surface-600 transition-colors"
      >
        <!-- Component Header -->
        <div class="flex items-start justify-between mb-3">
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <h4 class="font-semibold text-surface-100">{{ component.name }}</h4>
              <Tag
                v-if="component.quantization"
                :value="component.quantization"
                severity="info"
                class="text-xs"
              />
              <Tag
                v-if="component.isSharded"
                :value="`${component.shardCount} shards`"
                severity="contrast"
                class="text-xs"
              />
            </div>
            <p class="text-xs text-surface-500 font-mono">
              {{ component.repoId || 'Unknown repository' }}
            </p>
          </div>

          <!-- Availability Status -->
          <div class="flex items-center gap-2">
            <i
              :class="component.isAvailable ? 'pi pi-check-circle text-green-400' : 'pi pi-times-circle text-red-400'"
              class="text-lg"
            ></i>
          </div>
        </div>

        <!-- Component Metadata -->
        <div class="grid grid-cols-3 gap-3 text-xs">
          <div>
            <span class="text-surface-500">Type:</span>
            <span class="ml-1 text-surface-300">{{ formatComponentType(component.componentType) }}</span>
          </div>
          <div>
            <span class="text-surface-500">Format:</span>
            <span class="ml-1 text-surface-300">{{ component.format }}</span>
          </div>
          <div>
            <span class="text-surface-500">Size:</span>
            <span class="ml-1 text-surface-300">{{ formatFileSize(component.fileSize) }}</span>
          </div>
          <div v-if="component.vramMb">
            <span class="text-surface-500">VRAM:</span>
            <span class="ml-1 text-surface-300">{{ formatVram(component.vramMb) }}</span>
          </div>
          <div v-if="component.architecture">
            <span class="text-surface-500">Arch:</span>
            <span class="ml-1 text-surface-300">{{ component.architecture }}</span>
          </div>
          <div>
            <span class="text-surface-500">LoRA:</span>
            <span class="ml-1" :class="component.supportsLoras ? 'text-green-400' : 'text-surface-500'">
              {{ component.supportsLoras ? 'Yes' : 'No' }}
            </span>
          </div>
        </div>

        <!-- File Path -->
        <div class="mt-3 pt-3 border-t border-surface-700">
          <div class="text-xs">
            <span class="text-surface-500">Path:</span>
            <code class="ml-2 text-surface-400 bg-surface-800 px-2 py-1 rounded">
              {{ component.filePath }}
            </code>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useBundlesStore } from '@/stores/bundles'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import ProgressSpinner from 'primevue/progressspinner'

const bundlesStore = useBundlesStore()

const filterType = ref<string>('all')

const filteredComponents = computed(() => {
  const components = bundlesStore.availableComponents
  if (filterType.value === 'all') {
    return components
  }
  return components.filter(c => c.componentType === filterType.value)
})

function formatComponentType(type: string): string {
  const typeMap: Record<string, string> = {
    transformer: 'Transformer',
    t5_encoder: 'T5 Encoder',
    clip_encoder: 'CLIP Encoder',
    vae: 'VAE Decoder',
    clip_tokenizer: 'CLIP Tokenizer',
    t5_tokenizer: 'T5 Tokenizer',
  }
  return typeMap[type] || type
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

function formatVram(vramMb: number): string {
  if (vramMb >= 1000) {
    return `${(vramMb / 1000).toFixed(1)} GB`
  }
  return `${vramMb} MB`
}
</script>

<style scoped>
.component-list :deep(.p-button-group) {
  flex-wrap: wrap;
}
</style>
