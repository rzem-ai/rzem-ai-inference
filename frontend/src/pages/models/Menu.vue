<template>
  <MenuPanel title="Models" icon="Box">
    <template #title-button>
      <Button class="transition-colors" severity="secondary" title="" disabled>
        <Plus :size="16" />
      </Button>
    </template>
    <template #content>
      <!-- Search -->
      <div>
        <InputGroup>
          <InputGroupAddon>
            <Search :size="14" class="text-slate-400" />
          </InputGroupAddon>
          <InputText
            v-model="modelsStore.searchQuery"
            placeholder="Search models..."
            class="w-full"
            size="small" />
        </InputGroup>
      </div>

      <!-- Favorites section -->
      <div v-if="modelsStore.favoriteBundles.length" class="flex flex-col gap-1">
        <div class="flex h-8 items-center px-2">
          <div class="font-medium text-slate-900">Favorites</div>
        </div>
        <button
          v-for="bundle in modelsStore.favoriteBundles"
          :key="bundle.id"
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="modelsStore.selectedBundleId === bundle.id ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50'"
          @click="modelsStore.selectBundle(bundle.id)">
          <Star :size="14" class="text-amber-400 fill-amber-400 shrink-0" />
          <span class="flex-1 truncate font-medium">{{ bundle.label }}</span>
        </button>
      </div>

      <!-- Grouped bundles -->
      <div v-for="group in modelsStore.bundlesByType" :key="group.type" class="flex flex-col gap-1">
        <div class="flex h-8 items-center gap-2 px-2">
          <Cloud v-if="group.icon === 'cloud'" :size="14" class="text-blue-500" />
          <Gpu v-else :size="14" class="text-blue-500" />
          <div class="font-medium text-slate-900">{{ group.label }}</div>
        </div>
        <button
          v-for="bundle in group.items"
          :key="bundle.id"
          class="flex h-8 w-full items-center gap-2 rounded-lg px-3 text-left transition-colors"
          :class="[
            modelsStore.selectedBundleId === bundle.id ? 'bg-blue-50 text-blue-600' : 'text-slate-700 hover:bg-slate-50',
            bundle.hidden ? 'opacity-50' : '',
          ]"
          @click="modelsStore.selectBundle(bundle.id)">
          <EyeOff v-if="bundle.hidden" :size="14" class="text-slate-400 shrink-0" />
          <Star v-else-if="bundle.favorite" :size="14" class="text-amber-400 fill-amber-400 shrink-0" />
          <span v-else class="w-3.5 shrink-0" />
          <span class="flex-1 truncate font-medium">{{ bundle.label }}</span>
          <Tag v-if="bundle.source === 'cloud'" severity="info" class="shrink-0">Cloud</Tag>
        </button>
      </div>
    </template>
  </MenuPanel>
</template>

<script setup lang="ts">
import { useModelsStore } from '@/stores/models';
import MenuPanel from '@/components/MenuPanel.vue';

const modelsStore = useModelsStore();
</script>
