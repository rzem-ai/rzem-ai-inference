<template>
  <div class="flex flex-col gap-6 p-4 overflow-y-auto">
    <div>
      <div class="text-xl font-semibold text-slate-900 mb-1">AI</div>
      <div class="text-base text-slate-500">
        Configure the AI provider, model, and scan button prompts used by the AI Assistant.
      </div>
    </div>

    <!-- Provider selector -->
    <Card>
      <template #title>
        <div class="flex items-center gap-2">
          <Zap :size="16" class="text-amber-500" />
          AI Provider
        </div>
      </template>
      <template #content>
        <p class="text-muted-color mb-3">Choose which AI service powers the assistant.</p>
        <SelectButton
          v-model="localProvider"
          :options="providerOptions"
          option-label="label"
          option-value="value"
          @change="onProviderChange" />
      </template>
    </Card>

    <!-- Claude config -->
    <Card v-if="localProvider === 'claude'">
      <template #title>
        <div class="flex items-center gap-2">
          <Cpu :size="16" class="text-blue-500" />
          Claude Configuration
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">API Key</label>
            <div class="flex gap-2">
              <InputText
                v-model="claudeApiKey"
                type="password"
                class="flex-1"
                placeholder="sk-ant-..." />
              <Button label="Save" severity="primary" @click="saveClaudeKey" :disabled="!claudeApiKey.trim()" />
            </div>
            <p class="text-xs text-slate-400 mt-1">Your Anthropic API key. Stored locally.</p>
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Model</label>
            <Select
              v-model="localClaudeModel"
              :options="claudeModelOptions"
              option-label="label"
              option-value="value"
              class="w-full"
              @change="onClaudeModelChange" />
          </div>
        </div>
      </template>
    </Card>

    <!-- Perplexity config -->
    <Card v-if="localProvider === 'perplexity'">
      <template #title>
        <div class="flex items-center gap-2">
          <Globe :size="16" class="text-violet-500" />
          Perplexity Configuration
        </div>
      </template>
      <template #content>
        <div class="flex flex-col gap-4">
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">API Key</label>
            <div class="flex gap-2">
              <InputText
                v-model="perplexityApiKey"
                type="password"
                class="flex-1"
                placeholder="pplx-..." />
              <Button label="Save" severity="primary" @click="savePerplexityKey" :disabled="!perplexityApiKey.trim()" />
            </div>
            <p class="text-xs text-slate-400 mt-1">Your Perplexity API key. Stored locally.</p>
          </div>
          <div>
            <label class="text-sm font-medium text-slate-700 mb-1 block">Model</label>
            <Select
              v-model="localPerplexityModel"
              :options="perplexityModelOptions"
              option-label="label"
              option-value="value"
              class="w-full"
              @change="onPerplexityModelChange" />
          </div>
        </div>
      </template>
    </Card>

    <Message severity="secondary" :closable="false">
      <template #icon><Info :size="16" /></template>
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
import { useChatStore } from '@/stores/chat';
import { Box, Cpu, Globe, Info, Layers, Paintbrush, Zap } from 'lucide-vue-next';

const settingsStore = useSettingsStore();
const chatStore = useChatStore();

const providerOptions = [
  { label: 'Claude', value: 'claude' },
  { label: 'Perplexity', value: 'perplexity' },
];

const claudeModelOptions = [
  { label: 'Claude Haiku 4.6 — Fast, low cost', value: 'claude-haiku-4-5-20251001' },
  { label: 'Claude Sonnet 4.6 — Balanced (default)', value: 'claude-sonnet-4-6' },
  { label: 'Claude Opus 4.6 — Most capable', value: 'claude-opus-4-6' },
];

const perplexityModelOptions = [
  { label: 'Sonar — Fast, web search', value: 'perplexity/sonar' },
  { label: 'GPT-5.2 — Frontier', value: 'openai/gpt-5.2' },
  { label: 'GPT-5.1 — Mid-tier', value: 'openai/gpt-5.1' },
  { label: 'GPT-5 Mini — Budget', value: 'openai/gpt-5-mini' },
  { label: 'Claude Sonnet 4.6 — Balanced (default)', value: 'anthropic/claude-sonnet-4-6' },
  { label: 'Claude Haiku 4.5 — Fast', value: 'anthropic/claude-haiku-4-5' },
  { label: 'Gemini 2.5 Pro', value: 'google/gemini-2.5-pro' },
  { label: 'Gemini 2.5 Flash — Budget', value: 'google/gemini-2.5-flash' },
];

const localProvider = ref('claude');
const claudeApiKey = ref('');
const perplexityApiKey = ref('');
const localClaudeModel = ref('claude-sonnet-4-6');
const localPerplexityModel = ref('anthropic/claude-sonnet-4-6');

async function onProviderChange() {
  await settingsStore.saveAiProvider(localProvider.value);
  await chatStore.checkConfigured();
}

async function saveClaudeKey() {
  if (!claudeApiKey.value.trim()) return;
  await chatStore.setApiKey(claudeApiKey.value.trim(), 'claude');
  claudeApiKey.value = '';
  await chatStore.checkConfigured();
}

async function savePerplexityKey() {
  if (!perplexityApiKey.value.trim()) return;
  await chatStore.setApiKey(perplexityApiKey.value.trim(), 'perplexity');
  perplexityApiKey.value = '';
  await chatStore.checkConfigured();
}

async function onClaudeModelChange() {
  await settingsStore.saveClaudeModel(localClaudeModel.value);
}

async function onPerplexityModelChange() {
  await settingsStore.savePerplexityModel(localPerplexityModel.value);
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
  await settingsStore.loadAiProvider();
  localProvider.value = settingsStore.aiProvider;
  await settingsStore.loadClaudeModel();
  localClaudeModel.value = settingsStore.claudeModel;
  await settingsStore.loadPerplexityModel();
  localPerplexityModel.value = settingsStore.perplexityModel;
  await settingsStore.loadAiPrompts();
  syncFromStore();
});
</script>
