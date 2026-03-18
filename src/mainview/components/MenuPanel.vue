<template>
  <div class="h-full p-2">
    <div class="h-full bg-white rounded-xl shadow-lg flex overflow-hidden transition-all duration-300 ease-in-out">
      <!-- Left column: params (always visible) -->
      <div class="w-120 flex flex-col h-full">
        <!-- Header -->
        <div class="h-16 px-4 w-full content-end mb-4">
          <div class="flex items-center justify-between w-full border-b border-surface-300 pb-2">
            <div class="flex items-center gap-x-2">
              <component :is="resolvedIcon" :size="14" class="text-blue-500" />
              <span class="text-xl font-semibold text-slate-700">{{ title }}</span>
            </div>
            <slot name="title-button" />
          </div>

          <!-- :class="store.chatbotOpen ? 'bg-blue-500 text-white' : 'hover:bg-slate-100 text-slate-500'" -->
        </div>

        <!-- Scrollable params -->
        <div class="flex-1 overflow-y-auto px-4 pb-2 flex flex-col gap-4">
          <slot name="content" />
        </div>

        <!-- Generate button (pinned at bottom) -->
        <div v-if="$slots.footer" class="px-4 py-3 border-t border-slate-200 bg-surface-100 min-h-16 flex flex-col gap-2">
          <slot name="footer" />
        </div>
      </div>

      <!-- Right column: chatbot (conditional) -->
      <div v-if="$slots.secondary" class="flex-1 transition-[width] duration-300 ease-in-out" :class="expanded ? 'w-120 border-l border-slate-100 ' : 'w-0'">
        <!-- Content -->
        <slot name="secondary" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue';

const props = defineProps<{
  title: string;
  icon?: Component | string;
  expand?: boolean;
}>();

const resolvedIcon = computed(() => props.icon ?? 'ImageOff');

const expanded = computed(() => props.expand ?? false);
</script>
