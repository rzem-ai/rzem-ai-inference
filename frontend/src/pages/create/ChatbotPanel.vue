<template>
  <div
    class="flex flex-col h-full"
    @dragover.prevent="isDragging = true"
    @dragleave.self="isDragging = false"
    @drop.prevent="onDrop">

    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-slate-100">
      <div class="flex items-center gap-2">
        <Bot :size="14" class="text-blue-500" />
        <span class="text-xs font-semibold text-slate-700">AI Assistant</span>
      </div>
      <button
        class="p-1 rounded hover:bg-slate-100 text-slate-400 hover:text-slate-600 transition-colors"
        title="New Chat"
        @click="onNewChat">
        <Plus :size="14" />
      </button>
    </div>

    <!-- Not configured state -->
    <div v-if="!chatStore.isConfigured" class="flex-1 flex flex-col items-center justify-center px-6 gap-3">
      <KeyRound :size="24" class="text-slate-300" />
      <p class="text-xs text-slate-500 text-center">Enter your Claude API key to enable the AI assistant.</p>
      <div class="flex gap-1.5 w-full">
        <input
          v-model="apiKeyInput"
          type="password"
          placeholder="sk-ant-..."
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
      <!-- Messages area -->
      <div ref="messagesContainer" class="flex-1 overflow-y-auto px-3 py-2 flex flex-col gap-3">
        <!-- Empty state with suggestions -->
        <div v-if="!chatStore.messages.length && !chatStore.isStreaming" class="flex-1 flex flex-col items-center justify-center gap-3">
          <Bot :size="24" class="text-slate-300" />
          <p class="text-xs text-slate-400 text-center">Ask me to help craft prompts, adjust settings, or analyze images.</p>
          <div class="flex gap-1.5 flex-wrap justify-center">
            <button
              v-for="chip in suggestionChips"
              :key="chip"
              class="text-[10px] px-2 py-1 rounded-full bg-slate-100 text-slate-600 hover:bg-slate-200 transition-colors"
              @click="onSendChip(chip)">
              {{ chip }}
            </button>
          </div>
        </div>

        <!-- Message list -->
        <template v-for="msg in chatStore.messages" :key="msg.id">
          <!-- User message -->
          <div v-if="msg.role === 'user'" class="flex justify-end">
            <div class="max-w-[85%]">
              <!-- Image thumbnails -->
              <div v-if="msg.image_paths" class="flex gap-1 mb-1 justify-end">
                <div
                  v-for="(_, i) in parsePaths(msg.image_paths)"
                  :key="i"
                  class="w-10 h-10 rounded bg-slate-200 flex items-center justify-center">
                  <ImageIcon :size="12" class="text-slate-400" />
                </div>
              </div>
              <div class="bg-blue-500 rounded-lg px-3 py-2 text-xs text-white leading-relaxed">
                {{ msg.content }}
              </div>
            </div>
          </div>

          <!-- Assistant message -->
          <div v-else class="flex gap-2">
            <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-0.5">
              <Bot :size="14" class="text-white" />
            </div>
            <div class="flex-1 min-w-0">
              <!-- Tool use badges -->
              <div v-if="msg.tool_calls" class="flex gap-1 mb-1 flex-wrap">
                <span
                  v-for="(tool, i) in parseToolCalls(msg.tool_calls)"
                  :key="i"
                  class="text-[9px] px-1.5 py-0.5 rounded-full bg-blue-100 text-blue-600 font-medium">
                  {{ formatToolName(tool.name, tool.input) }}
                </span>
              </div>
              <div
                v-if="msg.content"
                class="bg-slate-50 rounded-lg px-3 py-2 text-xs text-slate-700 leading-relaxed prose-chat"
                v-html="renderMarkdown(msg.content)" />
            </div>
          </div>
        </template>

        <!-- Streaming bubble -->
        <div v-if="chatStore.isStreaming" class="flex gap-2">
          <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-0.5">
            <Bot :size="14" class="text-white" />
          </div>
          <div v-if="chatStore.streamingText" class="bg-slate-50 rounded-lg px-3 py-2 text-xs text-slate-700 leading-relaxed prose-chat">
            <span v-html="renderMarkdown(chatStore.streamingText)" />
            <span class="inline-block w-1.5 h-3 bg-blue-500 ml-0.5 animate-pulse rounded-sm" />
          </div>
          <div v-else class="bg-slate-50 rounded-lg px-3 py-2">
            <div class="flex gap-1 items-center">
              <div class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 0ms" />
              <div class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 150ms" />
              <div class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 300ms" />
            </div>
          </div>
        </div>
      </div>

      <!-- Pending images strip -->
      <div v-if="chatStore.pendingImagePaths.length" class="px-3 py-1.5 border-t border-slate-100">
        <div class="flex gap-1.5">
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

      <!-- Drag overlay -->
      <div
        v-if="isDragging"
        class="absolute inset-0 bg-blue-500/10 border-2 border-dashed border-blue-400 rounded-xl flex items-center justify-center z-10 pointer-events-none">
        <span class="text-xs font-medium text-blue-600">Drop image here</span>
      </div>

      <!-- Chat input -->
      <div class="px-3 py-2 border-t border-slate-100">
        <div class="flex gap-1.5">
          <input
            v-model="chatInput"
            type="text"
            placeholder="Ask for prompt help..."
            class="flex-1 text-xs bg-slate-100 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-blue-300"
            :disabled="chatStore.isStreaming"
            @keydown.enter="onSend" />
          <button
            class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
            :disabled="!chatInput.trim() || chatStore.isStreaming"
            @click="onSend">
            <SendHorizontal :size="14" />
          </button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { Bot, SendHorizontal, Plus, KeyRound, X, Image as ImageIcon } from 'lucide-vue-next';
import { marked } from 'marked';
import { useChatStore } from '@/stores/chat';

const chatStore = useChatStore();
const chatInput = ref('');
const apiKeyInput = ref('');
const isDragging = ref(false);
const messagesContainer = ref<HTMLElement | null>(null);

const suggestionChips = ['Improve my prompt', 'Suggest lighting', 'Art style ideas', 'More detail'];

function renderMarkdown(text: string): string {
  return marked.parse(text, { breaks: true, async: false }) as string;
}

function parsePaths(json: string | null): string[] {
  if (!json) return [];
  try { return JSON.parse(json); } catch { return []; }
}

function parseToolCalls(json: string | null): Array<{ name: string; input: Record<string, any> }> {
  if (!json) return [];
  try { return JSON.parse(json); } catch { return []; }
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
  await chatStore.setApiKey(apiKeyInput.value.trim());
  apiKeyInput.value = '';
}

function onDrop(e: DragEvent) {
  isDragging.value = false;
  const imagePath = e.dataTransfer?.getData('text/image-path');
  if (imagePath) {
    chatStore.addPendingImage(imagePath);
  }
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

// Auto-scroll when streaming text changes
watch(() => chatStore.streamingText, () => {
  nextTick(scrollToBottom);
});

// Auto-scroll when new messages arrive
watch(() => chatStore.messages.length, () => {
  nextTick(scrollToBottom);
});
</script>

<style scoped>
.prose-chat :deep(p) {
  margin: 0.25em 0;
}
.prose-chat :deep(ul), .prose-chat :deep(ol) {
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
  font-size: 0.65rem;
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
