<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-slate-100">
      <span class="text-xs font-semibold text-slate-700">AI Prompt Assistant</span>
      <span class="text-[10px] font-bold bg-blue-500 text-white px-1.5 py-0.5 rounded">AI</span>
    </div>

    <!-- Dropzones -->
    <div class="flex gap-1.5 px-3 py-2">
      <div class="flex-1 py-1.5 rounded border border-dashed border-purple-300 bg-purple-50 text-center">
        <span class="text-[10px] font-medium text-purple-600">Style</span>
      </div>
      <div class="flex-1 py-1.5 rounded border border-dashed border-blue-300 bg-blue-50 text-center">
        <span class="text-[10px] font-medium text-blue-600">Style+Subject</span>
      </div>
      <div class="flex-1 py-1.5 rounded border border-dashed border-green-300 bg-green-50 text-center">
        <span class="text-[10px] font-medium text-green-600">Subject</span>
      </div>
    </div>

    <!-- Chat messages -->
    <div class="flex-1 overflow-y-auto px-3 py-2 flex flex-col gap-3">
      <!-- AI message -->
      <div class="flex gap-2">
        <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-0.5">
          <Bot :size="14" class="text-white" />
        </div>
        <div class="bg-slate-50 rounded-lg px-3 py-2 text-xs text-slate-700 leading-relaxed">
          Hi! I can help you craft better prompts. Try describing your idea and I'll suggest improvements for style, composition, and detail.
        </div>
      </div>

      <!-- User message (demo) -->
      <div class="flex justify-end">
        <div class="bg-blue-500 rounded-lg px-3 py-2 text-xs text-white leading-relaxed max-w-[85%]">
          A cat sitting on a windowsill
        </div>
      </div>

      <!-- AI response (demo) -->
      <div class="flex gap-2">
        <div class="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center shrink-0 mt-0.5">
          <Bot :size="14" class="text-white" />
        </div>
        <div class="bg-slate-50 rounded-lg px-3 py-2 text-xs text-slate-700 leading-relaxed">
          Here's an enhanced version: <em>"A fluffy tabby cat lounging on a sun-drenched windowsill, warm golden hour light streaming through sheer curtains, soft bokeh of a garden visible through the glass, photorealistic, 8k detail"</em>
        </div>
      </div>
    </div>

    <!-- Suggestion chips -->
    <div class="flex gap-1.5 px-3 py-2 flex-wrap">
      <button
        v-for="chip in suggestionChips"
        :key="chip"
        class="text-[10px] px-2 py-1 rounded-full bg-slate-100 text-slate-600 hover:bg-slate-200 transition-colors">
        {{ chip }}
      </button>
    </div>

    <!-- Chat input -->
    <div class="px-3 py-2 border-t border-slate-100">
      <div class="flex gap-1.5">
        <input
          v-model="chatInput"
          type="text"
          placeholder="Ask for prompt help..."
          class="flex-1 text-xs bg-slate-100 rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-blue-300"
          @keydown.enter="onSend" />
        <button
          class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
          :disabled="!chatInput.trim()"
          @click="onSend">
          <SendHorizontal :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Bot, SendHorizontal } from 'lucide-vue-next';

const chatInput = ref('');
const suggestionChips = ['Lighting tips', 'Art styles', 'Enhance', 'More detail'];

function onSend() {
  if (!chatInput.value.trim()) return;
  // UI-only for now — no backend integration
  chatInput.value = '';
}
</script>
