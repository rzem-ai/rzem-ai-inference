<template>
  <div class="flex flex-col gap-6 p-4 overflow-y-auto">
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-1">AI</div>
      <div class="text-base text-slate-500">
        Customize the prompts used by the AI Assistant's scan buttons. Each prompt has a display label shown in the chat window and the full prompt text sent to the AI.
      </div>
    </div>

    <Card>
      <template #title>
        <div class="flex items-center gap-2">
          <Cpu :size="16" class="text-blue-500" />
          Claude Model
        </div>
      </template>
      <template #content>
        <p class="text-muted-color mb-3">Select the Claude model used by the AI assistant for chat, prompt enhancement, and image analysis.</p>
        <Select
          v-model="localModel"
          :options="modelOptions"
          option-label="label"
          option-value="value"
          class="w-full"
          @change="saveModel" />
      </template>
    </Card>

    <Message severity="secondary" :closable="false">
      <template #messageicon><Info :size="16" /></template>
      <div class="text-sm leading-relaxed">
        <span class="font-semibold">How prompts work:</span> Each prompt is sent alongside a reference image. The prompt should tell the AI
        what to analyze in the image, then instruct it to update your generation prompt. The AI has two tools it can call:
        <span class="font-semibold">update prompt</span> and <span class="font-semibold">update generation settings</span>
        (dimensions, steps, cfg scale, seed). Without an explicit instruction like "Then update my prompt to...",
        the AI will only describe the image without modifying anything.
      </div>
    </Message>

    <Card v-for="entry in promptEntries" :key="entry.key">
      <template #title>
        <div class="flex items-center gap-2">
          <component :is="entry.icon" :size="16" :class="entry.iconClass" />
          {{ entry.label }}
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Display Text</label>
            <InputText
              v-model="local[entry.key].displayText"
              class="w-full"
              :placeholder="`Text shown in chat for ${entry.label}`"
              @change="saveDisplayText(entry.key)" />
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Prompt</label>
            <Textarea
              v-model="local[entry.key].prompt"
              class="w-full"
              rows="4"
              auto-resize
              :placeholder="`Full prompt sent to the AI for ${entry.label}`"
              @change="savePrompt(entry.key)" />
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { Box, Layers, Paintbrush } from 'lucide-vue-next';

const settingsStore = useSettingsStore();

const modelOptions = [
  { label: 'Claude Haiku 4.6 — Fast, low cost', value: 'claude-haiku-4-5-20251001' },
  { label: 'Claude Sonnet 4.6 — Balanced (default)', value: 'claude-sonnet-4-6' },
  { label: 'Claude Opus 4.6 — Most capable', value: 'claude-opus-4-6' },
];

const localModel = ref('claude-sonnet-4-6');

async function saveModel() {
  await settingsStore.saveClaudeModel(localModel.value);
}

const promptEntries = [
  { key: 'style', label: 'Style', icon: Paintbrush, iconClass: 'text-purple-500' },
  { key: 'both', label: 'Style + Subject', icon: Layers, iconClass: 'text-blue-500' },
  { key: 'subject', label: 'Subject', icon: Box, iconClass: 'text-green-500' },
];

const local = reactive({
  style: { prompt: '', displayText: '' },
  both: { prompt: '', displayText: '' },
  subject: { prompt: '', displayText: '' },
} as Record<string, { prompt: string; displayText: string }>);

function syncFromStore() {
  for (const key of ['style', 'both', 'subject']) {
    local[key].prompt = settingsStore.aiPrompts[key].prompt;
    local[key].displayText = settingsStore.aiPrompts[key].displayText;
  }
}

async function savePrompt(key: string) {
  await settingsStore.saveAiPrompt(key, local[key].prompt);
}

async function saveDisplayText(key: string) {
  await settingsStore.saveAiDisplayText(key, local[key].displayText);
}

onMounted(async () => {
  await settingsStore.loadClaudeModel();
  localModel.value = settingsStore.claudeModel;
  await settingsStore.loadAiPrompts();
  syncFromStore();
});
</script>
