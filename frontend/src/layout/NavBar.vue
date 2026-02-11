<template>
  <div class="flex flex-col w-full h-full gap-4">
    <div class="flex items-center justify-center h-20 shrink">
      <div class="p-2 bg-blue-200 border border-blue-600 rounded shadow text-surface-800"><CameraOff :size="30" /></div>
    </div>
    <div class="flex grow place-content-center">
      <div class="place-content-start flex flex-col gap-2">
        <RouterLink :to="{ name: 'create' }" class="">
          <div class="p-2 border border-blue-300 rounded shadow text-surface-800">
            <WandSparkles :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'gallery' }" class="">
          <div class="p-2 border border-blue-300 rounded shadow text-surface-800">
            <Images :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'styles' }" class="">
          <div class="p-2 border border-blue-300 rounded shadow text-surface-800">
            <Palette :size="20" />
          </div>
        </RouterLink>
      </div>
    </div>
    <div class="flex items-center justify-center h-20 shrink">
      <div class="p-2 text-surface-600"><Cog :size="30" /></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CameraOff, Cog, Images, Palette, WandSparkles } from 'lucide-vue-next';
</script>
