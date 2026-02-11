<template>
  <div class="flex flex-col w-full h-full gap-4">
    <div class="flex items-center justify-center pt-4 h-16 shrink">
      <PixelCard variant="default" :gap="4" :speed="35" colors="#f8fafc,#f1f5f9,#cbd5e1" :no-focus="false"></PixelCard>
    </div>
    <div class="flex grow place-content-center">
      <div class="place-content-start flex flex-col gap-4">
        <RouterLink :to="{ name: 'create' }" class="">
          <div class="p-3 border border-surface-300 rounded-lg shadow text-surface-600">
            <WandSparkles :size="20" class="text-surface-500" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'gallery' }" class="">
          <div class="p-3 border border-surface-300 rounded-lg shadow text-surface-600">
            <Images :size="20" class="text-surface-500" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'styles' }" class="">
          <div class="p-3 border border-surface-300 rounded-lg shadow text-surface-600">
            <Palette :size="20" class="text-surface-500" />
          </div>
        </RouterLink>
      </div>
    </div>
    <div class="flex items-center justify-center h-20 shrink">
      <RouterLink :to="{ name: 'settings' }">
        <div class="p-2 text-surface-600 hover:text-surface-800 transition-colors">
          <Cog :size="20" class="text-surface-500" />
        </div>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CameraOff, Cog, Images, Palette, WandSparkles } from 'lucide-vue-next';
import PixelCard from './PixelCard.vue';
</script>
