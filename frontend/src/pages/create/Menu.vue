<template>
  <div class="h-full p-2">
    <div class="h-full bg-white rounded-xl shadow-sm flex overflow-hidden transition-[width] duration-300 ease-in-out">
      <!-- Left column: params (always visible) -->
      <div class="w-120 flex flex-col h-full">
        <!-- Header -->
        <div class="flex items-center justify-between px-4 pt-4 pb-2">
          <div class="text-xl min-h-6 font-semibold text-slate-800">Create</div>
          <button
            class="p-1.5 rounded-lg transition-colors"
            :class="store.chatbotOpen ? 'bg-blue-500 text-white' : 'hover:bg-slate-100 text-slate-500'"
            title="AI Prompt Assistant"
            @click="store.toggleChatbot()">
            <WandSparkles :size="16" />
          </button>
        </div>

        <!-- Scrollable params -->
        <div class="flex-1 overflow-y-auto px-4 pb-2 flex flex-col gap-4">
          <PromptInput @submit="onSubmit" />
          <AspectRatioStrip />
          <ModelSelect />
          <QualitySliders />
          <StyleSelect />
          <AdvancedSection />

          <!-- Dev controls (Ctrl+Shift+D) -->
          <template v-if="store.devMode">
            <div class="border-t border-slate-200 pt-3 flex flex-col gap-3">
              <div class="flex items-center gap-1">
                <Terminal :size="12" class="text-orange-500" />
                <span class="text-[10px] font-bold text-orange-500 uppercase">Dev Mode</span>
              </div>

              <!-- Engine controls -->
              <div class="flex gap-1">
                <button
                  v-if="!store.engineReady"
                  class="flex-1 text-[11px] py-1.5 rounded bg-green-500 text-white hover:bg-green-600 transition-colors disabled:opacity-50"
                  :disabled="store.engineStarting"
                  @click="store.startEngine()">
                  {{ store.engineStarting ? 'Starting...' : 'Start Engine' }}
                </button>
                <button v-else class="flex-1 text-[11px] py-1.5 rounded bg-red-500 text-white hover:bg-red-600 transition-colors" @click="store.stopEngine()">
                  Stop Engine
                </button>
                <div
                  class="text-[10px] px-2 py-1.5 rounded font-medium"
                  :class="store.engineReady ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'">
                  {{ store.engineReady ? 'Ready' : 'Stopped' }}
                </div>
              </div>

              <!-- Model path overrides -->
              <div class="flex flex-col gap-1.5">
                <label class="text-[10px] font-medium text-slate-500">Transformer</label>
                <input v-model="store.params.transformer_model" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">VAE</label>
                <input v-model="store.params.vae_model" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">CLIP Tokenizer</label>
                <input v-model="store.params.clip_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">CLIP Encoder</label>
                <input v-model="store.params.clip_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">T5 Tokenizer</label>
                <input v-model="store.params.t5_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">T5 Encoder</label>
                <input v-model="store.params.t5_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">Qwen3 Tokenizer</label>
                <input v-model="store.params.qwen3_tokenizer" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
                <label class="text-[10px] font-medium text-slate-500">Qwen3 Encoder</label>
                <input v-model="store.params.qwen3_encoder" class="text-[10px] bg-slate-50 rounded px-2 py-1 border border-slate-200" />
              </div>

              <!-- Debug events -->
              <div class="flex flex-col gap-1">
                <label class="text-[10px] font-medium text-slate-500">Debug Events</label>
                <Select
                  v-model="debugEvent"
                  :options="debugEventOptions"
                  option-label="label"
                  option-value="value"
                  placeholder="Inject event..."
                  size="small"
                  fluid
                  @change="onDebugEvent" />
              </div>
            </div>
          </template>
        </div>

        <!-- Model status / Error -->
        <div v-if="store.modelStatus" class="px-4 py-1">
          <span class="text-[11px] text-slate-500">{{ store.modelStatus }}</span>
        </div>
        <div v-if="store.error" class="px-4 py-1">
          <span class="text-[11px] text-red-500">{{ store.error }}</span>
        </div>

        <!-- Generate button (pinned at bottom) -->
        <div class="px-4 py-3 border-t border-slate-100">
          <Button
            v-if="!store.isGenerating"
            class="transition-colors flex items-center justify-center gap-2"
            severity="primary"
            fluid
            raised
            :disabled="!canGenerate"
            @click="onSubmit">
            <Sparkles :size="16" />
            Generate
          </Button>
          <Button v-else class="transition-colors flex items-center justify-center gap-2" severity="danger" fluid raised @click="store.cancelJob()">
            <Square :size="14" />
            Cancel
          </Button>
        </div>
      </div>

      <!-- Right column: chatbot (conditional) -->
      <div class="flex-1 transition-[width] duration-300 ease-in-out" :class="store.chatbotOpen ? 'w-120 border-l border-slate-100 ' : 'w-0'">
        <ChatbotPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { WandSparkles, Sparkles, Square, Terminal } from 'lucide-vue-next';
