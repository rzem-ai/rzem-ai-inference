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
        <div class="flex flex-1">
          <Button class="flex-1 transition-colors flex items-center justify-center gap-2 rounded-r-none!" severity="primary" raised :disabled="!canGenerate" @click="onSubmit">
            <Sparkles :size="16" />
            {{ imageCount > 1 ? `Generate ×${imageCount}` : 'Generate' }}
          </Button>
          <Button class="transition-colors rounded-l-none! border-l border-white/20! px-3!" severity="primary" raised :disabled="!canGenerate" @click="toggleCountPopover">
            <ChevronDown :size="14" />
          </Button>
        </div>
        <Button severity="secondary" variant="outlined" title="Batch Generation" :disabled="!store.engineReady && !store.isCloudBundle" @click="showBatchDialog = true">
          <Layers :size="16" />
        </Button>
        <Button severity="secondary" variant="outlined" title="X/Y Parameter Grid" :disabled="!store.engineReady && !store.isCloudBundle" @click="showGridDialog = true">
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

  <Popover ref="countPopover">
    <div class="flex flex-col py-1">
      <button
        v-for="n in [1, 2, 3, 4]"
        :key="n"
        class="px-4 py-2 text-sm text-left hover:bg-surface-100 transition-colors"
        :class="imageCount === n ? 'font-semibold text-primary-600' : 'text-surface-700'"
        @click="selectCount(n)">
        {{ n }} {{ n === 1 ? 'image' : 'images' }}
      </button>
    </div>
  </Popover>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
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
const imageCount = ref((() => {
  try { return parseInt(localStorage.getItem('imageCount') ?? '1', 10) || 1; }
  catch { return 1; }
})());
const countPopover = ref();

watch(imageCount, (n) => { try { localStorage.setItem('imageCount', String(n)); } catch {} });

const canGenerate = computed(() => (store.engineReady || store.isCloudBundle) && store.params.prompt.trim() && !store.isGenerating);

function toggleCountPopover(event: Event) {
  countPopover.value?.toggle(event);
}

function selectCount(n: number) {
  imageCount.value = n;
  countPopover.value?.hide();
}

function onSubmit() {
  if (imageCount.value > 1) {
    const prompts = Array(imageCount.value).fill(store.params.prompt);
    store.submitBatch(prompts);
  } else {
    store.submitJob();
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

  await store.checkEngineAvailable();
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
  if (store.engineAvailable) {
    await store.startEngine();
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>
