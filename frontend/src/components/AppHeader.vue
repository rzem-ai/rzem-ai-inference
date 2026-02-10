<script setup lang="ts">
import Toolbar from "primevue/toolbar";
import Button from "primevue/button";
import { useRouter } from "vue-router";

const router = useRouter();
</script>

<template>
  <Toolbar class="border-0 border-b border-surface-200 rounded-none">
    <template #start>
      <span class="text-xl font-semibold">PyWebView + Vue 3</span>
    </template>
    <template #end>
      <div class="flex gap-2">
        <Button
          label="Home"
          icon="pi pi-home"
          severity="secondary"
          text
          @click="router.push('/')"
        />
        <Button
          label="About"
          icon="pi pi-info-circle"
          severity="secondary"
          text
          @click="router.push('/about')"
        />
      </div>
    </template>
  </Toolbar>
</template>
