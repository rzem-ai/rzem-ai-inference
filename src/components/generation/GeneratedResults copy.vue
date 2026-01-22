<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs font-semibold tracking-wider text-(--text-secondary)">GENERATED RESULTS</span>
      <span class="text-xs text-(--accent-primary)">{{ images.length }} {{ images.length === 1 ? 'image' : 'images' }}</span>
    </div>

    <div v-if="images.length === 0" class="flex items-center justify-center flex-1">
      <div class="flex flex-col items-center gap-3 text-(--text-secondary)">
        <i class="text-5xl pi pi-image"></i>
        <div class="text-center">
          <p class="text-sm font-medium">No images generated yet</p>
          <p class="text-xs text-(--text-muted)">Generated images will appear here</p>
        </div>
      </div>
    </div>

    <div v-else class="flex-1 min-h-0 grid gap-4" :class="gridClasses">
      <div
        v-for="(image, index) in images"
        :key="image.id"
        class="relative flex items-center justify-center overflow-hidden rounded-xl border-2 border-solid border-(--accent-primary) bg-(--bg-element) group"
        :class="getImageWrapperClass(index)">
        <Image :src="image.src" alt="Generated image" preview image-class="slot-image" @load="(e: Event) => handleImageLoad(e, index)" />
        <Button
          icon="pi pi-download"
          class="absolute right-3 top-3 z-10 opacity-0 transition-opacity duration-200 ease-in-out backdrop-blur-lg group-hover:opacity-100 bg-[rgba(212,168,83,0.9)!important] border-[rgba(212,168,83,0.9)!important] hover:bg-[rgba(196,154,58,0.95)!important] hover:border-[rgba(196,154,58,0.95)!important]"
          rounded
          severity="secondary"
          @click.stop="handleDownload(image.src, index + 1)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import Image from 'primevue/image';
import Button from 'primevue/button';

interface GeneratedImage {
  id: string;
  src: string;
  width?: number;
  height?: number;
  isPortrait?: boolean;
  isLandscape?: boolean;
  aspectRatio?: number;
}

interface Props {
  images: GeneratedImage[];
}

interface Emits {
  (e: 'download', src: string, index: number): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Track image dimensions as they load
const imageDimensions = ref<Map<number, { width: number; height: number }>>(new Map());

// Handle image load to get dimensions
const handleImageLoad = (event: Event, index: number) => {
  const img = event.target as HTMLImageElement;
  if (img.naturalWidth && img.naturalHeight) {
    imageDimensions.value.set(index, {
      width: img.naturalWidth,
      height: img.naturalHeight,
    });
  }
};

// Analyze images to determine orientations
const analyzedImages = computed(() => {
  return props.images.map((image, index) => {
    const dims = imageDimensions.value.get(index);
    if (!dims) {
      // Default to landscape if not loaded yet
      return {
        ...image,
        isPortrait: false,
        isLandscape: true,
        isSquare: false,
        aspectRatio: 16 / 9,
      };
    }

    const isPortrait = dims.height > dims.width;
    const isLandscape = dims.width > dims.height;
    const isSquare = Math.abs(dims.width - dims.height) < dims.width * 0.1;
    const aspectRatio = dims.width / dims.height;

    return {
      ...image,
      isPortrait,
      isLandscape,
      isSquare,
      aspectRatio,
    };
  });
});

// Determine layout based on image count and orientations
const layoutType = computed(() => {
  const count = analyzedImages.value.length;
  const portraitCount = analyzedImages.value.filter((img) => img.isPortrait).length;
  const landscapeCount = analyzedImages.value.filter((img) => img.isLandscape).length;
  const allPortrait = portraitCount === count;
  const allLandscape = landscapeCount === count;
  const majorityPortrait = portraitCount > count / 2;

  switch (count) {
    case 1:
      return 'single';

    case 2:
      if (allPortrait) {
        return 'portrait-row';
      } else if (allLandscape) {
        return 'landscape-stack';
      } else {
        return 'mixed-stack';
      }

    case 3:
      if (allPortrait) {
        return 'portrait-row';
      } else if (majorityPortrait) {
        return 'two-one'; // 2 on top, 1 on bottom
      } else if (allLandscape) {
        return 'landscape-stack';
      } else {
        return 'one-two'; // 1 on top, 2 on bottom
      }

    case 4:
      if (allPortrait) {
        return 'portrait-grid';
      } else if (allLandscape) {
        return 'landscape-grid';
      } else {
        return 'mixed-grid';
      }

    default:
      // For more than 4 images, use a simple grid
      return 'multi-grid';
  }
});

// Compute grid classes based on layout type
const gridClasses = computed(() => {
  const count = props.images.length;
  const layout = layoutType.value;

  const classes: string[] = [];

  // Single image - centered
  if (layout === 'single') {
    classes.push('grid-cols-1', 'place-items-center');
  }
  // Two images
  else if (count === 2) {
    if (layout === 'portrait-row') {
      classes.push('grid-cols-1', 'md:grid-cols-2');
    } else {
      classes.push('grid-cols-1');
    }
  }
  // Three images
  else if (count === 3) {
    if (layout === 'portrait-row') {
      classes.push('grid-cols-1', 'md:grid-cols-3');
    } else if (layout === 'two-one' || layout === 'one-two') {
      classes.push('grid-cols-1', 'md:grid-cols-2');
    } else {
      classes.push('grid-cols-1');
    }
  }
  // Four images
  else if (count === 4) {
    if (layout === 'portrait-grid' || layout === 'landscape-grid' || layout === 'mixed-grid') {
      classes.push('grid-cols-1', 'md:grid-cols-2');
    }
  }
  // More than 4 images
  else {
    classes.push('grid-cols-1', 'md:grid-cols-2', 'lg:grid-cols-3');
  }

  return classes;
});

// Get wrapper classes for specific images based on layout
const getImageWrapperClass = (index: number) => {
  const count = props.images.length;
  const layout = layoutType.value;

  // Single image gets max width
  if (layout === 'single') {
    return 'max-w-3xl w-full';
  }

  // Three images with two-one layout
  if (count === 3 && layout === 'two-one' && index === 2) {
    return 'md:col-span-2';
  }

  // Three images with one-two layout
  if (count === 3 && layout === 'one-two' && index === 0) {
    return 'md:col-span-2';
  }

  return '';
};

const handleDownload = (src: string, index: number) => {
  emit('download', src, index);
};

// Reset dimensions when images change
watch(
  () => props.images.length,
  () => {
    imageDimensions.value.clear();
  },
);
</script>


