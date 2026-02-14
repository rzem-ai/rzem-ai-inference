<template>
  <MenuPanel title="Create" :icon="ImageIcon" :expand="store.chatbotOpen">
    <template #title-button>
      <Button
        class="transition-colors"
        severity="primary"
        :variant="store.chatbotOpen ? 'outlined' : ''"
        :text="!store.chatbotOpen"
        title="AI Prompt Assistant"
        @click="store.toggleChatbot()">
        <WandSparkles :size="16" />
      </Button>
    </template>
    <template #content>
      <!-- Scrollable params -->
      <div class="flex-1 overflow-y-auto px-4 pb-2 flex flex-col gap-4">
        <PromptInput @submit="onSubmit" />
        <AspectRatioStrip />
        <ModelSelect />
        <QualitySliders />
        <StyleSelect />
        <AdvancedSection />

        <!-- Dev controls (Ctrl+Shift+D) -->
        <DevMode v-if="store.devMode" />
      </div>

      <!-- Model status / Error -->
      <div v-if="store.modelStatus" class="px-4 py-1">
        <span class="text-[11px] text-slate-500">{{ store.modelStatus }}</span>
      </div>
      <div v-if="store.error" class="px-4 py-1">
        <span class="text-[11px] text-red-500">{{ store.error }}</span>
      </div>
    </template>
    <template #footer>
      <div v-if="!store.isGenerating" class="flex gap-2">
        <Button class="flex-1 transition-colors flex items-center justify-center gap-2" severity="primary" raised :disabled="!canGenerate" @click="onSubmit">
          <Sparkles :size="16" />
          Generate
        </Button>
        <Button severity="secondary" variant="outlined" title="Batch Generation" :disabled="!store.engineReady" @click="showBatchDialog = true">
          <Layers :size="16" />
        </Button>
      </div>
      <Button v-else class="transition-colors flex items-center justify-center gap-2" severity="danger" fluid raised @click="store.cancelJob()">
        <Square :size="14" />
        Cancel
      </Button>
    </template>
    <template #secondary>
      <ChatbotPanel />
    </template>
  </MenuPanel>

  <BatchDialog v-model:visible="showBatchDialog" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Image as ImageIcon, WandSparkles, Sparkles, Square, Layers } from 'lucide-vue-next';

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
import DevMode from './DevMode.vue';
import BatchDialog from './BatchDialog.vue';
import { Button } from 'primevue';
import MenuPanel from '@/components/MenuPanel.vue';

const { api, isReady } = usePywebview();
const store = useInferenceStore();
const stylesStore = useStylesStore();
const chatStore = useChatStore();

const showBatchDialog = ref(false);

const canGenerate = computed(() => store.engineReady && store.params.prompt.trim() && !store.isGenerating);

function onSubmit() {
  store.submitJob();
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
