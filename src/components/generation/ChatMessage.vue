<template>
  <div
    :class="[
      'rounded-lg p-3',
      message.role === 'user' ? 'ml-4 bg-primary-900/30' : 'mr-4 bg-surface-700',
    ]"
  >
    <!-- Message Header -->
    <div class="mb-1 flex items-center justify-between">
      <span class="text-xs font-medium" :class="message.role === 'user' ? 'text-primary-400' : 'text-surface-400'">
        {{ message.role === 'user' ? 'You' : 'Assistant' }}
      </span>
      <span class="text-xs text-surface-500">{{ formattedTime }}</span>
    </div>

    <!-- Message Content -->
    <div class="text-sm text-surface-200 whitespace-pre-wrap">
      <template v-if="message.role === 'assistant'">
        <!-- Render assistant message with prompt highlighting -->
        <div v-html="formattedContent" />
      </template>
      <template v-else>
        {{ message.content }}
      </template>
    </div>

    <!-- Apply Suggested Prompt Button -->
    <div v-if="message.suggestedPrompt" class="mt-3 flex gap-2">
      <Button label="Apply Prompt" icon="pi pi-check" size="small" severity="success" @click="handleApply" />
      <Button label="Copy" icon="pi pi-copy" size="small" severity="secondary" outlined @click="handleCopy" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
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

// Format content with highlighted prompt blocks
const formattedContent = computed(() => {
  let content = props.message.content;

  // Escape HTML first
  content = content.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  // Highlight prompt blocks
  content = content.replace(
    /```prompt\n([\s\S]*?)```/g,
    '<div class="mt-2 mb-2 rounded bg-surface-800 border border-primary-700/50 p-2"><code class="text-primary-300 text-xs whitespace-pre-wrap">$1</code></div>'
  );

  // Highlight inline code
  content = content.replace(
    /`([^`]+)`/g,
    '<code class="bg-surface-800 px-1 rounded text-primary-300">$1</code>'
  );

  return content;
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
