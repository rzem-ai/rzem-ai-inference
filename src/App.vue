<template>
  <div class="app-container">
    <Toast />
    <WorkspaceNav />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router';
import Toast from 'primevue/toast';
import WorkspaceNav from '@/components/shared/WorkspaceNav.vue';
import { useAppInit } from '@/composables/useAppInit';

// Initialize app on mount
useAppInit();
</script>
