<template>
  <div class="flex flex-col w-full h-full gap-4">
    <div class="flex items-center justify-center pt-4 h-16 shrink">
      <PixelCard variant="blue" :gap="4" :speed="35" :no-focus="false"></PixelCard>
    </div>
    <div class="flex grow place-content-center">
      <div class="place-content-start flex flex-col gap-4">
        <RouterLink :to="{ name: 'create' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <ImagePlus :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'edit' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <PenTool :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'gallery' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Images :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'styles' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Palette :size="20" />
          </div>
        </RouterLink>
        <RouterLink :to="{ name: 'models' }" v-slot="{ isActive }">
          <div
            class="p-3 transition-all duration-300 ease-in-out"
            :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
            <Box :size="20" />
          </div>
        </RouterLink>
      </div>
    </div>
    <div class="flex items-center justify-center h-20 shrink">
      <RouterLink :to="{ name: 'settings' }" v-slot="{ isActive }">
        <div
          class="p-3 transition-all duration-300 ease-in-out"
          :class="isActive ? 'text-white' : 'text-surface-400 hover:text-white'">
          <Cog :size="20" />
        </div>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import PixelCard from '../components/PixelCard.vue';
</script>
