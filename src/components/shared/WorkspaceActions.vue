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
      <div class="p-2">
        <slot name="toolbar"> </slot>
      </div>
    </div>

    <!-- -->
    <div v-if="$slots.body" class="overflow-y-scroll grow">
      <slot name="body"> </slot>
    </div>

    <!-- -->
    <div v-if="$slots.footer" class="shrink">
      <div class="border-t border-surface-600"></div>
      <slot name="footer"> </slot>
    </div>
  </aside>
</template>
<script setup lang="ts">

</script>
