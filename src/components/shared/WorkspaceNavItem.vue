<template>
  <RouterLink v-if="workspace.enabled" :id="id" :to="workspace.path" v-slot="{ isActive }" class="w-full">
    <div
      class="flex flex-col items-center justify-center gap-1 px-2 py-4"
      :class="[
        isActive
          ? 'border-l-4 bg-surface-800/10 border-blue-500 text-surface-400 hover:text-surface-300'
          : 'border-l-4 bg-transparent border-surface-800/20 text-surface-500 hover:text-surface-200 hover:border-blue-400',
      ]">
      <fa :icon="['fal', workspace.icon]" size="xl" />
      <div class="text-xs">
        {{ workspace.label }}
      </div>
    </div>
  </RouterLink>
  <div v-else>
    <div
      class="flex flex-col items-center justify-center gap-1 px-2 py-4 bg-transparent text-surface-700">
      <fa :icon="['fal', workspace.icon]" size="xl" />
      <div class="text-xs">
        {{ workspace.label }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface Workspace {
  label: string;
  path: string;
  to: string;
  icon: string;
  enabled: boolean;
}

defineProps<{
  workspace: Workspace;
  id?: string;
}>();
</script>
