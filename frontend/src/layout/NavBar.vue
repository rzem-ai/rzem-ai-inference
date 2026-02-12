<template>
  <div class="flex flex-col w-full h-full gap-4">
    <div class="flex items-center justify-center pt-4 h-16 shrink">
      <PixelCard variant="blue" :gap="4" :speed="35" :no-focus="false"></PixelCard>
    </div>
    <div class="flex grow place-content-center">
      <div class="place-content-start flex flex-col gap-4">
        <RouterLink :to="{ name: 'create' }" class="" v-slot="{ isActive, href, navigate }">
          <div
            class="p-3 border border-surface-300 rounded-lg shadow transition-all duration-300 ease-in-out"
            :class="
              isActive
                ? 'text-blue-400 shadow-blue-400/50  hover:shadow-blue-500'
                : 'text-surface-400 hover:text-blue-500 shadow-surface-400/50  hover:shadow-blue-500'
            ">
            <WandSparkles :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'gallery' }" class="" v-slot="{ isActive, href, navigate }">
          <div
            class="p-3 border border-surface-300 rounded-lg shadow transition-all duration-300 ease-in-out"
            :class="
              isActive
                ? 'text-blue-400 shadow-blue-400/50  hover:shadow-blue-500'
                : 'text-surface-400 hover:text-blue-500 shadow-surface-400/50  hover:shadow-blue-500'
            ">
            <Images :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'styles' }" class="" v-slot="{ isActive, href, navigate }">
          <div
            class="p-3 border border-surface-300 rounded-lg shadow transition-all duration-300 ease-in-out"
            :class="
              isActive
                ? 'text-blue-400 shadow-blue-400/50  hover:shadow-blue-500'
                : 'text-surface-400 hover:text-blue-500 shadow-surface-400/50  hover:shadow-blue-500'
            ">
            <Palette :size="20" />
          </div>
        </RouterLink>
      </div>
    </div>
    <div class="flex items-center justify-center h-20 shrink">
      <RouterLink :to="{ name: 'settings' }" v-slot="{ isActive, href, navigate }">
        <div
          class="p-2 transition-all duration-300 ease-in-out"
          :class="isActive ? 'text-blue-400 hover:text-blue-500' : 'text-surface-400  hover:text-blue-500'">
          <Cog :size="20" />
        </div>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CameraOff, Cog, Images, Palette, WandSparkles } from 'lucide-vue-next';
import PixelCard from './PixelCard.vue';
import { isActive } from '@tiptap/core';
</script>
