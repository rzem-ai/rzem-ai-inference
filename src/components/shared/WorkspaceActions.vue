<template>
  <aside class="flex flex-col h-full overflow-hidden w-120 min-w-120">
    <!-- -->
    <div v-if="$slots.toolbar" class="shrink">
      <div class="p-2">
        <Toolbar>
          <template #center>
            <div class="flex gap-1">
              <slot name="toolbar"> </slot>
            </div>
          </template>
        </Toolbar>
      </div>
    </div>

    <!-- -->
    <div v-if="$slots.header" class="shrink">
      <div class="flex items-start justify-between p-0">
        <div class="flex flex-col gap-y-1">
          <h2 class="text-xl font-medium text-surface-50"><slot name="header"> </slot></h2>
        </div>
        <div class=""> </div>
      </div>
    </div>

    <!-- -->
    <div v-if="$slots.body" class="overflow-y-scroll grow">
      <slot name="body"> </slot>
    </div>

    <!-- -->
    <div v-if="$slots.footer" class="shrink">
      <slot name="footer"> </slot>
    </div>
  </aside>
</template>
<script setup lang="ts">
import Toolbar from 'primevue/toolbar';
</script>
