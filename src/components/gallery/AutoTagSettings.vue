<template>
  <Dialog
    v-model:visible="visibleModel"
    header="Auto-Tag Settings"
    modal
    :style="{ width: '450px' }"
    :closable="true"
    :draggable="false">
    <div class="settings-container">
      <!-- Enable Auto-Tagging -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">Enable Auto-Tagging</span>
          <span class="setting-desc">Allow automatic tagging of images</span>
        </div>
        <ToggleSwitch v-model="localSettings.enabled" />
      </div>

      <!-- Auto-tag on Generation -->
      <div class="setting-row" :class="{ disabled: !localSettings.enabled }">
        <div class="setting-info">
          <span class="setting-label">Tag on Generation</span>
          <span class="setting-desc">Automatically tag images after generation</span>
        </div>
        <ToggleSwitch v-model="localSettings.auto_tag_on_generation" :disabled="!localSettings.enabled" />
      </div>

      <!-- Preferred Backend -->
      <div class="setting-section">
        <span class="section-title">Preferred Backend</span>

        <div class="backend-options">
          <div
            class="backend-option"
            :class="{
              active: localSettings.preferred_backend === 'local',
              disabled: !autoTagStore.isLocalAvailable,
            }"
            @click="localSettings.preferred_backend = 'local'">
            <div class="backend-header">
              <Monitor class="w-5 h-5" />
              <span class="backend-name">Local (Moondream)</span>
            </div>
            <span class="backend-desc">Free, works offline, runs on your GPU</span>
            <div v-if="!autoTagStore.isLocalAvailable" class="backend-status">
              <template v-if="autoTagStore.isDownloading">
                <ProgressBar :value="autoTagStore.downloadProgressPercent" :showValue="true" />
              </template>
              <template v-else>
                <Button size="small" severity="secondary" @click.stop="downloadModel">
                  <Download class="w-4 h-4 mr-2" />
                  Download Model (~1.8GB)
                </Button>
              </template>
            </div>
            <div v-else class="backend-status ready">
              <Check class="w-4 h-4" />
              <span>Ready</span>
            </div>
          </div>

          <div
            class="backend-option"
            :class="{
              active: localSettings.preferred_backend === 'claude',
              disabled: !autoTagStore.isClaudeAvailable && !showApiKeyInput,
            }"
            @click="localSettings.preferred_backend = 'claude'">
            <div class="backend-header">
              <Cloud class="w-5 h-5" />
              <span class="backend-name">Claude Vision API</span>
            </div>
            <span class="backend-desc">Higher quality, requires API key, uses credits</span>
            <div class="backend-status">
              <template v-if="autoTagStore.isClaudeAvailable && !showApiKeyInput">
                <div class="flex items-center gap-2">
                  <Check class="w-4 h-4 text-green-400" />
                  <span>API Key Configured</span>
                  <Button size="small" text severity="secondary" @click.stop="showApiKeyInput = true">
                    <Pencil class="w-3 h-3" />
                  </Button>
                </div>
              </template>
              <template v-else>
                <div class="api-key-input" @click.stop>
                  <InputText
                    v-model="localSettings.claude_api_key"
                    type="password"
                    placeholder="Enter Claude API Key"
                    size="small"
                    :fluid="true" />
                  <Button v-if="showApiKeyInput" size="small" text @click.stop="showApiKeyInput = false">
                    <X class="w-3 h-3" />
                  </Button>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>

      <!-- Confidence Threshold -->
      <div class="setting-section">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Minimum Confidence</span>
            <span class="setting-desc">Only add tags above this confidence threshold</span>
          </div>
          <span class="confidence-value">{{ Math.round(localSettings.min_confidence * 100) }}%</span>
        </div>
        <Slider v-model="confidenceSlider" :min="0" :max="100" :step="5" class="confidence-slider" />
      </div>

      <!-- Error Message -->
      <div v-if="autoTagStore.lastError" class="error-message">
        <AlertCircle class="w-4 h-4" />
        <span>{{ autoTagStore.lastError }}</span>
      </div>
    </div>

    <template #footer>
      <Button label="Cancel" severity="secondary" @click="handleCancel" />
      <Button label="Save" :loading="isSaving" @click="handleSave" />
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAutoTagStore, type AutoTagSettings } from '@/stores/autoTag'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import ToggleSwitch from 'primevue/toggleswitch'
import Slider from 'primevue/slider'
import InputText from 'primevue/inputtext'
import ProgressBar from 'primevue/progressbar'
import { Monitor, Cloud, Download, Check, Pencil, X, AlertCircle } from 'lucide-vue-next'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const autoTagStore = useAutoTagStore()

