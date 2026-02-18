<template>
  <div class="flex flex-col gap-6 p-4 overflow-y-auto">
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-1">AI</div>
      <div class="text-base text-slate-500">
        Customize the prompts used by the AI Assistant's scan buttons. Each prompt has a display label shown in the chat window and the full prompt text sent to the AI.
      </div>
    </div>

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
import { reactive, onMounted } from 'vue';
import { Paintbrush, Layers, Box, Info } from 'lucide-vue-next';
import { Card, InputText, Message, Textarea } from 'primevue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

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
  await settingsStore.loadAiPrompts();
  syncFromStore();
});
</script>
