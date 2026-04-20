<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="h-16 px-4 w-full content-end">
      <div class="flex items-center justify-between w-full border-b border-surface-300 pb-2">
        <div class="flex items-center gap-x-2">
          <Bot :size="14" class="text-blue-500" />
          <span class="text-base font-semibold text-slate-700">AI Assistant</span>
        </div>
        <Button class="transition-colors" title="New Chat" text @click="onNewChat">
          <MessageCirclePlus :size="16" />
        </Button>
      </div>
    </div>

    <!-- Not configured state -->
    <div v-if="!chatStore.isConfigured" class="flex-1 flex flex-col items-center justify-center px-6 gap-3">
      <KeyRound :size="24" class="text-slate-300" />
      <p class="text-xs text-slate-500 text-center">Enter your {{ providerLabel }} API key to enable the AI assistant.</p>
      <div class="flex gap-1 w-full">
        <input
          v-model="apiKeyInput"
          type="password"
          :placeholder="providerPlaceholder"
          class="flex-1 text-xs bg-slate-100 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-blue-300"
          @keydown.enter="onSetApiKey" />
        <button
          class="px-3 py-2 bg-blue-500 text-white text-xs rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
          :disabled="!apiKeyInput.trim()"
          @click="onSetApiKey">
          Save
        </button>
      </div>
    </div>

    <!-- Configured state -->
    <template v-else>
      <!-- Scan buttons -->
      <div class="flex gap-1 px-3 py-2 border-b border-slate-100">
        <button
          class="flex-1 py-1 rounded border border-dashed border-purple-300 bg-purple-50 text-center hover:bg-purple-100 hover:border-purple-500 transition-colors"
          @click="openPicker('style')">
          <span class="text-xs font-medium text-purple-600">Style</span>
        </button>
        <button
          class="flex-1 py-1 rounded border border-dashed border-blue-300 bg-blue-50 text-center hover:bg-blue-100 hover:border-blue-500 transition-colors"
          @click="openPicker('both')">
          <span class="text-xs font-medium text-blue-600">Style+Subject</span>
        </button>
        <button
          class="flex-1 py-1 rounded border border-dashed border-green-300 bg-green-50 text-center hover:bg-green-100 hover:border-green-500 transition-colors"
          @click="openPicker('subject')">
          <span class="text-xs font-medium text-green-600">Subject</span>
        </button>
      </div>

      <!-- Messages area -->
      <div ref="messagesContainer" class="flex-1 overflow-y-auto px-3 py-2 flex flex-col gap-3">
        <!-- Empty state with suggestions -->
        <div v-if="!chatStore.messages.length && !chatStore.isStreaming" class="flex-1 flex flex-col items-center justify-center gap-3">
          <Bot :size="44" class="text-slate-300" />
          <p class="text-base text-slate-400 text-center">Ask me to help craft prompts, adjust settings, or analyze images.</p>
          <div class="flex gap-1 flex-wrap justify-center">
            <Button v-for="chip in suggestionChips" :key="chip" @click="onSendChip(chip)" severity="info" size="small" class="text-sm"> {{ chip }} </Button>
          </div>
        </div>

        <!-- Message list -->
        <template v-for="msg in chatStore.messages" :key="msg.id">
          <!-- User message -->
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div class="max-w-[85%]">
              <!-- Image thumbnails -->
              <div v-if="msg.image_paths" class="flex gap-1 mb-1 justify-end">
                <ChatImageThumb v-for="path in parsePaths(msg.image_paths)" :key="path" :path="path" />
              </div>
              <div class="bg-blue-500 rounded-lg px-3 py-2 text-base text-white leading-relaxed">
                {{ msg.display_text || msg.content }}
              </div>
            </div>
          </div>

          <!-- Assistant message -->
          <div v-else-if="msg.role === 'assistant'" class="flex gap-2">
            <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-1">
              <Bot :size="14" class="text-white" />
            </div>
            <div class="flex-1 min-w-0">
              <!-- Tool use badges -->
              <div v-if="msg.tool_calls" class="flex gap-1 mb-1 flex-wrap">
                <Tag v-for="(tool, i) in parseToolCalls(msg.tool_calls)" :key="i" :value="formatToolName(tool.name, tool.input)" />
              </div>
              <div
                v-if="msg.content"
                class="bg-slate-50 rounded-lg px-3 py-2 text-base text-slate-700 leading-relaxed prose-chat"
                v-html="renderMarkdown(msg.content)" />
            </div>
          </div>

          <!-- Error message -->
          <div v-else-if="msg.role === 'error'" class="flex gap-2">
            <div class="w-6 h-6 rounded-full bg-red-500 flex items-center justify-center shrink-0 mt-1">
              <TriangleAlert :size="14" class="text-white" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="bg-red-50 border border-red-200 rounded-lg px-3 py-2 text-sm text-red-700 leading-relaxed">
                {{ msg.content }}
              </div>
            </div>
          </div>
        </template>

        <!-- Streaming bubble -->
        <div v-if="chatStore.isStreaming" class="flex gap-2">
          <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-1">
            <Bot :size="14" class="text-white" />
          </div>
          <div v-if="chatStore.streamingText" class="bg-slate-50 rounded-lg px-3 py-2 text-base text-slate-700 leading-relaxed prose-chat">
            <span v-html="renderMarkdown(chatStore.streamingText)" />
            <span class="inline-block w-3 h-3 bg-blue-500 ml-1 animate-pulse rounded-sm"></span>
          </div>
          <div v-else class="bg-slate-50 rounded-lg px-3 py-2">
            <div class="flex gap-1 items-center">
              <div class="w-1 h-1 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 0ms" />
              <div class="w-1 h-1 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 150ms" />
              <div class="w-1 h-1 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 300ms" />
            </div>
          </div>
        </div>
      </div>

      <!-- Pending images strip -->
      <div v-if="chatStore.pendingImagePaths.length" class="px-3 py-1 border-t border-slate-100">
        <div class="flex gap-1">
          <div
            v-for="(path, i) in chatStore.pendingImagePaths"
            :key="path"
            class="relative w-10 h-10 rounded bg-slate-100 flex items-center justify-center group">
            <ImageIcon :size="14" class="text-slate-400" />
            <button
              class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
              @click="chatStore.removePendingImage(i)">
              <X :size="10" />
            </button>
          </div>
        </div>
      </div>

      <!-- Chat input -->
      <div class="px-4 py-3 border-t border-slate-200 min-h-16 bg-surface-100">
        <InputGroup>
          <InputText v-model="chatInput" placeholder="Ask for prompt help..." @keydown.enter="onSend" />
          <InputGroupAddon
            @click="onSend"
            :disabled="!isSendMessageEnabled"
            class="text-base bg-blue-400 text-surface-50"
            :class="isSendMessageEnabled ? 'cursor-pointer hover:bg-blue-500 hover:text-white' : 'cursor-not-allowed'">
            <SendHorizontal :size="14" />
          </InputGroupAddon>
        </InputGroup>
      </div>
    </template>

    <!-- Image picker dialog -->
    <Dialog v-model:visible="pickerVisible" modal header="Select an image" :style="{ width: '400px' }">
      <button
        class="w-full mb-3 py-2 rounded border border-dashed border-slate-300 bg-slate-50 text-sm text-slate-600 hover:bg-slate-100 hover:border-slate-400 transition-colors flex items-center justify-center gap-2"
        @click="onBrowseFile">
        <FolderOpen :size="14" />
        Browse files...
      </button>
      <div v-if="inferenceStore.generatedImages.length" class="grid grid-cols-4 gap-2 max-h-80 overflow-y-auto">
        <div
          v-for="img in inferenceStore.generatedImages"
          :key="img.jobId"
          class="aspect-square rounded cursor-pointer overflow-hidden border-2 border-transparent hover:border-blue-500 transition-colors"
          @click="onPickImage(img.imagePath)">
          <img v-if="img.dataUrl" :src="img.dataUrl" alt="" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-slate-200 flex items-center justify-center">
            <ImageIcon :size="16" class="text-slate-400" />
          </div>
        </div>
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { marked } from 'marked';
import { useChatStore } from '@/stores/chat';
import { useInferenceStore } from '@/stores/inference';
import { useSettingsStore } from '@/stores/settings';
import { usePywebview } from '@/composables/usePywebview';
import ChatImageThumb from './ChatImageThumb.vue';