const visibleModel = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
})

// Local copy of settings for editing
const localSettings = ref<AutoTagSettings>({
  enabled: false,
  auto_tag_on_generation: false,
  preferred_backend: 'local',
  min_confidence: 0.6,
  claude_api_key: undefined,
})

const showApiKeyInput = ref(false)
const isSaving = ref(false)

// Slider works with 0-100, settings use 0.0-1.0
const confidenceSlider = computed({
  get: () => Math.round(localSettings.value.min_confidence * 100),
  set: (value: number) => {
    localSettings.value.min_confidence = value / 100
  },
})

// Load settings when dialog opens
watch(
  () => props.visible,
  async (isVisible) => {
    if (isVisible) {
      await autoTagStore.loadSettings()
      await autoTagStore.checkModelStatus()
      // Copy store settings to local
      localSettings.value = { ...autoTagStore.settings }
      showApiKeyInput.value = !autoTagStore.isClaudeAvailable
    }
  },
  { immediate: true }
)

const downloadModel = async () => {
  await autoTagStore.downloadModel()
}

const handleCancel = () => {
  visibleModel.value = false
}

const handleSave = async () => {
  isSaving.value = true
  try {
    await autoTagStore.saveSettings(localSettings.value)
    visibleModel.value = false
  } finally {
    isSaving.value = false
  }
}
</script>

<style scoped>
@reference "tailwindcss";

.settings-container {
  @apply flex flex-col gap-6;
}

.setting-row {
  @apply flex items-center justify-between gap-4;

  &.disabled {
    @apply opacity-50 pointer-events-none;
  }
}

.setting-info {
  @apply flex flex-col gap-0.5;
}

.setting-label {
  @apply text-sm font-medium;
  color: var(--color-slate-200);
}

.setting-desc {
  @apply text-xs;
  color: var(--color-slate-400);
}

.setting-section {
  @apply flex flex-col gap-3;
}

.section-title {
  @apply text-xs font-semibold uppercase tracking-wide;
  color: var(--color-slate-400);
}

.backend-options {
  @apply flex flex-col gap-3;
}

.backend-option {
  @apply flex flex-col gap-2 p-3 rounded-lg cursor-pointer transition-all;
  background-color: var(--color-slate-800);
  border: 2px solid transparent;

  &:hover:not(.disabled) {
    background-color: var(--color-slate-700);
  }

  &.active {
    border-color: var(--color-blue-500);
    background-color: var(--color-blue-900/30);
  }

  &.disabled {
    @apply opacity-60;
  }
}

.backend-header {
  @apply flex items-center gap-2;
  color: var(--color-slate-200);
}

.backend-name {
  @apply font-medium;
}

.backend-desc {
  @apply text-xs;
  color: var(--color-slate-400);
}

.backend-status {
  @apply mt-1;

  &.ready {
    @apply flex items-center gap-1 text-xs;
    color: var(--color-green-400);
  }
}

.api-key-input {
  @apply flex items-center gap-2;
}

.confidence-slider {
  @apply w-full;
}

.confidence-value {
  @apply text-sm font-medium;
  color: var(--color-blue-400);
}

.error-message {
  @apply flex items-center gap-2 p-3 rounded-lg text-sm;
  background-color: var(--color-red-900/30);
  color: var(--color-red-300);
}
</style>
