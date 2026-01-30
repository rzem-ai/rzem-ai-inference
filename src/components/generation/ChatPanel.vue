<template>
  <div class="flex h-full flex-col">
    <!-- Header -->
    <div class="shrink">
      <div class="flex items-start justify-between px-4 pt-6 pb-0">
        <div class="flex flex-col gap-y-1">
          <h2 class="text-xl font-medium text-surface-50">Prompt Assistant</h2>
          <p class="text-xs text-surface-400">AI-powered prompt refinement</p>
        </div>
        <Button
          v-if="chatStore.hasMessages"
          icon="pi pi-trash"
          severity="secondary"
          text
          size="small"
          v-tooltip.left="'Clear chat'"
          @click="chatStore.clearChat()"
        />
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="!chatStore.hasMessages && !chatStore.isLoading" class="flex flex-1 flex-col items-center justify-center gap-4 p-4">
      <div class="text-center text-surface-400">
        <fa :icon="['fal', 'sparkles']" size="2x" class="mb-2 text-primary-400/50" />
        <p class="text-sm">Ask me to help refine your prompt</p>
        <p class="mt-1 text-xs text-surface-500">I can suggest details, styles, and improvements</p>
      </div>

      <!-- Quick Suggestions -->
      <div class="flex flex-wrap justify-center gap-2">
        <Button
          v-for="suggestion in quickSuggestions"
          :key="suggestion"
          :label="suggestion"
          size="small"
          severity="secondary"
          outlined
          class="text-xs"
          @click="handleQuickSuggestion(suggestion)"
        />
      </div>
    </div>

    <!-- Messages List -->
    <div v-else ref="messagesContainer" class="flex-1 overflow-y-auto p-3">
      <div class="flex flex-col gap-3">
        <ChatMessage
          v-for="(msg, index) in chatStore.messages"
          :key="index"
          :message="msg"
          @apply-prompt="handleApplyPrompt"
        />

        <!-- Loading Indicator -->
        <div v-if="chatStore.isLoading" class="mr-4 flex items-center gap-2 rounded-lg bg-surface-700 p-3">
          <ProgressSpinner style="width: 16px; height: 16px" strokeWidth="4" />
          <span class="text-sm text-surface-400">Thinking...</span>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="chatStore.error" class="mx-3 mb-2 rounded border border-red-800/50 bg-red-900/20 p-2">
      <div class="flex items-start gap-2">
        <fa :icon="['fal', 'exclamation-circle']" class="mt-0.5 text-red-400" />
        <div class="flex-1">
          <p class="text-xs text-red-400">{{ chatStore.error }}</p>
          <Button
            v-if="chatStore.error.includes('API key')"
            label="Open Settings"
            size="small"
            severity="secondary"
            text
            class="mt-1 p-0 text-xs"
            @click="openSettings"
          />
        </div>
        <Button
          icon="pi pi-times"
          severity="secondary"
          text
          size="small"
          class="p-0"
          @click="chatStore.error = null"
        />
      </div>
    </div>

    <!-- Input Area -->
    <div class="shrink border-t border-surface-700 p-3">
      <div class="flex gap-2">
        <InputText
          v-model="inputText"
          placeholder="Ask for help with your prompt..."
          class="flex-1"
          size="small"
          :disabled="chatStore.isLoading"
          @keyup.enter="handleSend"
        />
        <Button
          icon="pi pi-send"
          size="small"
          :loading="chatStore.isLoading"
          :disabled="!inputText.trim() || chatStore.isLoading"
          @click="handleSend"
        />
      </div>

      <!-- Context Info -->
      <div v-if="currentPromptPreview" class="mt-2 text-xs text-surface-500">
        <span class="font-medium">Current prompt:</span>
        <span class="ml-1 italic">{{ currentPromptPreview }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue';
import { useRouter } from 'vue-router';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import ProgressSpinner from 'primevue/progressspinner';
import ChatMessage from './ChatMessage.vue';
import { useChatbotStore, QUICK_SUGGESTIONS } from '@/stores/chatbot';
import { useGenerationStore } from '@/stores/generation';

const chatStore = useChatbotStore();
const generationStore = useGenerationStore();
const router = useRouter();

const inputText = ref('');
const messagesContainer = ref<HTMLElement | null>(null);

// Use a subset of quick suggestions
const quickSuggestions = QUICK_SUGGESTIONS.slice(0, 4);

// Preview of current prompt (truncated)
const currentPromptPreview = computed(() => {
  const prompt = generationStore.currentParams.prompt;
  if (!prompt || prompt.length === 0) return null;
  if (prompt.length <= 50) return prompt;
  return prompt.substring(0, 50) + '...';
});

function handleSend() {
  if (!inputText.value.trim() || chatStore.isLoading) return;
  chatStore.sendMessage(inputText.value);
  inputText.value = '';
}

function handleQuickSuggestion(suggestion: string) {
  chatStore.sendMessage(suggestion);
}

function handleApplyPrompt(prompt: string) {
  chatStore.applyPrompt(prompt);
}

function openSettings() {
  router.push('/settings');
}

// Auto-scroll on new messages
watch(
  () => chatStore.messages.length,
  async () => {
    await nextTick();
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  }
);

// Also scroll when loading state changes
watch(
  () => chatStore.isLoading,
  async () => {
    await nextTick();
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  }
);
</script>

<style scoped>
@reference "tailwindcss";

/* Custom scrollbar */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}
</style>
