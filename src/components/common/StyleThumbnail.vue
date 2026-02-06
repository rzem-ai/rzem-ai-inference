<template>
  <div
    class="style-thumbnail"
    :class="[sizeClass, { 'is-dragging': isDragging }]"
    @click="handleClick"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <img
      v-if="thumbnailPath"
      :src="thumbnailPath"
      :alt="alt || 'Thumbnail'"
      class="object-cover w-full h-full"
    />
    <fa v-else :icon="['fal', 'palette']" size="lg" class="text-surface-500" />

    <!-- Drag overlay -->
    <div v-if="isDragging" class="style-thumbnail__overlay">
      <fa :icon="['fal', 'upload']" size="2x" class="text-primary-400" />
      <span class="mt-2 text-sm font-medium text-primary-400">Drop image here</span>
    </div>

    <!-- Hidden file input -->
    <input
      ref="fileInputRef"
      type="file"
      accept="image/*"
      @change="handleFileChange"
      class="hidden"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Props {
  thumbnailPath?: string | null;
  alt?: string;
  size?: 'small' | 'medium' | 'large';
}

const props = withDefaults(defineProps<Props>(), {
  alt: 'Thumbnail',
  size: 'medium',
});

const emit = defineEmits<{
  upload: [file: File];
}>();

const isDragging = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);

const sizeClass = computed(() => {
  switch (props.size) {
    case 'small':
      return 'w-12 h-12';
    case 'medium':
      return 'w-20 h-20';
    case 'large':
      return 'w-32 h-32';
    default:
      return 'w-20 h-20';
  }
});

function handleClick() {
  fileInputRef.value?.click();
}

function handleDragOver(event: DragEvent) {
  // Check if dragged item contains files
  if (event.dataTransfer?.types.includes('Files')) {
    isDragging.value = true;
  }
}

function handleDragLeave() {
  isDragging.value = false;
}

function handleDrop(event: DragEvent) {
  isDragging.value = false;

  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.type.startsWith('image/')) {
      emit('upload', file);
    }
  }
}

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];

  if (file) {
    emit('upload', file);
    // Reset input so same file can be selected again
    target.value = '';
  }
}
</script>

<style scoped>
@reference "tailwindcss";

.style-thumbnail {
  @apply relative flex items-center justify-center overflow-hidden rounded shrink-0 bg-gray-800;
  @apply cursor-pointer transition-all duration-200;
  @apply hover:ring-2 hover:ring-blue-500/50 hover:shadow-lg;
}

.style-thumbnail.is-dragging {
  @apply ring-2 ring-blue-400 shadow-lg scale-105;
}

.style-thumbnail__overlay {
  @apply absolute inset-0 flex flex-col items-center justify-center;
  @apply bg-gray-900/95 backdrop-blur-sm;
}
</style>
