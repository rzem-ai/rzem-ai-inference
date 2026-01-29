<template>
  <div
    class="bundle-card border border-surface-700 rounded-lg p-4 hover:border-primary transition-colors cursor-pointer"
    :class="{ 'border-primary bg-primary-900/10': bundle.isActive }"
    @click="$emit('activate', bundle)"
  >
    <!-- Bundle Header -->
    <div class="flex items-start justify-between mb-3">
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
          <h4 class="font-semibold text-surface-100">{{ bundle.name }}</h4>
          <Tag
            v-if="bundle.isActive"
            value="Active"
            severity="success"
            class="text-xs"
          />
          <Tag
            v-if="!bundle.isComplete"
            value="Incomplete"
            severity="warning"
            class="text-xs"
          />
          <Tag
            v-if="bundle.bundleType === 'user_created'"
            value="Custom"
            severity="info"
            class="text-xs"
          />
        </div>
        <p v-if="bundle.description" class="text-sm text-surface-400 m-0">
          {{ bundle.description }}
        </p>
      </div>

      <!-- Actions -->
      <div v-if="bundle.bundleType === 'user_created'" class="flex gap-1">
        <Button
          icon="pi pi-pencil"
          text
          rounded
          size="small"
          severity="secondary"
          @click.stop="$emit('edit', bundle)"
        />
        <Button
          icon="pi pi-trash"
          text
          rounded
          size="small"
          severity="danger"
          @click.stop="$emit('delete', bundle)"
        />
      </div>
    </div>

    <!-- Bundle Metadata -->
    <div class="flex flex-wrap gap-3 text-xs text-surface-500 mb-3">
      <div class="flex items-center gap-1">
        <i class="pi pi-box"></i>
        <span>{{ bundle.modelFamily.toUpperCase() }}</span>
      </div>
      <div v-if="bundle.totalVramMb" class="flex items-center gap-1">
        <i class="pi pi-server"></i>
        <span>{{ formatVram(bundle.totalVramMb) }}</span>
      </div>
      <div class="flex items-center gap-1">
        <i class="pi pi-th-large"></i>
        <span>{{ bundle.components.length }} components</span>
      </div>
      <div v-if="bundle.defaultSteps" class="flex items-center gap-1">
        <i class="pi pi-sliders-h"></i>
        <span>{{ bundle.defaultSteps }} steps</span>
      </div>
    </div>

    <!-- Component Grid -->
    <div class="grid grid-cols-2 gap-2">
      <div
        v-for="comp in bundle.components"
        :key="comp.id"
        class="text-xs px-3 py-2 bg-surface-800 rounded-lg flex items-center justify-between gap-2"
      >
        <div class="flex items-center gap-2 min-w-0">
          <i
            :class="comp.isAvailable ? 'pi pi-check-circle text-green-400' : 'pi pi-times-circle text-red-400'"
            class="shrink-0"
          ></i>
          <span class="truncate text-surface-300">{{ formatComponentRole(comp.role) }}</span>
        </div>
        <Tag
          v-if="comp.quantization"
          :value="comp.quantization"
          severity="contrast"
          class="text-xs shrink-0"
        />
      </div>
    </div>

    <!-- Warning for Missing Components -->
    <div v-if="missingComponents.length > 0" class="mt-3 p-2 bg-red-900/20 border border-red-800/50 rounded text-xs text-red-300">
      <i class="pi pi-exclamation-triangle mr-1"></i>
      Missing: {{ missingComponents.join(', ') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { BundleInfo } from '@/stores/bundles'
import Button from 'primevue/button'
import Tag from 'primevue/tag'

interface Props {
  bundle: BundleInfo
}

const props = defineProps<Props>()

const emit = defineEmits<{
  activate: [bundle: BundleInfo]
  edit: [bundle: BundleInfo]
  delete: [bundle: BundleInfo]
}>()

const missingComponents = computed(() => {
  return props.bundle.components
    .filter(c => !c.isAvailable)
    .map(c => formatComponentRole(c.role))
})

function formatComponentRole(role: string): string {
  const roleMap: Record<string, string> = {
    transformer: 'Transformer',
    t5: 'T5 Encoder',
    clip: 'CLIP',
    vae: 'VAE',
    clip_tokenizer: 'CLIP Tok',
    t5_tokenizer: 'T5 Tok',
  }
  return roleMap[role] || role
}

function formatVram(vramMb: number): string {
  if (vramMb >= 1000) {
    return `${(vramMb / 1000).toFixed(1)} GB`
  }
  return `${vramMb} MB`
}
</script>

<style scoped>
.bundle-card {
  transition: all 0.2s ease;
}

.bundle-card:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
}
</style>
