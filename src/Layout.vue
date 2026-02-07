<template>
  <div class="flex flex-col w-full h-screen overflow-hidden">
    <div class="flex flex-row flex-1 min-h-0">
      <WorkspaceNav />
      <RouterView />
    </div>
    <div class="h-10 shrink-0">
      <StatusBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import StatusBar from '@/components/shared/StatusBar.vue';
import WorkspaceNav from '@/components/shared/WorkspaceNav.vue';
</script>
