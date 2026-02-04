<template>
  <div
    class="flex items-center gap-3 p-3 transition-all border rounded-lg border-surface-700 hover:border-primary-500 hover:bg-surface-800/50 group"
    :class="{ 'bg-surface-800/30': isSelected }"
    @click="gotoStyle()">
    <!-- Checkbox -->

    <!-- Thumbnail -->
    <div class="flex items-center justify-center w-12 h-12 overflow-hidden rounded shrink-0 bg-surface-700">
      <img v-if="style.thumbnailPath" :src="style.thumbnailPath" :alt="style.name" class="object-cover w-full h-full" />
      <fa v-else :icon="['fal', 'palette']" size="lg" class="text-surface-500" />
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0 cursor-pointer">
      <div class="flex items-center gap-2 mb-1">
        <h3 class="text-sm font-semibold truncate text-surface-100">{{ style.name }}</h3>
        <Tag v-if="style.category" severity="secondary" :value="style.category" class="text-xs" />
      </div>
      <p v-if="style.description" class="text-xs truncate text-surface-400">
        {{ style.description }}
      </p>
    </div>

    <!-- Metadata -->
    <div class="flex items-center gap-3 ml-auto text-xs shrink-0 text-surface-500">
      <span class="flex items-center gap-1" :title="`Used ${style.usageCount} times`">
        <fa :icon="['fal', 'chart-line']" size="sm" />
        {{ style.usageCount }}
      </span>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-1 shrink-0">
      <!-- Favorite button -->
      <Button
        @click.stop="toggleFavorite()"
        :severity="style.isFavorite ? 'warning' : 'secondary'"
        variant="text"
        size="small"
        class="px-2! py-1!"
        :title="style.isFavorite ? 'Remove from favorites' : 'Add to favorites'">
        <fa :icon="style.isFavorite ? ['fas', 'star'] : ['fal', 'star']" size="sm" />
      </Button>

      <!-- Edit/Delete buttons (visible on hover) -->
      <div class="flex gap-1 transition-opacity opacity-0 group-hover:opacity-100">
        <Button @click.stop="edit()" severity="secondary" variant="text" size="small" class="px-2! py-1!" title="Edit">
          <fa :icon="['fal', 'pen']" size="sm" />
        </Button>
        <Button @click.stop="deleteStyle()" severity="danger" variant="text" size="small" class="px-2! py-1!" title="Delete">
          <fa :icon="['fal', 'trash']" size="sm" />
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { StyleInfo } from '@/types';

import Button from 'primevue/button';
import Tag from 'primevue/tag';
import { useRouter } from 'vue-router';

const router = useRouter();

const { style } = defineProps<{
  style: StyleInfo;
}>();

const isSelected = computed(() => {
  return false;
});

const gotoStyle = function () {
  console.log('goto:', `/styles/style/${style.id}`);
  router.push({ name: `styles-style`, params: { id: style.id } });
};

const toggleFavorite = function () {
  // TODO: Implement toggle favorite logic
  console.log('toggleFavorite:', style.id);
};

const edit = function () {
  // TODO: Implement edit logic
  console.log('edit:', style.id);
};

const deleteStyle = function () {
  // TODO: Implement delete logic
  console.log('delete:', style.id);
};
</script>
