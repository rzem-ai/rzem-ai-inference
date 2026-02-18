<template>
  <div class="flex flex-col gap-4 p-4 overflow-y-auto">
    <Message>
      <template #icon>
        <Info :size="16" />
      </template>
      <div>API keys are stored locally on your machine and never sent to our servers.</div>
    </Message>

    <div class="flex flex-col gap-3">
      <Card v-for="keyDef in keyDefinitions" :key="keyDef.key">
        <template #title>
          <div class="flex items-center gap-2">
            {{ keyDef.label }}
            <Tag :severity="isConfigured(keyDef.key) ? 'success' : 'warn'">
              {{ isConfigured(keyDef.key) ? 'Configured' : 'Not configured' }}
            </Tag>
          </div>
        </template>
        <template #content>
          <p class="text-muted-color mb-3">{{ keyDef.description }}</p>
          <InputGroup>
            <InputGroupAddon class="bg-surface-100"><KeySquare :size="16" /></InputGroupAddon>
            <InputText
              v-model="localKeys[keyDef.key]"
              :type="visibleKeys[keyDef.key] ? 'text' : 'password'"
              :placeholder="keyDef.placeholder"
              class="font-mono" />
            <InputGroupAddon v-if="isConfigured(keyDef.key)">
              <Button severity="secondary" outlined :title="visibleKeys[keyDef.key] ? 'Hide' : 'Reveal'" @click="toggleVisibility(keyDef.key)">
                <EyeOff v-if="visibleKeys[keyDef.key]" :size="16" />
                <Eye v-else :size="16" />
              </Button>
            </InputGroupAddon>
            <InputGroupAddon v-if="isConfigured(keyDef.key)">
              <Button severity="secondary" outlined title="Copy to clipboard" @click="handleCopy(keyDef.key)">
                <Copy :size="16" />
              </Button>
            </InputGroupAddon>
            <InputGroupAddon v-if="isDirty(keyDef.key)">
              <Button severity="success" title="Save" @click="handleSave(keyDef.key)">
                <Check :size="16" />
              </Button>
            </InputGroupAddon>
            <InputGroupAddon v-if="isConfigured(keyDef.key) && !isDirty(keyDef.key)">
              <Button severity="danger" outlined title="Delete" @click="handleDelete(keyDef.key)">
                <Trash2 :size="16" />
              </Button>
            </InputGroupAddon>
          </InputGroup>
        </template>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue';
import { Info, Eye, EyeOff, Copy, Check, Trash2, KeySquare } from 'lucide-vue-next';
import { Message, Card, InputGroup, InputGroupAddon, InputText, Tag, Button } from 'primevue';
import { useInferenceStore } from '@/stores/inference';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();
const inferenceStore = useInferenceStore();

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

const localKeys = reactive<Record<string, string>>({});
const visibleKeys = reactive<Record<string, boolean>>({});

function isConfigured(key: string): boolean {
  const stored = settingsStore.apiKeys[key];
  return stored != null && stored !== '';
}

function isDirty(key: string): boolean {
  const stored = settingsStore.apiKeys[key] ?? '';
  return (localKeys[key] ?? '') !== stored;
}

function toggleVisibility(key: string) {
  visibleKeys[key] = !visibleKeys[key];
}

async function handleSave(key: string) {
  const value = (localKeys[key] ?? '').trim();
  if (value) {
    await settingsStore.saveApiKey(key, value);
    localKeys[key] = value;
  } else {
    await settingsStore.deleteApiKey(key);
    localKeys[key] = '';
  }
  if (key === 'FAL_KEY') {
    await inferenceStore.loadBundles();
  }
}

async function handleDelete(key: string) {
  await settingsStore.deleteApiKey(key);
  localKeys[key] = '';
  visibleKeys[key] = false;
  if (key === 'FAL_KEY') {
    // Deselect current bundle if it's a cloud bundle
    if (inferenceStore.selectedBundle?.source === 'cloud') {
      inferenceStore.selectedBundleId = null;
    }
    await inferenceStore.loadBundles();
  }
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
    localKeys[def.key] = settingsStore.apiKeys[def.key] ?? '';
  }
});
</script>
