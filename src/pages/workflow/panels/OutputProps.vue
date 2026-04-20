<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Output</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Output preview -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Result</label>

      <div v-if="!nodeState" class="text-xs text-surface-500 italic">
        No output yet. Run the workflow to see results.
      </div>

      <template v-else>
        <!-- Status -->
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full" :class="statusColor" />
          <span class="text-xs text-surface-400 capitalize">{{ nodeState.status }}</span>
        </div>

        <!-- Error -->
        <div v-if="nodeState.error" class="text-xs text-red-400 bg-red-950/30 rounded-lg p-2 mt-1">
          {{ nodeState.error }}
        </div>

        <!-- Outputs -->
        <div v-if="nodeState.outputs" class="flex flex-col gap-2 mt-1">
          <!-- Image outputs — clickable to open lightbox -->
          <div v-if="outputImageUrl" class="relative cursor-zoom-in" @click="lightboxOpen = true">
            <img :src="outputImageUrl" alt="Output" class="w-full rounded-lg object-contain max-h-48 hover:opacity-90 transition-opacity" />
          </div>

          <!-- Text outputs (rendered as HTML) -->
          <div
            v-if="outputText"
            class="text-sm text-surface-300 bg-surface-800 rounded-lg p-3 overflow-y-auto prose-workflow"
            v-html="renderMarkdown(outputText)" />
        </div>
      </template>
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>

  <!-- Lightbox -->
  <Teleport to="body">
    <Transition name="overlay-fade">
      <div
        v-if="lightboxOpen"
        class="fixed inset-0 z-50 flex items-center justify-center"
        @keydown.escape="lightboxOpen = false"
        @click.self="lightboxOpen = false"
        tabindex="0"
        ref="lightboxRef">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/90" @click="lightboxOpen = false" />

        <!-- Close button -->
        <button
          class="absolute top-4 right-4 z-10 p-2 rounded-full bg-white/10 backdrop-blur-sm text-white/70 hover:text-white hover:bg-white/20 transition-colors"
          @click="lightboxOpen = false">
          <X :size="20" />
        </button>

        <!-- Image -->
        <div class="relative max-w-[90vw] max-h-[90vh] z-1">
          <img :src="outputImageUrl!" alt="Output" class="max-w-[90vw] max-h-[90vh] object-contain rounded-lg select-none" draggable="false" />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { marked } from 'marked';
import { X } from 'lucide-vue-next';
import { useWorkflowStore } from '@/stores/workflow';
import { getApiAsync } from '@/bridge';
import type { OutputNodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

function renderMarkdown(text: string): string {
  return marked.parse(text, { breaks: true, async: false }) as string;
}

const workflowStore = useWorkflowStore();

const nodeData = computed(() => workflowStore.selectedNodeData as OutputNodeData);
const nodeState = computed(() => workflowStore.selectedNodeState);
const outputImageUrl = ref<string | null>(null);
const lightboxOpen = ref(false);
const lightboxRef = ref<HTMLElement>();

const statusColor = computed(() => {
  switch (nodeState.value?.status) {
    case 'completed': return 'bg-green-400';
    case 'running': return 'bg-blue-400';
    case 'pending': return 'bg-yellow-400';
    case 'failed': return 'bg-red-400';
    default: return 'bg-surface-500';
  }
});

const outputText = computed(() => {
  const outputs = nodeState.value?.outputs;
  if (!outputs) return null;
  if (typeof outputs.text === 'string') return outputs.text;
  if (typeof outputs.answer === 'string') return outputs.answer;
  return null;
});

// Load image output if present
watch(
  () => nodeState.value?.outputs,
  async (outputs) => {
    if (!outputs) {
      outputImageUrl.value = null;
      return;
    }
    const imagePath = typeof outputs.image === 'string' ? outputs.image
      : typeof outputs.image_path === 'string' ? outputs.image_path
      : null;
    if (imagePath) {
      const api = await getApiAsync();
      const res = await api.get_image_base64({ image_path: imagePath });
      if (res.status === 'success' && res.data_url) {
        outputImageUrl.value = res.data_url;
      }
    } else {
      outputImageUrl.value = null;
    }
  },
  { immediate: true, deep: true },
);

// Focus the lightbox element so Escape key works
watch(lightboxOpen, async (open) => {
  if (open) {
    await nextTick();
    lightboxRef.value?.focus();
  }
});

function handleUpdate(partial: Partial<OutputNodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}
</script>

<style scoped>
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.2s ease;
}
.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}

.prose-workflow :deep(h1),
.prose-workflow :deep(h2),
.prose-workflow :deep(h3) {
  font-weight: 600;
  margin-top: 0.75em;
  margin-bottom: 0.25em;
  color: var(--p-surface-200);
}
.prose-workflow :deep(h1) { font-size: 1.125rem; }
.prose-workflow :deep(h2) { font-size: 1rem; }
.prose-workflow :deep(h3) { font-size: 0.875rem; }
.prose-workflow :deep(p) { margin: 0.4em 0; line-height: 1.6; }
.prose-workflow :deep(ul),
.prose-workflow :deep(ol) { padding-left: 1.25em; margin: 0.4em 0; }
.prose-workflow :deep(li) { margin: 0.2em 0; }
.prose-workflow :deep(strong) { color: var(--p-surface-100); }
.prose-workflow :deep(code) {
  background: var(--p-surface-700);
  padding: 0.1em 0.35em;
  border-radius: 0.25rem;
  font-size: 0.85em;
}
.prose-workflow :deep(blockquote) {
  border-left: 3px solid var(--p-surface-500);
  padding-left: 0.75em;
  color: var(--p-surface-400);
  margin: 0.5em 0;
}
</style>
