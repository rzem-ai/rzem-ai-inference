<template>
  <MenuPanel title="Create" icon="ImageIcon" :expand="store.chatbotOpen">
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
        <StyleSelect />
        <QualitySliders />
        <AdvancedSection />

        <!-- Dev controls (Ctrl+Shift+D) -->
        <DevMode v-if="store.devMode" />
      </div>

      <!-- Error -->
      <div v-if="store.error" class="px-4 py-1">
        <span class="text-sm text-red-500">{{ store.error }}</span>
      </div>
    </template>
    <template #footer>
      <ProgressOverlay />
      <div v-if="!store.isGenerating" class="flex gap-2">
        <Button class="flex-1 transition-colors flex items-center justify-center gap-2" severity="primary" raised :disabled="!canGenerate" @click="onSubmit">
          <Sparkles :size="16" />
          Generate
        </Button>
        <Button severity="secondary" variant="outlined" title="Batch Generation" :disabled="!store.engineReady" @click="showBatchDialog = true">
          <Layers :size="16" />
        </Button>
        <Button severity="secondary" variant="outlined" title="X/Y Parameter Grid" :disabled="!store.engineReady" @click="showGridDialog = true">
          <Grid3x3 :size="16" />
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
  <GridDialog v-model:visible="showGridDialog" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
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
import GridDialog from './GridDialog.vue';
import ProgressOverlay from './ProgressOverlay.vue';
import MenuPanel from '@/components/MenuPanel.vue';

const store = useInferenceStore();
const stylesStore = useStylesStore();
const chatStore = useChatStore();

const showBatchDialog = ref(false);
const showGridDialog = ref(false);

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
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>