import Select from 'primevue/select';
import { usePywebview } from '@/composables/usePywebview';
import { useInferenceStore } from '@/stores/inference';
import { useStylesStore } from '@/stores/styles';
import { useChatStore } from '@/stores/chat';
import PromptInput from './PromptInput.vue';
import AspectRatioStrip from './AspectRatioStrip.vue';
import ModelSelect from './ModelSelect.vue';
import StyleSelect from './StyleSelect.vue';
import QualitySliders from './QualitySliders.vue';
import AdvancedSection from './AdvancedSection.vue';
import ChatbotPanel from './ChatbotPanel.vue';
import { Button } from 'primevue';

const { api, isReady } = usePywebview();
const store = useInferenceStore();
const stylesStore = useStylesStore();
const chatStore = useChatStore();

const debugEvent = ref<string | null>(null);

const canGenerate = computed(() => store.engineReady && store.params.prompt.trim() && !store.isGenerating);

const debugEventOptions = [
  { label: 'default (reset)', value: 'default' },
  { label: 'model_loading', value: 'model_loading' },
  { label: 'model_loaded', value: 'model_loaded' },
  { label: 'model_unloaded', value: 'model_unloaded' },
  { label: 'job_queued', value: 'job_queued' },
  { label: 'job_started', value: 'job_started' },
  { label: 'job_progress', value: 'job_progress' },
  { label: 'job_progress_0', value: 'job_progress_0' },
  { label: 'job_progress_5', value: 'job_progress_5' },
  { label: 'job_progress_10', value: 'job_progress_10' },
  { label: 'job_progress_15', value: 'job_progress_15' },
  { label: 'job_progress_20', value: 'job_progress_20' },
  { label: 'job_progress_25', value: 'job_progress_25' },
  { label: 'job_progress_28', value: 'job_progress_28' },
  { label: 'job_completed', value: 'job_completed' },
  { label: 'job_failed', value: 'job_failed' },
  { label: 'job_cancelled', value: 'job_cancelled' },
];

function onSubmit() {
  store.submitJob();
}

function onDebugEvent(e: any) {
  if (e.value) {
    store.injectDebugEvent(e.value);
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    e.preventDefault();
    store.toggleDevMode();
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown);

  const check = setInterval(async () => {
    if (isReady.value) {
      clearInterval(check);
      store.setApi(api.value);
      stylesStore.setApi(api.value);
      chatStore.setApi(api.value);
      await store.loadGpuInfo();
      await store.loadBundles();
      stylesStore.loadStyles();
      chatStore.checkConfigured();
      chatStore.loadConversations().then(() => {
        if (!chatStore.conversations.length) {
          chatStore.createConversation();
        } else {
          chatStore.switchConversation(chatStore.conversations[0].id);
        }
      });
      if (store.bundles.length && !store.selectedBundleId) {
        store.applyBundle(store.bundles[0]);
      }
      await store.startEngine();
    }
  }, 50);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>
