<template>
  <div class="h-full flex flex-col px-2 py-4 gap-2">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useStylesStore } from '@/stores/styles';

const stylesStore = useStylesStore();

onMounted(() => {
  Promise.all([stylesStore.loadStyles(), stylesStore.loadCategories(), stylesStore.loadTags(), stylesStore.loadLoras()]);
});
</script>
