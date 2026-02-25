<template>
  <div
    class="group relative rounded-lg overflow-hidden cursor-pointer bg-white border border-slate-200 hover:border-slate-300 transition-colors"
    :class="{ 'ring-2 ring-blue-500 border-blue-500': selected }"
    @click="emit('click', styleData.id)"
    @contextmenu.prevent="emit('contextmenu', $event, styleData.id)">
    <div class="aspect-4/3 w-full relative bg-slate-50 flex items-center justify-center">
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
        class="absolute top-1 left-1 px-2 py-1 rounded-full bg-black/30 backdrop-blur-sm text-white text-xs font-medium">
        {{ styleData.category }}
      </div>

      <!-- Favorite button -->
      <button
        class="absolute top-1 right-1 p-1 rounded-full bg-black/30 backdrop-blur-sm
               opacity-0 group-hover:opacity-100 transition-opacity hover:bg-black/50"
        :class="{ 'opacity-100!': styleData.is_favorite }"
        @click.stop="emit('favorite', styleData.id)">
        <Star
          :size="14"
          class="text-white"
          :class="{ 'fill-yellow-400 text-yellow-400': styleData.is_favorite }" />
      </button>

      <!-- Base model badge -->
      <div
        v-if="bundleLabel"
        class="absolute bottom-1 right-1 rounded-full bg-black/30 px-2 py-1 text-xs font-medium text-white backdrop-blur-sm">
        {{ bundleLabel }}
      </div>

      <!-- Selection checkbox -->
      <div
        class="absolute bottom-1 left-1 opacity-0 group-hover:opacity-100 transition-opacity"
        :class="{ 'opacity-100!': selected }">
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
    <div class="p-2">
      <h3 class="text-sm font-medium text-slate-800 truncate">{{ styleData.name }}</h3>
      <p class="text-sm text-slate-400 mt-1 line-clamp-2">{{ styleData.prompt_template }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Star, Check, Palette } from 'lucide-vue-next';
import { usePywebview } from '@/composables/usePywebview';
import { useInferenceStore } from '@/stores/inference';
import type { Style } from '@/types/inference';

const props = defineProps<{
  styleData: Style;
  selected: boolean;
}>();

const emit = defineEmits<{
  click: [styleId: string];
  favorite: [styleId: string];
  select: [styleId: string];
  contextmenu: [event: MouseEvent, styleId: string];
}>();

const inferenceStore = useInferenceStore();
const { api, isReady } = usePywebview();
const thumbnailUrl = ref<string | null>(null);

const TYPE_LABELS: Record<string, string> = {
  flux1_dev: 'FLUX.1',
  flux2_dev: 'FLUX.2',
  z_image: 'Z-Image',
  qwen_image: 'Qwen',
  fal_cloud: 'FAL.ai',
};

const bundleLabel = computed(() => {
  if (!props.styleData.bundle_id) return null;
  const bundle = inferenceStore.bundles.find(b => b.id === props.styleData.bundle_id);
  if (!bundle) return null;
  return TYPE_LABELS[bundle.transformer_type] ?? bundle.transformer_type;
});

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
