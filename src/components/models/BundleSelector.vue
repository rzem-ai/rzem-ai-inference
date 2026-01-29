<template>
  <div class="bundle-selector">
    <!-- Header with Scan Button -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold">Model Bundles</h3>
      <Button
        label="Scan Models"
        icon="pi pi-refresh"
        size="small"
        :loading="bundlesStore.isScanning"
        @click="handleScan"
      />
    </div>

    <!-- Scan Result Message -->
    <Message v-if="scanMessage" :severity="scanSeverity" @close="scanMessage = null">
      {{ scanMessage }}
    </Message>

    <!-- Error Message -->
    <Message v-if="bundlesStore.error" severity="error" @close="bundlesStore.clearError()">
      {{ bundlesStore.error }}
    </Message>

    <!-- Loading State -->
    <div v-if="bundlesStore.isLoading" class="flex items-center justify-center py-8">
      <ProgressSpinner style="width: 50px; height: 50px" />
    </div>

    <!-- Empty State -->
    <div v-else-if="bundlesStore.bundles.length === 0" class="text-center py-8 text-surface-500">
      <i class="pi pi-inbox text-4xl mb-3"></i>
      <p class="mb-2">No model bundles found</p>
      <p class="text-sm">Click "Scan Models" to detect installed models</p>
    </div>

    <!-- Bundle List -->
    <div v-else class="space-y-3">
      <div
        v-for="bundle in bundlesStore.bundles"
        :key="bundle.id"
        class="bundle-card border border-surface-200 rounded-lg p-4 hover:border-primary transition-colors cursor-pointer"
        :class="{ 'border-primary bg-primary-50': bundle.isActive }"
        @click="handleActivate(bundle)"
      >
        <!-- Bundle Header -->
        <div class="flex items-start justify-between mb-2">
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <h4 class="font-semibold">{{ bundle.name }}</h4>
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
            </div>
            <p v-if="bundle.description" class="text-sm text-surface-600">
              {{ bundle.description }}
            </p>
          </div>

          <!-- Actions -->
          <div class="flex gap-2">
            <Button
              v-if="bundle.bundleType === 'user_created'"
              icon="pi pi-pencil"
              text
              rounded
              size="small"
              severity="secondary"
              @click.stop="$emit('editBundle', bundle)"
            />
            <Button
              v-if="bundle.bundleType === 'user_created'"
              icon="pi pi-trash"
              text
              rounded
              size="small"
              severity="danger"
              @click.stop="handleDelete(bundle)"
            />
          </div>
        </div>

        <!-- Bundle Metadata -->
        <div class="flex flex-wrap gap-3 text-sm text-surface-600 mb-3">
          <div class="flex items-center gap-1">
            <i class="pi pi-tag"></i>
            <span>{{ formatBundleType(bundle.bundleType) }}</span>
          </div>
          <div class="flex items-center gap-1">
            <i class="pi pi-box"></i>
            <span>{{ bundle.modelFamily.toUpperCase() }}</span>
          </div>
          <div v-if="bundle.totalVramMb" class="flex items-center gap-1">
            <i class="pi pi-server"></i>
            <span>{{ bundlesStore.formatVram(bundle.totalVramMb) }} VRAM</span>
          </div>
          <div class="flex items-center gap-1">
            <i class="pi pi-th-large"></i>
            <span>{{ bundle.components.length }} components</span>
          </div>
        </div>

        <!-- Component List -->
        <div class="grid grid-cols-2 gap-2">
          <div
            v-for="comp in bundle.components"
            :key="comp.id"
            class="text-xs px-2 py-1 bg-surface-100 rounded flex items-center gap-1"
          >
            <i
              :class="comp.isAvailable ? 'pi pi-check-circle text-green-500' : 'pi pi-times-circle text-red-500'"
            ></i>
            <span class="truncate">{{ formatComponentRole(comp.role) }}</span>
            <span v-if="comp.quantization" class="text-surface-500">({{ comp.quantization }})</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBundlesStore } from '@/stores/bundles'
import type { BundleInfo } from '@/stores/bundles'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import Message from 'primevue/message'
import ProgressSpinner from 'primevue/progressspinner'

const emit = defineEmits<{
  editBundle: [bundle: BundleInfo]
}>()

const bundlesStore = useBundlesStore()
const confirm = useConfirm()
const toast = useToast()

const scanMessage = ref<string | null>(null)
const scanSeverity = ref<'success' | 'info' | 'warn' | 'error'>('success')

onMounted(async () => {
  await bundlesStore.initialize()
})

async function handleScan() {
  scanMessage.value = null

  try {
    const result = await bundlesStore.scanModels()

    if (result.componentsAdded > 0 || result.bundlesCreated > 0) {
      scanMessage.value = `Found ${result.componentsFound} components, added ${result.componentsAdded} new, created ${result.bundlesCreated} bundles`
      scanSeverity.value = 'success'
    } else {
      scanMessage.value = 'Scan complete. No new models found.'
      scanSeverity.value = 'info'
    }

    toast.add({
      severity: 'success',
      summary: 'Scan Complete',
      detail: scanMessage.value,
      life: 3000,
    })
  } catch (err) {
    scanSeverity.value = 'error'
    scanMessage.value = `Scan failed: ${err}`
  }
}

async function handleActivate(bundle: BundleInfo) {
  if (bundle.isActive) {
    return // Already active
  }

  if (!bundle.isComplete) {
    toast.add({
      severity: 'warn',
      summary: 'Incomplete Bundle',
      detail: 'This bundle is missing required components',
      life: 3000,
    })
    return
  }

  try {
    await bundlesStore.setActiveBundle(bundle.id)

    toast.add({
      severity: 'success',
      summary: 'Bundle Activated',
      detail: `${bundle.name} is now active`,
      life: 3000,
    })
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: 'Activation Failed',
      detail: String(err),
      life: 5000,
    })
  }
}

function handleDelete(bundle: BundleInfo) {
  confirm.require({
    message: `Delete bundle "${bundle.name}"? This cannot be undone.`,
    header: 'Confirm Deletion',
    icon: 'pi pi-exclamation-triangle',
    acceptClass: 'p-button-danger',
    accept: async () => {
      try {
        await bundlesStore.deleteBundle(bundle.id)

        toast.add({
          severity: 'success',
          summary: 'Bundle Deleted',
          detail: `${bundle.name} has been deleted`,
          life: 3000,
        })
      } catch (err) {
        toast.add({
          severity: 'error',
          summary: 'Deletion Failed',
          detail: String(err),
          life: 5000,
        })
      }
    },
  })
}

function formatBundleType(type: string): string {
  switch (type) {
    case 'auto_discovered':
      return 'Auto-Discovered'
    case 'user_created':
      return 'User Created'
    case 'system':
      return 'System'
    default:
      return type
  }
}

function formatComponentRole(role: string): string {
  const roleMap: Record<string, string> = {
    transformer: 'Transformer',
    t5: 'T5 Encoder',
    clip: 'CLIP Encoder',
    vae: 'VAE Decoder',
    clip_tokenizer: 'CLIP Tokenizer',
    t5_tokenizer: 'T5 Tokenizer',
  }
  return roleMap[role] || role
}
</script>

<style scoped>
.bundle-card {
  transition: all 0.2s ease;
}

.bundle-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
</style>
