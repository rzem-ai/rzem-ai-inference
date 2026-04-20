<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :draggable="false"
    :closable="false"
    :header="stepTitle"
    class="w-160">

    <!-- Step 1: Generation Mode Selection -->
    <div v-if="step === 'mode'" class="flex flex-col gap-4  ">
      <p class="text-surface-700 text-lg">Choose how you want to generate images. You can change this later in Settings.</p>

      <div class="flex flex-col gap-3">
        <Button
          v-for="mode in modes"
          :key="mode.id"
          :severity="selectedMode === mode.id ? 'primary' : 'secondary'"
          class="flex items-start gap-4 p-4 text-left "
          :class="selectedMode === mode.id ? '' : ''"
          @click="selectedMode = mode.id">
          <div class="mt-1 shrink-0">
            <component :is="mode.icon" :size="28" class="text-primary-500" :class="selectedMode === mode.id ? 'text-surface-0' : 'text-primary-500'" />
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xl font-semibold" :class="selectedMode === mode.id ? 'text-surface-0' : 'text-surface-700'">{{ mode.label }}</span>
            <span class="text-lg" :class="selectedMode === mode.id ? 'text-surface-100' : 'text-surface-700'">{{ mode.description }}</span>
          </div>
        </Button>
      </div>
    </div>

    <!-- Step 2: API Key Entry -->
    <div v-else-if="step === 'keys'" class="flex flex-col gap-4 py-2">
      <Message>
        <template #icon>
          <Info :size="16" />
        </template>
        <div>API keys are stored locally on your machine and never sent to our servers.</div>
      </Message>

      <div class="flex flex-col gap-3">
        <div v-for="keyDef in requiredKeys" :key="keyDef.key" class="flex flex-col gap-1">
          <label class="text-lg font-medium text-surface-700">{{ keyDef.label }}</label>
          <p class="text-base text-surface-500 mb-1">{{ keyDef.description }}</p>
          <InputGroup>
            <InputGroupAddon class="bg-surface-100"><KeySquare :size="16" /></InputGroupAddon>
            <InputText
              v-model="keyValues[keyDef.key]"
              type="password"
              :placeholder="keyDef.placeholder"
              class="font-mono" />
          </InputGroup>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button
          v-if="step === 'keys'"
          severity="secondary"
          label="Back"
          @click="step = 'mode'" />
        <Button
          v-if="step === 'keys'"
          severity="secondary"
          :label="hasAnyKey ? 'Skip remaining' : 'Skip'"
          @click="handleComplete" />
        <Button
          v-if="step === 'mode'"
          label="Continue"
          :disabled="!selectedMode"
          @click="handleModeSelected" />
        <Button
          v-if="step === 'keys' && hasAnyKey"
          label="Save & Continue"
          @click="handleComplete" />
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue';
import { Monitor, CloudCog, Layers, KeySquare, Info } from 'lucide-vue-next';
import { getApiAsync } from '@/bridge';
import { useInferenceStore } from '@/stores/inference';

const visibleModel = defineModel<boolean>('visible');

const emit = defineEmits<{
  completed: [];
}>();

type GenerationMode = 'local' | 'both' | 'cloud';
type Step = 'mode' | 'keys';

const step = ref<Step>('mode');
const selectedMode = ref<GenerationMode | null>(null);
const saving = ref(false);

const modes = [
  {
    id: 'local' as GenerationMode,
    label: 'Local Generation Only',
    description: 'Run models on your GPU. Requires a compatible NVIDIA GPU with sufficient VRAM.',
    icon: Monitor,
  },
  {
    id: 'both' as GenerationMode,
    label: 'Local & Cloud Generation',
    description: 'Use your local GPU and cloud services like FAL.ai for additional models and faster generation.',
    icon: Layers,
  },
  {
    id: 'cloud' as GenerationMode,
    label: 'Cloud Generation Only',
    description: 'Generate images using cloud APIs. No GPU required — runs on FAL.ai servers.',
    icon: CloudCog,
  },
];

interface KeyDefinition {
  key: string;
  label: string;
  description: string;
  placeholder: string;
}

const allKeyDefinitions: Record<string, KeyDefinition[]> = {
  local: [
    {
      key: 'CLAUDE_API_KEY',
      label: 'Chatbot API Key (Claude or Perplexity)',
      description: 'Used for AI-powered prompt enhancement and image analysis. Enter either a Claude or Perplexity API key.',
      placeholder: 'sk-ant-... or pplx-...',
    },
    {
      key: 'HF_API_KEY',
      label: 'HuggingFace API Key',
      description: 'Used for downloading gated models and accessing private repositories.',
      placeholder: 'hf_...',
    },
  ],
  both: [
    {
      key: 'CLAUDE_API_KEY',
      label: 'Chatbot API Key (Claude or Perplexity)',
      description: 'Used for AI-powered prompt enhancement and image analysis. Enter either a Claude or Perplexity API key.',
      placeholder: 'sk-ant-... or pplx-...',
    },
    {
      key: 'FAL_KEY',
      label: 'FAL.ai API Key',
      description: 'Used for cloud-based image generation via FAL.ai.',
      placeholder: 'fal-...',
    },
    {
      key: 'HF_API_KEY',
      label: 'HuggingFace API Key',
      description: 'Used for downloading gated models and accessing private repositories.',
      placeholder: 'hf_...',
    },
  ],
  cloud: [
    {
      key: 'CLAUDE_API_KEY',
      label: 'Chatbot API Key (Claude or Perplexity)',
      description: 'Used for AI-powered prompt enhancement and image analysis. Enter either a Claude or Perplexity API key.',
      placeholder: 'sk-ant-... or pplx-...',
    },
    {
      key: 'FAL_KEY',
      label: 'FAL.ai API Key',
      description: 'Used for cloud-based image generation via FAL.ai.',
      placeholder: 'fal-...',
    },
  ],
};

const keyValues = reactive<Record<string, string>>({});

const requiredKeys = computed(() => {
  if (!selectedMode.value) return [];
  return allKeyDefinitions[selectedMode.value] ?? [];
});

const hasAnyKey = computed(() => {
  return requiredKeys.value.some(k => (keyValues[k.key] ?? '').trim() !== '');
});

const stepTitle = computed(() => {
  return step.value === 'mode' ? 'Welcome to Inference' : 'API Keys (Optional)';
});

function handleModeSelected() {
  step.value = 'keys';
}

async function handleComplete() {
  if (!selectedMode.value || saving.value) return;
  saving.value = true;

  try {
    const apiKeys: Record<string, string> = {};
    for (const keyDef of requiredKeys.value) {
      const val = (keyValues[keyDef.key] ?? '').trim();
      if (val) {
        apiKeys[keyDef.key] = val;
      }
    }

    const api = await getApiAsync();
    await api.complete_setup({
      generation_mode: selectedMode.value,
      api_keys: Object.keys(apiKeys).length > 0 ? apiKeys : undefined,
    });

    // Reload bundles to reflect hidden cloud models
    const inferenceStore = useInferenceStore();
    await inferenceStore.loadBundles();

    visibleModel.value = false;
    emit('completed');
  } finally {
    saving.value = false;
  }
}
</script>
