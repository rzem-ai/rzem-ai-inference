<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <!-- Info banner -->
    <Message>
      <template #icon>
        <Info :size="16" />
      </template>
      <div> API keys are stored locally on your machine and never sent to our servers. </div>
    </Message>

    <!-- API Key cards -->
    <div class="flex flex-col gap-3">
      <Card v-for="keyDef in keyDefinitions" :key="keyDef.key" class="bg-white rounded-xl border border-slate-200 p-4">
        <template #title>
          {{ keyDef.label }}
          <Tag :severity="isConfigured(keyDef.key) ? 'success' : 'warn'">{{ isConfigured(keyDef.key) ? 'Configured' : 'Not configured' }}</Tag>
        </template>
        <template #content>
          <div class="flex flex-col gap-2">
            <div class="text-base text-slate-500">{{ keyDef.description }}</div>

            <!-- Edit mode -->
            <InputGroup v-if="editingKey === keyDef.key">
              <InputGroupAddon>A</InputGroupAddon>
              <InputText />
              <InputGroupAddon>C</InputGroupAddon>
              <InputGroupAddon>D</InputGroupAddon>
              <InputGroupAddon>E</InputGroupAddon>
            </InputGroup>

            <!-- Display mode -->
            <InputGroup v-else>
              <InputGroupAddon>A</InputGroupAddon>
              <InputText />
              <InputGroupAddon>C</InputGroupAddon>
              <InputGroupAddon>D</InputGroupAddon>
              <InputGroupAddon>E</InputGroupAddon>
            </InputGroup>

            <!-- Edit mode -->
            <div v-if="editingKey === keyDef.key" class="flex gap-2">
              <input
                v-model="editValue"
                type="text"
                :placeholder="keyDef.placeholder"
                class="flex-1 px-3 py-1.5 text-sm border border-slate-200 rounded-lg bg-slate-50 outline-none focus:border-blue-300 focus:ring-1 focus:ring-blue-200 font-mono" />
              <button
                class="px-3 py-1.5 text-xs font-medium rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                @click="handleSave(keyDef.key)">
                Save
              </button>
              <button
                class="px-3 py-1.5 text-xs font-medium rounded-lg border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors"
                @click="cancelEdit">
                Cancel
              </button>
            </div>

            <div v-else class="flex items-center gap-2">
              <div class="flex-1 px-3 py-1.5 text-sm bg-slate-50 rounded-lg border border-slate-100 font-mono text-slate-600 truncate">
                <template v-if="isConfigured(keyDef.key)">
                  {{ visibleKeys[keyDef.key] ? settingsStore.apiKeys[keyDef.key] : maskedValue(keyDef.key) }}
                </template>
                <template v-else>
                  <span class="text-slate-400">Not set</span>
                </template>
              </div>
              <button
                v-if="isConfigured(keyDef.key)"
                class="p-1.5 text-slate-400 hover:text-slate-600 transition-colors"
                :title="visibleKeys[keyDef.key] ? 'Hide' : 'Show'"
                @click="toggleVisibility(keyDef.key)">
                <EyeOff v-if="visibleKeys[keyDef.key]" :size="14" />
                <Eye v-else :size="14" />
              </button>
              <button
                v-if="isConfigured(keyDef.key)"
                class="p-1.5 text-slate-400 hover:text-slate-600 transition-colors"
                title="Copy"
                @click="handleCopy(keyDef.key)">
                <Copy :size="14" />
              </button>
              <button
                class="px-3 py-1.5 text-xs font-medium rounded-lg border border-slate-200 text-slate-700 hover:bg-slate-50 transition-colors"
                @click="startEdit(keyDef.key)">
                {{ isConfigured(keyDef.key) ? 'Edit' : 'Add' }}
              </button>
            </div>
          </div>
        </template>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { Info, Eye, EyeOff, Copy } from 'lucide-vue-next';
import Message from 'primevue/message';
import { useSettingsStore } from '@/stores/settings';
import { Card, InputGroup, InputGroupAddon, InputText, Tag } from 'primevue';

const settingsStore = useSettingsStore();

const keyDefinitions = [
  {
    key: 'CLAUDE_API_KEY',
    label: 'Claude API Key',
    description: 'Used for AI-powered prompt enhancement and image analysis.',
    placeholder: 'sk-ant-...',
  },
  {
    key: 'FAL_KEY',
    label: 'fal.ai API Key',
    description: 'Used for cloud-based image generation when local GPU is unavailable.',
    placeholder: 'fal-...',
  },
  {
    key: 'HF_API_KEY',
    label: 'HuggingFace API Key',
    description: 'Used for downloading gated models and accessing private repositories.',
    placeholder: 'hf_...',
  },
];

const editingKey = ref<string | null>(null);
const editValue = ref('');
const visibleKeys = reactive<Record<string, boolean>>({});

function isConfigured(key: string): boolean {
  const val = settingsStore.apiKeys[key];
  return val != null && val !== '';
}

function maskedValue(key: string): string {
  const val = settingsStore.apiKeys[key];
  if (!val) return '';
  if (val.length <= 8) return '\u2022'.repeat(val.length);
  return val.slice(0, 4) + '\u2022'.repeat(Math.min(val.length - 8, 20)) + val.slice(-4);
}

function toggleVisibility(key: string) {
  visibleKeys[key] = !visibleKeys[key];
}

function startEdit(key: string) {
  editingKey.value = key;
  editValue.value = settingsStore.apiKeys[key] ?? '';
}

function cancelEdit() {
  editingKey.value = null;
  editValue.value = '';
}

async function handleSave(key: string) {
  if (editValue.value.trim()) {
    await settingsStore.saveApiKey(key, editValue.value.trim());
  } else {
    await settingsStore.deleteApiKey(key);
  }
  editingKey.value = null;
  editValue.value = '';
}

async function handleCopy(key: string) {
  const val = settingsStore.apiKeys[key];
  if (val) {
    await navigator.clipboard.writeText(val);
  }
}

onMounted(async () => {
  for (const def of keyDefinitions) {
    await settingsStore.loadApiKey(def.key);
  }
});
</script>
