<template>
  <!-- History strip -->
  <Card class="min-h-30">
    <template #content>
      <div class="flex gap-1.5 overflow-x-auto" v-if="store.generatedImages.length">
        <div
          v-for="(img, index) in store.generatedImages"
          :key="img.jobId"
          class="shrink-0 w-20 h-20 rounded cursor-pointer overflow-hidden border-2 transition-colors"
          :class="index === store.selectedImageIndex ? 'border-blue-500' : 'border-transparent hover:border-slate-300'"
          draggable="true"
          @dragstart="(e) => e.dataTransfer?.setData('text/image-path', img.imagePath)"
          @click="store.selectImage(index)">
          <img v-if="img.dataUrl" :src="img.dataUrl" alt="" draggable="false" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full bg-slate-200 flex items-center justify-center">
            <ImageIcon :size="10" class="text-slate-400" />
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { Image as ImageIcon } from 'lucide-vue-next';
import { useInferenceStore } from '@/stores/inference';
import { Card } from 'primevue';

const store = useInferenceStore();
</script>
