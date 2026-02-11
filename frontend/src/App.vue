<template>
  <div class="h-screen flex">
    <div class="w-20 h-full shrink-0 overflow-hidden">
      <NavBar />
    </div>
    <!-- :class="store.chatbotOpen ? 'w-220' : 'w-120'" -->
    <aside class="h-full shrink-0 transition-[width] duration-300 ease-in-out" >
      <RouterView name="menu" />
    </aside>
    <main class="h-full min-w-0 flex-1">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import NavBar from './layout/NavBar.vue';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();
</script>
