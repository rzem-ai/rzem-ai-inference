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
        <ToggleSwitch v-model="localSettings.autoTagOnGeneration" :disabled="!localSettings.enabled" />
      </div>

      <!-- Preferred Backend -->
      <div class="setting-section">
        <span class="section-title">Preferred Backend</span>

        <div class="backend-options">
          <div
            class="backend-option"
            :class="{
              active: localSettings.preferredBackend === 'local',
              disabled: !autoTagStore.isLocalAvailable,
            }"
            @click="localSettings.preferredBackend = 'local'">
            <div class="backend-header">
              <fa :icon="['fal', 'desktop']" size="lg" />
              <span class="backend-name">Local (Moondream)</span>
            </div>
            <span class="backend-desc">Free, works offline, runs on your GPU</span>
            <div v-if="!autoTagStore.isLocalAvailable" class="backend-status">
              <template v-if="autoTagStore.isDownloading">
                <ProgressBar :value="autoTagStore.downloadProgressPercent" :showValue="true" />
              </template>
              <template v-else>
                <div class="flex flex-col gap-2">
                  <div class="flex items-center gap-2">
                    <Button size="small" severity="secondary" @click.stop="downloadModel">
                      <fa :icon="['fal', 'download']" size="sm" class="mr-2" />
                      Download Model (~1.8GB)
                    </Button>
                    <Button
                      size="small"
                      severity="secondary"
                      text
                      :loading="isClearingLocks"
                      @click.stop="clearLocks"
                      title="Clear stale lock files from failed downloads">
                      <fa :icon="['fal', 'trash-can']" size="sm" />
                    </Button>
                  </div>
                  <span v-if="lockClearMessage" class="text-xs text-gray-400">{{ lockClearMessage }}</span>
                </div>
              </template>
            </div>
            <div v-else class="backend-status ready">
              <fa :icon="['fal', 'check']" size="sm" />
              <span>Ready</span>
            </div>
          </div>

          <div
            class="backend-option"
            :class="{
              active: localSettings.preferredBackend === 'claude',
              disabled: !autoTagStore.isClaudeAvailable && !showApiKeyInput,
            }"
            @click="localSettings.preferredBackend = 'claude'">
            <div class="backend-header">
              <fa :icon="['fal', 'cloud']" size="lg" />
              <span class="backend-name">Claude Vision API</span>
            </div>
            <span class="backend-desc">Higher quality, requires API key, uses credits</span>
            <div class="backend-status">
              <template v-if="autoTagStore.isClaudeAvailable && !showApiKeyInput">
                <div class="flex items-center gap-2">
                  <fa :icon="['fal', 'check']" size="sm" class="text-green-400" />
                  <span>API Key Configured</span>
                  <Button size="small" text severity="secondary" @click.stop="showApiKeyInput = true">
                    <fa :icon="['fal', 'pencil']" size="sm" />
                  </Button>
                </div>
              </template>
              <template v-else>
                <div class="api-key-input" @click.stop>
                  <InputText
                    v-model="localSettings.claudeApiKey"
                    type="password"
                    placeholder="Enter Claude API Key"
                    size="small"
                    :fluid="true" />
                  <Button v-if="showApiKeyInput" size="small" text @click.stop="showApiKeyInput = false">
                    <fa :icon="['fal', 'xmark']" size="sm" />
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
          <span class="confidence-value">{{ Math.round(localSettings.minConfidence * 100) }}%</span>
        </div>
        <Slider v-model="confidenceSlider" :min="0" :max="100" :step="5" class="confidence-slider" />
      </div>

      <!-- Error Message -->
      <div v-if="autoTagStore.lastError" class="error-message">
        <fa :icon="['fal', 'circle-exclamation']" size="sm" />
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
  autoTagOnGeneration: false,
  preferredBackend: 'local',
  minConfidence: 0.6,
  claudeApiKey: undefined,
})

const showApiKeyInput = ref(false)
const isSaving = ref(false)
const isClearingLocks = ref(false)
const lockClearMessage = ref<string | null>(null)

// Slider works with 0-100, settings use 0.0-1.0
const confidenceSlider = computed({
  get: () => Math.round(localSettings.value.minConfidence * 100),
  set: (value: number) => {
    localSettings.value.minConfidence = value / 100
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
  lockClearMessage.value = null
  await autoTagStore.downloadModel()
}

const clearLocks = async () => {
  isClearingLocks.value = true
  lockClearMessage.value = null
  try {
    const result = await autoTagStore.clearLocks()
    lockClearMessage.value = result
    // Refresh model status after clearing locks
    await autoTagStore.checkModelStatus()
  } catch (error) {
    lockClearMessage.value = `Failed: ${error}`
  } finally {
    isClearingLocks.value = false
  }
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
  color: var(--color-gray-200);
}

.setting-desc {
  @apply text-xs;
  color: var(--color-gray-400);
}

.setting-section {
  @apply flex flex-col gap-3;
}

.section-title {
  @apply text-xs font-semibold uppercase tracking-wide;
  color: var(--color-gray-400);
}

.backend-options {
  @apply flex flex-col gap-3;
}

.backend-option {
  @apply flex flex-col gap-2 p-3 rounded-lg cursor-pointer transition-all;
  background-color: var(--color-gray-800);
  border: 2px solid transparent;

  &:hover:not(.disabled) {
    background-color: var(--color-gray-700);
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
  color: var(--color-gray-200);
}

.backend-name {
  @apply font-medium;
}

.backend-desc {
  @apply text-xs;
  color: var(--color-gray-400);
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
