<template>
  <div class="w-full h-full bg-blue-500">
    <div>
      <span class="text-xl font-semibold">PyWebView + Vue 3</span>
    </div>
    <div>
      <div class="flex gap-2">
        <Button label="Home" icon="pi pi-home" severity="secondary" text @click="router.push('/')" />
        <Button label="About" icon="pi pi-info-circle" severity="secondary" text @click="router.push('/about')" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';

const router = useRouter();
</script>
