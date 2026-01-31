<template>
  <div class="flex w-full">
    <!-- Sidebar -->
    <WorkspaceActions>
      <template #toolbar></template>
      <template #header>
        <div>Settings </div>
        <div class="text-sm font-light">Configure application settings and integrations</div>
      </template>
      <template #body>
        <ul>
          <li> <RouterLink :to="{name: 'apikeys'}"> API Keys </RouterLink></li>
          <li> <RouterLink :to="{name: 'cache'}"> Cache </RouterLink></li>
          <li></li>
        </ul>
      </template>
      <template #footer></template>
    </WorkspaceActions>

    <RouterView />
  </div>
</template>

<script setup lang="ts">
import WorkspaceActions from '@/components/shared/WorkspaceActions.vue';
</script>

<style scoped>
@reference "tailwindcss";
</style>