const chatStore = useChatStore();
const inferenceStore = useInferenceStore();
const settingsStore = useSettingsStore();
const { api } = usePywebview();
const chatInput = ref('');
const apiKeyInput = ref('');
const messagesContainer = ref<HTMLElement | null>(null);

const providerLabel = computed(() => {
  return settingsStore.aiProvider === 'perplexity' ? 'Perplexity' : 'Claude';
});

const providerPlaceholder = computed(() => {
  return settingsStore.aiProvider === 'perplexity' ? 'pplx-...' : 'sk-ant-...';
});

// Image picker state
const pickerVisible = ref(false);
const pickerMode = ref<'style' | 'subject' | 'both'>('both');

const suggestionChips = ['Improve my prompt', 'Suggest lighting', 'Art style ideas', 'More detail'];

const isSendMessageEnabled = computed(() => {
  return !(!chatInput.value.trim() || chatStore.isStreaming);
});

function renderMarkdown(text: string): string {
  return marked.parse(text, { breaks: true, async: false }) as string;
}

function parsePaths(json: string | null): string[] {
  if (!json) return [];
  try {
    return JSON.parse(json);
  } catch {
    return [];
  }
}

function parseToolCalls(json: string | null): Array<{ name: string; input: Record<string, any> }> {
  if (!json) return [];
  try {
    return JSON.parse(json);
  } catch {
    return [];
  }
}

