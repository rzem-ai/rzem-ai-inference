<template>
  <div>
    <div>
      <div class="flex items-center gap-1 px-0 py-2 text-sm font-semibold tracking-wider text-surface-300">
        {{ props.label }}
      </div>
    </div>

    <div class="flex flex-col gap-2 px-2">
      <slot></slot>
    </div>
  </div>
</template>
<script setup lang="ts">

const props = defineProps(['collapsed', 'icon', 'label', 'toggleable']);
</script>
