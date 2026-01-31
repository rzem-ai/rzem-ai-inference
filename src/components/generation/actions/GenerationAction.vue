<template>
  <li class="pb-3">
    <div :class="props.panelClass ? props.panelClass : ''">
      <div>
        <div class="flex items-center gap-1 px-0 py-2 text-sm font-semibold tracking-wider text-surface-300">
          {{ props.label }}
        </div>
      </div>

      <div class="flex flex-col gap-2" :class="props.slotClass ? props.slotClass : ''">
        <slot></slot>
      </div>
    </div>
  </li>
</template>
<script setup lang="ts">
const props = defineProps(['collapsed', 'icon', 'label', 'toggleable', 'panelClass', 'slotClass']);
</script>
