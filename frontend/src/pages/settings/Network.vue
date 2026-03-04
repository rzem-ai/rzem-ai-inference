<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-3">Network</div>
      <div class="text-base">Configure network and security settings for corporate environments.</div>

      <Card class="mt-3">
        <template #title>
          <div class="flex items-center gap-2">SSL Certificate Verification</div>
        </template>
        <template #content>
          <div class="flex items-center justify-between">
            <div class="flex-1 pr-4">
              <div class="text-sm font-medium mb-1">Disable SSL verification</div>
              <div class="text-sm text-slate-500">
                Required when running behind a corporate SSL proxy that intercepts HTTPS
                traffic with its own certificate. Disabling this allows all network requests
                (model downloads, API calls, remote servers) to work through the proxy.
              </div>
              <div v-if="localDisabled" class="mt-2 flex items-center gap-2 text-amber-600">
                <ShieldAlert :size="14" />
                <span class="text-sm font-medium">SSL verification is disabled. Connections are not verified.</span>
              </div>
            </div>
            <ToggleSwitch v-model="localDisabled" @change="handleToggle" />
          </div>

          <div class="mt-3 p-3 bg-slate-50 rounded-lg">
            <div class="text-sm text-slate-600">
              <span class="font-medium">What this affects:</span>
              <ul class="mt-1 list-disc list-inside space-y-1 text-slate-500">
                <li>HuggingFace model downloads</li>
                <li>AI assistant API calls (Claude, Perplexity)</li>
                <li>Remote inference server connections</li>
                <li>Style image downloads from URLs</li>
              </ul>
            </div>
          </div>

          <div v-if="showRestartNotice" class="mt-3 flex items-center gap-2 p-3 bg-blue-50 border border-blue-200 rounded-lg text-blue-700">
            <RotateCcw :size="14" />
            <span class="text-sm">Restart the application for this change to fully take effect on existing connections.</span>
          </div>

          <div class="mt-3 text-sm text-slate-400">
            The setting is persisted and applied automatically on startup.
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();
const localDisabled = ref(false);
const initialValue = ref(false);
const showRestartNotice = ref(false);

async function handleToggle() {
  await settingsStore.saveSslVerificationDisabled(localDisabled.value);
  showRestartNotice.value = localDisabled.value !== initialValue.value;
}

onMounted(async () => {
  await settingsStore.loadSslVerificationDisabled();
  localDisabled.value = settingsStore.sslVerificationDisabled;
  initialValue.value = settingsStore.sslVerificationDisabled;
});
</script>
