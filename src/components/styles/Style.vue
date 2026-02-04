<template>
  <div class="flex flex-col items-center justify-center h-full p-8 text-center"> STYLE {{ route.name }} {{ route.params }}</div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();
</script>
