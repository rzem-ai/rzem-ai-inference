<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="text-sm font-semibold text-surface-300">Image Generation</div>

    <!-- Label -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Label</label>
      <InputText :model-value="nodeData.label" @update:model-value="handleUpdate({ label: $event })" fluid />
    </div>

    <!-- Bundle selection -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Model Bundle</label>
      <Select
        :model-value="nodeData.bundleId"
        :options="bundleOptions"
        option-label="label"
        option-value="value"
        placeholder="Select bundle"
        fluid
        @update:model-value="handleBundleChange" />
    </div>

    <!-- Steps -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Steps</label>
      <InputNumber
        :model-value="nodeData.steps"
        @update:model-value="handleUpdate({ steps: $event ?? 28 })"
        :min="1"
        :max="150"
        fluid />
    </div>

    <!-- CFG Scale -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">CFG Scale</label>
      <InputNumber
        :model-value="nodeData.cfg"
        @update:model-value="handleUpdate({ cfg: $event ?? 3.5 })"
        :min="1"
        :max="30"
        :min-fraction-digits="1"
        :max-fraction-digits="1"
        fluid />
    </div>

    <!-- Width -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Width</label>
      <InputNumber
        :model-value="nodeData.width"
        @update:model-value="handleUpdate({ width: $event ?? 1024 })"
        :min="64"
        :max="2048"
        :step="64"
        fluid />
    </div>

    <!-- Height -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Height</label>
      <InputNumber
        :model-value="nodeData.height"
        @update:model-value="handleUpdate({ height: $event ?? 1024 })"
        :min="64"
        :max="2048"
        :step="64"
        fluid />
    </div>

    <!-- Seed -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Seed <span class="text-surface-500">(-1 = random)</span></label>
      <InputNumber
        :model-value="nodeData.seed"
        @update:model-value="handleUpdate({ seed: $event ?? -1 })"
        :min="-1"
        fluid />
    </div>

    <!-- Sampler -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Sampler</label>
      <Select
        :model-value="nodeData.sampler"
        :options="samplerOptions"
        option-label="label"
        option-value="value"
        fluid
        @update:model-value="handleUpdate({ sampler: $event })" />
    </div>

    <!-- Scheduler -->
    <div class="flex flex-col gap-1">
      <label class="text-sm text-surface-400">Scheduler</label>
      <Select
        :model-value="nodeData.scheduler"
        :options="schedulerOptions"
        option-label="label"
        option-value="value"
        fluid
        @update:model-value="handleUpdate({ scheduler: $event })" />
    </div>

    <!-- Execution status -->
    <NodeStatusBadge :node-id="workflowStore.selectedNodeId!" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useWorkflowStore } from '@/stores/workflow';
import { useInferenceStore } from '@/stores/inference';
import type { ImageGenNodeData } from '@/types/workflow';
import NodeStatusBadge from './NodeStatusBadge.vue';

const workflowStore = useWorkflowStore();
const inferenceStore = useInferenceStore();

const nodeData = computed(() => workflowStore.selectedNodeData as ImageGenNodeData);

const bundleOptions = computed(() =>
  inferenceStore.bundles.map((b) => ({
    label: b.label,
    value: b.id,
  })),
);

const samplerOptions = [
  { value: 'euler', label: 'Euler' },
  { value: 'euler_a', label: 'Euler Ancestral' },
  { value: 'dpm++_2m', label: 'DPM++ 2M' },
  { value: 'dpm++_2s', label: 'DPM++ 2S' },
  { value: 'dpm++_sde', label: 'DPM++ SDE' },
  { value: 'heun', label: 'Heun' },
  { value: 'lms', label: 'LMS' },
];

const schedulerOptions = [
  { value: 'normal', label: 'Normal' },
  { value: 'karras', label: 'Karras' },
  { value: 'simple', label: 'Simple' },
  { value: 'sgm_uniform', label: 'SGM Uniform' },
  { value: 'ddim_uniform', label: 'DDIM Uniform' },
  { value: 'exponential', label: 'Exponential' },
  { value: 'beta', label: 'Beta' },
];

function handleUpdate(partial: Partial<ImageGenNodeData>) {
  if (workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, partial);
  }
}

function handleBundleChange(bundleId: string) {
  const bundle = inferenceStore.bundles.find((b) => b.id === bundleId);
  if (bundle && workflowStore.selectedNodeId) {
    workflowStore.updateNodeData(workflowStore.selectedNodeId, {
      bundleId: bundle.id,
      steps: bundle.steps,
      cfg: bundle.cfg_scale,
      sampler: bundle.sampler,
      scheduler: bundle.scheduler,
    });
  }
}
</script>
