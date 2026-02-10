<template>
  <div class="h-screen flex gap-0 ">
    <div class="w-20 h-full col-span-1 overflow-hidden">
      <NavBar />
    </div>
    <aside class="col-span-1 w-80 h-full">
      <RouterView name="menu"/>
    </aside>
    <main class="col-span-1 h-full">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import NavBar from './layout/NavBar.vue';
</script>
