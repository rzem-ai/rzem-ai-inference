<template>
  <Card class="h-full border rounded-xl border-surface-700 bg-surface-950">
    <template #title>
      <!-- Header -->
      <div class="flex items-start">
        <Button v-if="chatStore.hasMessages" icon="pi pi-trash" severity="secondary" size="small" v-tooltip.left="'Clear chat'" @click="chatStore.clearChat()">
          clear
        </Button>
      </div>
      <div class="flex flex-col gap-1">
        <div class="text-sm font-semibold"> Prompt Assistant </div>
        <div class="text-xs font-light"> AI-powered prompt refinement </div>
      </div>
    </template>

    <template #content>
      <div class="flex flex-col h-full">
        <!-- Empty State -->
        <div v-if="!chatStore.hasMessages && !chatStore.isLoading" class="flex flex-col items-center justify-center flex-1 gap-4 p-4">
          <div class="text-center text-surface-400">
            <fa :icon="['fal', 'sparkles']" size="2x" class="mb-2 text-primary-400/50" />
            <p class="text-base">You can ask me to help refine your prompt</p>
            <p class="mt-1 text-xs text-surface-500">I can suggest details, styles, and improvements</p>
          </div>

          <!-- Quick Suggestions -->
          <div class="flex flex-wrap justify-center gap-2">
            <Button
              v-for="suggestion in quickSuggestions"
              :key="suggestion"
              :label="suggestion"
              size="small"
              severity="primary"
              variant="outlined"
              class="text-sm"
              @click="handleQuickSuggestion(suggestion)" />
          </div>
        </div>

        <!-- Messages List -->
        <div v-else ref="messagesContainer" class="flex-1 min-h-0 overflow-y-auto">
          <div class="flex flex-col w-full gap-3">
            <ChatMessage v-for="(msg, index) in chatStore.messages" :key="index" :message="msg" @apply-prompt="handleApplyPrompt" />

            <!-- Loading Indicator -->
            <div v-if="chatStore.isLoading" class="flex justify-center p-3 mr-4 border rounded-lg bg-surface-500/10 border-surface-500/40">
              <div class="flex gap-2">
              <ProgressSpinner style="width: 16px; height: 16px" strokeWidth="4" />
              <span class="text-sm font-medium text-surface-400">Thinking ...</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Error Display -->
        <Message v-if="chatStore.error" severity="error" class="mt-2 shrink-0">
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
                class="p-0 mt-1 text-xs"
                @click="openSettings" />
            </div>
            <Button icon="pi pi-times" severity="secondary" text size="small" class="p-0" @click="chatStore.error = null" />
          </div>
        </Message>
      </div>
    </template>
    <template #footer>
      <!-- Input Area -->
      <div class="pt-2 border-t shrink border-surface-700">
        <div class="flex gap-2">
          <InputText
            v-model="inputText"
            placeholder="Ask for help with your prompt..."
            class="flex-1"
            size="small"
            :disabled="chatStore.isLoading"
            @keyup.enter="handleSend" />
          <Button severity="primary" size="small" :loading="chatStore.isLoading" :disabled="!inputText.trim() || chatStore.isLoading" @click="handleSend">
            <fa :icon="['fal', 'send']" class="" />
          </Button>
        </div>

        <!-- Context Info -->
        <div v-if="currentPromptPreview" class="mt-2 text-xs text-surface-500">
          <span class="font-medium">Current prompt:</span>
          <span class="ml-1 italic">{{ currentPromptPreview }}</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import { useRouter } from 'vue-router';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import ProgressSpinner from 'primevue/progressspinner';
import ChatMessage from './ChatMessage.vue';
import { useChatbotStore, QUICK_SUGGESTIONS } from '@/stores/chatbot';
import { useGenerationStore } from '@/stores/generation';
import { Card, Message } from 'primevue';
import { TextureLoader } from 'three';

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
  if (prompt.length <= 80) return prompt;
  return prompt.substring(0, 80) + '...';
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
  },
);

// Also scroll when loading state changes
watch(
  () => chatStore.isLoading,
  async () => {
    await nextTick();
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  },
);

onMounted(() => {
  //chatStore.isLoading = true;
});
</script>

<style scoped>
@reference "tailwindcss";

:deep(.p-card-body) {
  @apply h-full flex flex-col;
}

:deep(.p-card-caption) {
  @apply shrink-0;
}

:deep(.p-card-content) {
  @apply flex-1 min-h-0;
}

:deep(.p-card-footer) {
  @apply shrink-0;
}
</style>
