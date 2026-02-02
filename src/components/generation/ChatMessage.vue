<template>
  <div class="flex" :class="[isUserMessage ? 'justify-end' : 'justify-start']">
    <div
      class="flex flex-col w-full gap-1 p-2 border rounded-lg"
      :class="[isUserMessage ? 'ml-4 border-primary-500/40 bg-primary-500/10' : 'mr-4 bg-surface-500/10 border-surface-500/40']">
      <!-- Message Header -->
      <div class="flex items-center justify-between">
        <div class="text-xs font-semibold" :class="isUserMessage ? 'text-primary-600' : 'text-surface-400'">
          {{ isUserMessage ? 'You' : 'Assistant' }}
        </div>
        <div class="text-xs text-surface-500">{{ formattedTime }}</div>
      </div>

      <!-- Message Content -->
      <div class="text-sm text-surface-200">
        <div v-if="message.role === 'assistant'" class="assistant-message" v-html="formattedContent" />
        <div v-else class="whitespace-pre-wrap">{{ message.content }}</div>
      </div>

      <!-- Apply Suggested Prompt Button -->
      <div v-if="message.suggestedPrompt" class="flex gap-2 mt-3">
        <Button size="small" class="grow hover:shadow-lg hover:border hover:border-surface-400"  severity="secondary" outlined @click="handleCopy"><fa :icon="['fal', 'clipboard']" class="" /> Copy</Button>
        <Button size="small" class="grow hover:shadow-lg hover:border hover:border-blue-400" severity="primary" @click="handleApply"><fa :icon="['fal', 'square-check']" class="" /> Apply</Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';
import Button from 'primevue/button';
import { useToast } from 'primevue/usetoast';
import type { ChatMessage } from '@/stores/chatbot';

const props = defineProps<{
  message: ChatMessage;
}>();

const emit = defineEmits<{
  applyPrompt: [prompt: string];
}>();

const toast = useToast();

const formattedTime = computed(() => {
  const date = new Date(props.message.timestamp);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
});

const isUserMessage = computed(() => {
  return props.message.role === 'user';
});

const formattedContent = computed(() => {
  return marked.parse(props.message.content) as string;
});

function handleApply() {
  if (props.message.suggestedPrompt) {
    emit('applyPrompt', props.message.suggestedPrompt);
    toast.add({
      severity: 'success',
      summary: 'Prompt Applied',
      detail: 'The suggested prompt has been applied',
      life: 2000,
    });
  }
}

async function handleCopy() {
  if (props.message.suggestedPrompt) {
    try {
      await navigator.clipboard.writeText(props.message.suggestedPrompt);
      toast.add({
        severity: 'info',
        summary: 'Copied',
        detail: 'Prompt copied to clipboard',
        life: 2000,
      });
    } catch {
      toast.add({
        severity: 'error',
        summary: 'Copy Failed',
        detail: 'Could not copy to clipboard',
        life: 2000,
      });
    }
  }
}
</script>

<style scoped>
@reference "tailwindcss";

.assistant-message :deep(h2) {
  @apply m-0 font-semibold mb-0.5;
}

.assistant-message :deep(p) {
  @apply mb-2;
}

.assistant-message :deep(p:last-child) {
  @apply mb-0;
}

.assistant-message :deep(strong) {
  @apply font-medium;
}

.assistant-message :deep(ul),
.assistant-message :deep(ol) {
  @apply pl-4 mb-2;
}

.assistant-message :deep(li) {
  @apply mb-0.5;
}

.assistant-message :deep(code) {
  @apply px-1 rounded bg-gray-200 text-blue-300;
}

.assistant-message :deep(pre) {
  @apply my-4 px-2 py-1 border rounded bg-gray-100 border-blue-500;
}

.assistant-message :deep(pre code) {
  @apply p-0 bg-transparent text-xs font-light whitespace-pre-wrap text-blue-800;
}
</style>
