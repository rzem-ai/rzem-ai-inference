<template>
  <div class="flex flex-col h-full px-2 py-4 gap-2">
    <CompareView class="flex-1" />
    <History />
  </div>
</template>

<script setup lang="ts">
import CompareView from './CompareView.vue';
import History from './History.vue';
</script>
