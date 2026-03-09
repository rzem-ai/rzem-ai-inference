<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :draggable="false"
    :closable="false"
    :header="step === 'mode' ? 'Welcome to Rzem AI' : 'Configure API Keys'"
    class="w-175 max-w-[95vw]">

    <!-- Step 1: Generation Mode -->
    <div v-if="step === 'mode'" class="flex flex-col gap-4">
      <p class="text-sm text-surface-400">How would you like to generate images?</p>
      <div class="grid grid-cols-3 gap-3">
        <button
          v-for="option in modeOptions"
          :key="option.value"
          class="flex flex-col items-center gap-3 p-6 rounded-lg border-2 cursor-pointer transition-all text-center"
          :class="selectedMode === option.value
            ? 'border-primary bg-primary/10'
            : 'border-surface-500 hover:border-surface-300 bg-surface-800'"
          @click="selectMode(option.value)">
          <component :is="option.icon" :size="36" class="text-surface-300" />
          <span class="text-base font-semibold">{{ option.label }}</span>
          <span class="text-xs text-surface-400">{{ option.description }}</span>
        </button>
      </div>
    </div>

    <!-- Step 2: API Keys -->
    <div v-else class="flex flex-col gap-4">
      <p class="text-sm text-surface-400">
        Provide your API keys below. You can also configure these later in Settings.
      </p>

      <!-- Chatbot provider + key -->
      <Card>
        <template #title>
          <div class="text-sm">Chatbot API Key</div>
        </template>
        <template #content>
          <p class="text-xs text-surface-400 mb-3">Used for AI-powered prompt enhancement and image analysis.</p>
          <div class="flex flex-col gap-2">
            <SelectButton
              v-model="chatProvider"
              :options="chatProviderOptions"
              option-label="label"
              option-value="value"
              :allow-empty="false" />
            <InputGroup>
              <InputGroupAddon class="bg-surface-100"><KeySquare :size="16" /></InputGroupAddon>
              <InputText
                v-model="keys.chatbot"
                type="password"
                :placeholder="chatProvider === 'claude' ? 'sk-ant-...' : 'pplx-...'"
                class="font-mono" />
            </InputGroup>
          </div>
        </template>
      </Card>

      <!-- FAL.ai key (cloud modes) -->
      <Card v-if="selectedMode !== 'local'">
        <template #title>
          <div class="text-sm">fal.ai API Key</div>
        </template>
        <template #content>
          <p class="text-xs text-surface-400 mb-3">Used for cloud-based image generation.</p>
          <InputGroup>
            <InputGroupAddon class="bg-surface-100"><KeySquare :size="16" /></InputGroupAddon>
            <InputText
              v-model="keys.fal"
              type="password"
              placeholder="fal-..."
              class="font-mono" />
          </InputGroup>
        </template>
      </Card>

      <!-- HuggingFace key (local modes) -->
      <Card v-if="selectedMode !== 'cloud'">
        <template #title>
          <div class="text-sm">HuggingFace API Key</div>
        </template>
        <template #content>
          <p class="text-xs text-surface-400 mb-3">Used for downloading gated models and accessing private repositories.</p>
          <InputGroup>
            <InputGroupAddon class="bg-surface-100"><KeySquare :size="16" /></InputGroupAddon>
            <InputText
              v-model="keys.hf"
              type="password"
              placeholder="hf_..."
              class="font-mono" />
          </InputGroup>
        </template>
      </Card>
    </div>

    <template #footer>
      <div v-if="step === 'mode'" class="flex justify-end">
        <Button :disabled="!selectedMode" @click="goToKeys">Continue</Button>
      </div>
      <div v-else class="flex justify-between">
        <Button variant="text" @click="step = 'mode'">Back</Button>
        <div class="flex gap-2">
          <Button variant="outlined" @click="handleComplete">Skip</Button>
          <Button :disabled="!hasAnyKey" @click="handleComplete">Complete Setup</Button>
        </div>
      </div>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { Cpu, Layers, Cloud } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings';
import { useInferenceStore } from '@/stores/inference';
import { useModelsStore } from '@/stores/models';

const visibleModel = defineModel<boolean>({ default: false });

const settingsStore = useSettingsStore();
const inferenceStore = useInferenceStore();
const modelsStore = useModelsStore();

const step = ref<'mode' | 'keys'>('mode');
const selectedMode = ref<string | null>(null);
const chatProvider = ref('claude');

const keys = reactive({
  chatbot: '',
  fal: '',
  hf: '',
});

const modeOptions = [
  {
    value: 'local',
    label: 'Local Generation Only',
    description: 'Use your GPU to generate images locally',
    icon: Cpu,
  },
  {
    value: 'both',
    label: 'Local & Cloud Generation',
    description: 'Use both your GPU and cloud APIs',
    icon: Layers,
  },
  {
    value: 'cloud',
    label: 'Cloud Generation Only',
    description: 'Use cloud APIs (FAL.ai) for generation',
    icon: Cloud,
  },
];

const chatProviderOptions = [
  { label: 'Claude', value: 'claude' },
  { label: 'Perplexity', value: 'perplexity' },
];

const hasAnyKey = computed(() => {
  return keys.chatbot.trim() !== '' || keys.fal.trim() !== '' || keys.hf.trim() !== '';
});

function selectMode(mode: string) {
  selectedMode.value = mode;
}

function goToKeys() {
  if (selectedMode.value) {
    step.value = 'keys';
  }
}

async function handleComplete() {
  if (!selectedMode.value) return;

  const apiKeys: Record<string, string> = {};

  if (keys.chatbot.trim()) {
    const keyName = chatProvider.value === 'claude' ? 'CLAUDE_API_KEY' : 'PERPLEXITY_API_KEY';
    apiKeys[keyName] = keys.chatbot;
  }
  if (keys.fal.trim() && selectedMode.value !== 'local') {
    apiKeys['FAL_KEY'] = keys.fal;
  }
  if (keys.hf.trim() && selectedMode.value !== 'cloud') {
    apiKeys['HF_API_KEY'] = keys.hf;
  }

  // Save AI provider preference if chatbot key was provided
  if (keys.chatbot.trim()) {
    apiKeys['AI_PROVIDER'] = chatProvider.value;
  }

  await settingsStore.completeOnboarding(selectedMode.value, apiKeys);

  // Reload bundles to reflect the new generation mode filter
  await Promise.all([
    inferenceStore.loadBundles(),
    modelsStore.loadBundles(),
  ]);

  visibleModel.value = false;
}
</script>
