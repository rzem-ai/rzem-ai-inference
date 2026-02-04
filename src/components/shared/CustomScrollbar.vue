<template>
  <PerfectScrollbar :tag="tag" :options="options" class="custom-scrollbar">
    <slot />
  </PerfectScrollbar>
</template>

<script setup lang="ts">
import { PerfectScrollbar } from 'vue3-perfect-scrollbar';
 

/**
 * CustomScrollbar - A wrapper around vue3-perfect-scrollbar
 *
 * Provides consistent cross-platform scrollbar appearance and fixes
 * GTK overlay scrollbar z-index issues on Linux.
 *
 * Usage:
 *   <CustomScrollbar class="h-full">
 *     <div>Your scrollable content</div>
 *   </CustomScrollbar>
 *
 * Note: The parent container must have a defined height for scrolling to work.
 */

withDefaults(
  defineProps<{
    /** HTML tag to render (default: 'div') */
    tag?: string;
    /** PerfectScrollbar options */
    options?: Record<string, unknown>;
  }>(),
  {
    tag: 'div',
    options: () => ({
      suppressScrollX: true,
      wheelPropagation: false,
    }),
  },
);
</script>

<style scoped>
@reference "tailwindcss";

.custom-scrollbar {
  position: relative;
  height: 100%;
}
 
 
</style>
