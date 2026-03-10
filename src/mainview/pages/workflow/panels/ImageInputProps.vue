<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Image Input</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Image preview -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Image</label>
      <div v-if="imageDataUrl" class="relative">
        <img :src="imageDataUrl" alt="Input image" class="w-full rounded-lg object-contain max-h-48" />
        <Button
          class="absolute top-1 right-1"
          severity="danger"
          size="small"
          rounded
          text
          @click="handleClearImage">
          <X :size="14" />
        </Button>
      </div>
      <div v-else-if="nodeData.imagePath" class="text-xs text-surface-500 truncate">
        {{ nodeData.imagePath }}
      </div>
    </div>

    <!-- Browse button -->
    <div class="flex gap-2">
      <Button size="small" severity="secondary" variant="outlined" class="flex-1" @click="handleBrowse">
        <Upload :size="14" class="mr-1" />
        Browse Image
      </Button>
      <Button size="small" severity="secondary" variant="outlined" disabled title="Coming soon">
        <ClipboardCheck :size="14" class="mr-1" />
        Paste
      </Button>
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import { getApiAsync } from '@/bridge';
import type { ImageInputNodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

const workflowStore = useWorkflowStore();

const nodeData = computed(() => workflowStore.selectedNodeData as ImageInputNodeData);
const imageDataUrl = ref<string | null>(null);

watch(
  () => nodeData.value?.imagePath,
  async (path) => {
    if (!path) {
      imageDataUrl.value = null;
      return;
    }
    const api = await getApiAsync();
    const res = await api.get_image_base64({ image_path: path });
    if (res.status === 'success' && res.data_url) {
      imageDataUrl.value = res.data_url;
    }
  },
  { immediate: true },
);

function handleUpdate(partial: Partial<ImageInputNodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}

async function handleBrowse() {
  const api = await getApiAsync();
  const res = await api.browse_input_image();
  if (res.status === 'success' && res.path) {
    handleUpdate({ imagePath: res.path });
  }
}

function handleClearImage() {
  handleUpdate({ imagePath: '' });
  imageDataUrl.value = null;
}
</script>
