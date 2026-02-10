<template>
  <div class="min-h-screen bg-surface-0 text-surface-900">
    <AppHeader />
    <main class="max-w-5xl p-6 mx-auto">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import AppHeader from '@/components/AppHeader.vue';
</script>