function formatToolName(name: string, input: Record<string, any>): string {
  if (name === 'update_prompt') return 'Updated prompt';
  if (name === 'update_generation_settings') {
    const keys = Object.keys(input).join(', ');
    return `Updated ${keys}`;
  }
  return name;
}

function onSend() {
  if (!chatInput.value.trim() || chatStore.isStreaming) return;
  chatStore.sendMessage(chatInput.value);
  chatInput.value = '';
}

function onSendChip(chip: string) {
  chatStore.sendMessage(chip);
}

async function onNewChat() {
  await chatStore.createConversation();
}

async function onSetApiKey() {
  if (!apiKeyInput.value.trim()) return;
  await chatStore.setApiKey(apiKeyInput.value.trim(), settingsStore.aiProvider);
  apiKeyInput.value = '';
}

function openPicker(mode: 'style' | 'subject' | 'both') {
  pickerMode.value = mode;
  pickerVisible.value = true;
}

async function onBrowseFile() {
  if (!api.value) return;
  const res = await api.value.browse_image_file();
  if (res.status === 'success' && res.path) {
    await onPickImage(res.path);
  }
}

async function onPickImage(imagePath: string) {
  pickerVisible.value = false;

  // Ensure a conversation exists
  if (!chatStore.activeConversationId) {
    await chatStore.createConversation();
  }

  chatStore.addPendingImage(imagePath);
  const aiPrompt = settingsStore.aiPrompts[pickerMode.value];
  chatStore.sendMessage(aiPrompt.prompt, aiPrompt.displayText);
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

// Auto-scroll when streaming text changes
watch(
  () => chatStore.streamingText,
  () => {
    nextTick(scrollToBottom);
  },
);

// Auto-scroll when new messages arrive
watch(
  () => chatStore.messages.length,
  () => {
    nextTick(scrollToBottom);
  },
);
</script>

<style scoped>
.prose-chat :deep(p) {
  margin: 0.25em 0;
}
.prose-chat :deep(ul),
.prose-chat :deep(ol) {
  margin: 0.25em 0;
  padding-left: 1.25em;
}
.prose-chat :deep(li) {
  margin: 0.1em 0;
}
.prose-chat :deep(strong) {
  font-weight: 600;
}
.prose-chat :deep(code) {
  font-size: 1rem;
  background: #e2e8f0;
  padding: 0.1em 0.3em;
  border-radius: 0.25em;
}
.prose-chat :deep(pre) {
  background: #1e293b;
  color: #e2e8f0;
  padding: 0.5em;
  border-radius: 0.375em;
  overflow-x: auto;
  margin: 0.25em 0;
}
.prose-chat :deep(pre code) {
  background: transparent;
  padding: 0;
  color: inherit;
}
</style>
