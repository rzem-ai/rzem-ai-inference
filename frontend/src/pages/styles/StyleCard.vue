<template>
  <div
    class="group relative rounded-lg overflow-hidden cursor-pointer bg-white border border-slate-200 hover:border-slate-300 transition-colors"
    :class="{ 'ring-2 ring-blue-500 border-blue-500': selected }"
    @click="emit('click', styleData.id)">
    <div class="aspect-[4/3] w-full relative bg-slate-50 flex items-center justify-center">
      <!-- Thumbnail image -->
      <img
        v-if="thumbnailUrl"
        :src="thumbnailUrl"
        class="absolute inset-0 w-full h-full object-cover" />

      <!-- Placeholder icon -->
      <Palette v-else :size="32" class="text-slate-300" />

      <!-- Category badge -->
      <div
        v-if="styleData.category"
        class="absolute top-1.5 left-1.5 px-2 py-0.5 rounded-full bg-black/30 backdrop-blur-sm text-white text-[10px] font-medium">
        {{ styleData.category }}
      </div>

      <!-- Favorite button -->
      <button
        class="absolute top-1.5 right-1.5 p-1 rounded-full bg-black/30 backdrop-blur-sm
               opacity-0 group-hover:opacity-100 transition-opacity hover:bg-black/50"
        :class="{ '!opacity-100': styleData.is_favorite }"
        @click.stop="emit('favorite', styleData.id)">
        <Star
          :size="14"
          class="text-white"
          :class="{ 'fill-yellow-400 text-yellow-400': styleData.is_favorite }" />
      </button>

      <!-- Selection checkbox -->
      <div
        class="absolute bottom-1.5 left-1.5 opacity-0 group-hover:opacity-100 transition-opacity"
        :class="{ '!opacity-100': selected }">
        <button
          class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors"
          :class="selected
            ? 'bg-blue-500 border-blue-500'
            : 'bg-white/80 border-white/60 hover:border-white'"
          @click.stop="emit('select', styleData.id)">
          <Check v-if="selected" :size="12" class="text-white" />
        </button>
      </div>
    </div>

    <!-- Info section -->
    <div class="p-2.5">
      <h3 class="text-sm font-medium text-slate-800 truncate">{{ styleData.name }}</h3>
      <p class="text-[11px] text-slate-400 mt-0.5 line-clamp-2">{{ styleData.prompt_template }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { Star, Check, Palette } from 'lucide-vue-next';
import { usePywebview } from '@/composables/usePywebview';
import type { Style } from '@/types/inference';

const props = defineProps<{
  styleData: Style;
  selected: boolean;
}>();

const emit = defineEmits<{
  click: [styleId: string];
  favorite: [styleId: string];
  select: [styleId: string];
}>();

const { api, isReady } = usePywebview();
const thumbnailUrl = ref<string | null>(null);

watch([() => props.styleData.thumbnail_path, isReady], async ([path, ready]) => {
  if (!path) {
    thumbnailUrl.value = null;
    return;
  }
  if (!ready) return;
  const res = await api.value.get_image_base64({ image_path: path });
  if (res.status === 'success' && res.data_url) {
    thumbnailUrl.value = res.data_url;
  }
}, { immediate: true });
</script>
