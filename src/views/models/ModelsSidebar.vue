<template>
  <nav class="flex flex-col gap-1 p-2">
    <router-link
      to="/models/bundles"
      class="flex items-center gap-2 px-3 py-2 rounded text-sm text-surface-300 hover:bg-surface-800 transition-colors"
      active-class="bg-surface-800 text-surface-100"
    >
      <fa :icon="['fal', 'box-taped']" />
      Bundles
    </router-link>
    <router-link
      to="/models/models"
      class="flex items-center gap-2 px-3 py-2 rounded text-sm text-surface-300 hover:bg-surface-800 transition-colors"
      active-class="bg-surface-800 text-surface-100"
    >
      <fa :icon="['fal', 'microchip-ai']" />
      Models
    </router-link>
  </nav>
</template>

<script setup lang="ts">
</script>
