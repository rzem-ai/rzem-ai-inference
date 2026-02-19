<template>
  <!-- Error overlay -->
  <div v-if="store.error && !store.isGenerating" class="absolute bottom-0 left-0 right-0 bg-red-50/90 backdrop-blur-sm px-4 py-3 flex items-center gap-2">
    <AlertCircle :size="16" class="text-red-500 shrink-0" />
    <span class="text-xs text-red-600 flex-1">{{ store.error }}</span>
    <button class="text-xs text-red-500 hover:text-red-700 underline shrink-0" @click="store.setError(null)"> Dismiss </button>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
</script>
