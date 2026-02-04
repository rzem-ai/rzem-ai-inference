<template>
  <div class="flex flex-col items-center justify-center h-full p-8 text-center">
    <fa :icon="['fal', 'palette']" size="4x" class="mb-6 text-surface-500" />
    <h3 class="mb-2 text-xl font-semibold text-surface-100">Style Management</h3>
    <p class="mb-6 text-sm text-surface-400">
      Create and organize your prompt styles to streamline your workflow
    </p>
    <div class="flex flex-col w-full max-w-xs gap-2">
      <Button @click="emit('create')" severity="primary" size="large" class="w-full">
        <fa :icon="['fal', 'plus']" class="mr-2" />
        Create New Style
      </Button>
      <Button @click="emit('import')" severity="secondary" variant="outlined" size="large" class="w-full">
        <fa :icon="['fal', 'download']" class="mr-2" />
        Import LoRA
      </Button>
    </div>
    <Divider />
    <div class="mt-6 space-y-2 text-left text-surface-400">
      <div class="flex items-start gap-2 text-sm">
        <fa :icon="['fal', 'check']" class="mt-1 text-primary-400" />
        <span>Save prompt templates for quick reuse</span>
      </div>
      <div class="flex items-start gap-2 text-sm">
        <fa :icon="['fal', 'check']" class="mt-1 text-primary-400" />
        <span>Organize styles by category</span>
      </div>
      <div class="flex items-start gap-2 text-sm">
        <fa :icon="['fal', 'check']" class="mt-1 text-primary-400" />
        <span>Attach LoRAs to styles</span>
      </div>
      <div class="flex items-start gap-2 text-sm">
        <fa :icon="['fal', 'check']" class="mt-1 text-primary-400" />
        <span>Track usage statistics</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Button from 'primevue/button';
import Divider from 'primevue/divider';

const emit = defineEmits<{
  create: [];
  import: [];
}>();
</script>
