<template>
  <aside class="flex flex-col h-full overflow-hidden w-120 min-w-120">
    <!-- -->
    <div v-if="$slots.header" class="pt-4 shrink">
      <div class="flex items-start justify-between px-2">
        <div class="flex flex-col gap-y-1">
          <div class="text-lg font-medium text-surface-50"><slot name="header"> </slot></div>
        </div>
        <div class=""> </div>
      </div>
    </div>

    <!-- -->
    <div v-if="$slots.toolbar" class="shrink">
      <div class="py-2 pl-2 pr-4">
        <slot name="toolbar"> </slot>
      </div>
    </div>

    <!-- -->
    <div v-if="$slots.body" class="overflow-hidden grow">
      <CustomScrollbar class="h-full">
        <div class="pl-2 pr-4">
          <slot name="body"> </slot>
        </div>
      </CustomScrollbar>
    </div>

    <!-- -->
    <div v-if="$slots.footer" class="shrink">
      <div class="border-t border-surface-600"></div>
      <slot name="footer"> </slot>
    </div>
  </aside>
</template>

<script setup lang="ts">
import CustomScrollbar from '@/components/CustomScrollbar.vue';
</script>

<style scoped>
@reference "tailwindcss";

/* Ensure the scrollable area has proper height constraints */
.grow {
  min-height: 0; /* Critical for flex containers with overflow */
}

</style>
