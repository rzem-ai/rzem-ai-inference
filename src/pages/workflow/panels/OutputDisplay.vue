<template>
  <div class="flex flex-col gap-3 p-4 h-full">
    <div class="flex items-center justify-between">
      <div class="text-sm font-semibold text-surface-300">Workflow Output</div>
      <Tag v-if="workflowStore.isRunning" severity="info" value="Running" />
      <Tag v-else-if="hasOutputs" severity="success" value="Complete" />
    </div>

    <!-- No outputs -->
    <div v-if="!hasOutputs && !workflowStore.isRunning && !workflowStore.error" class="flex-1 flex items-center justify-center">
      <span class="text-sm text-surface-500">Run the workflow to see outputs here.</span>
    </div>

    <!-- Running indicator -->
    <div v-if="workflowStore.isRunning" class="flex items-center gap-2">
      <ProgressBar mode="indeterminate" class="h-1 flex-1" />
    </div>

    <!-- Error -->
    <div v-if="workflowStore.error" class="text-sm text-red-400 bg-red-950/30 rounded-lg p-3">
      {{ workflowStore.error }}
    </div>

    <!-- Output content -->
    <div v-if="hasOutputs" class="flex flex-col gap-3 flex-1 overflow-y-auto">
      <!-- Image outputs -->
      <div v-for="(url, index) in imageUrls" :key="'img-' + index" class="rounded-lg overflow-hidden">
        <img :src="url" alt="Workflow output" class="w-full object-contain max-h-64" />
      </div>

      <!-- Text outputs (rendered as markdown) -->
      <div
        v-for="(text, index) in textOutputs"
        :key="'txt-' + index"
        class="text-sm text-surface-300 bg-surface-800 rounded-lg p-3 prose-workflow"
        v-html="renderMarkdown(text)" />

      <!-- Raw data fallback -->
      <div v-if="!imageUrls.length && !textOutputs.length && finalOutputs" class="text-xs text-surface-500 bg-surface-800 rounded-lg p-3 font-mono whitespace-pre-wrap overflow-x-auto">
        {{ JSON.stringify(finalOutputs, null, 2) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { marked } from 'marked';
import { useWorkflowStore } from '@/stores/workflow';
import { getApiAsync } from '@/bridge';

function renderMarkdown(text: string): string {
  return marked.parse(text, { breaks: true, async: false }) as string;
}

const workflowStore = useWorkflowStore();

const finalOutputs = computed(() => workflowStore.finalOutputs);
const hasOutputs = computed(() => finalOutputs.value !== null);
const imageUrls = ref<string[]>([]);

function isImagePath(value: string): boolean {
  return /\.(png|jpe?g|webp|bmp|tiff?)$/i.test(value);
}

/** Extract all string values from nested output objects. */
function collectValues(obj: Record<string, unknown>): string[] {
  const values: string[] = [];
  for (const value of Object.values(obj)) {
    if (typeof value === 'string') {
      values.push(value);
    } else if (value && typeof value === 'object' && !Array.isArray(value)) {
      values.push(...collectValues(value as Record<string, unknown>));
    }
  }
  return values;
}

const textOutputs = computed(() => {
  if (!finalOutputs.value) return [];
  return collectValues(finalOutputs.value).filter(v => !isImagePath(v));
});

// Load images when finalOutputs change
watch(
  finalOutputs,
  async (outputs) => {
    imageUrls.value = [];
    if (!outputs) return;

    const imagePaths = collectValues(outputs).filter(isImagePath);
    if (!imagePaths.length) return;

    const api = await getApiAsync();
    for (const path of imagePaths) {
      const res = await api.get_image_base64({ image_path: path });
      if (res.status === 'success' && res.data_url) {
        imageUrls.value.push(res.data_url as string);
      }
    }
  },
  { immediate: true, deep: true },
);
</script>

<style scoped>
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
