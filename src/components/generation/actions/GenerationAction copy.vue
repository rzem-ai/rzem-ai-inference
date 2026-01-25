<template>
  <Panel :collapsed="props.collapsed" :toggleable="props.toggleable">
    <template #header>
      <div class="flex gap-2 px-0 py-2 text-xs font-semibold tracking-wider uppercase text-surface-300">
        <fa :icon="['fal', props.icon]" size="sm" />
        {{ props.label }}
      </div>
    </template>

    <div class="flex flex-col gap-2 px-2">
      <slot></slot>
    </div>
  </Panel>
</template>
<script setup lang="ts">
import Panel from 'primevue/panel';

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);
</script>
