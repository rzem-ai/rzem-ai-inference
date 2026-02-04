<template>
  <div class="flex flex-col h-full gap-4" v-if="style">
    <!-- Header -->
    <div>
      <div class="flex items-start justify-between mb-2">
        <Inplace class="w-full">
          <template #display>
            <div class="inline-flex items-center w-full gap-2">
              <div class="w-full text-lg font-semibold text-surface-100">{{ style.name || 'Click to Edit' }} </div>
              <div class="w-20"></div>
            </div>
          </template>
          <template #content="{ closeCallback }">
            <div class="inline-flex items-center w-full gap-2">
              <InputText v-model="style.name" autofocus fluid />
              <Button icon="pi pi-times" text severity="danger" @click="closeCallback" />
            </div>
          </template>
        </Inplace>

        <fa :icon="['fas', 'star']" class="text-yellow-400" />
      </div>

      <Inplace class="w-full">
        <template #display>
          <div class="inline-flex items-center w-full gap-2">
            <div class="w-full text-base font-normal text-surface-100">{{ style.description || 'Click to Edit' }} </div>
            <div class="w-20"></div>
          </div>
        </template>
        <template #content="{ closeCallback }">
          <div class="inline-flex items-center w-full gap-2">
            <InputText v-model="style.description" autofocus fluid />
            <Button icon="pi pi-times" text severity="danger" @click="closeCallback" />
          </div>
        </template>
      </Inplace>

      <div class="flex items-center gap-2 mt-2 text-xs text-surface-500">
        <span v-if="style.category" class="px-2 py-1 rounded bg-surface-800">
          {{ style.category }}
        </span>
        <span class="flex items-center gap-1">
          <fa :icon="['fal', 'chart-line']" size="sm" />
          Used {{ style.usageCount }} times
        </span>
      </div>
    </div>

    <Divider />

    <!-- Thumbnail -->
    <div>
      <h5 class="mb-2 text-sm font-semibold text-surface-200">Thumbnail</h5>
      <div class="flex items-center gap-3">
        <!-- Thumbnail preview -->
        <div class="flex items-center justify-center w-12 h-12 overflow-hidden rounded shrink-0 bg-surface-800">
          <img v-if="style.thumbnailPath" :src="style.thumbnailPath" :alt="style.name" class="object-cover w-full h-full" />
          <fa v-else :icon="['fal', 'palette']" size="lg" class="text-surface-500" />
        </div>

        <!-- Actions -->
        <div class="flex gap-2">
          <Button @click="handleUploadClick" severity="secondary" size="small">
            <fa :icon="['fal', 'upload']" size="sm" class="mr-2" />
            Upload
          </Button>
          <Button v-if="style.thumbnailPath" severity="danger" variant="outlined" size="small">
            <fa :icon="['fal', 'trash']" size="sm" class="mr-2" />
            Remove
          </Button>
        </div>

        <!-- Hidden file input -->
        <input ref="fileInput" type="file" accept="image/*" @change="handleFileChange" class="hidden" />
      </div>
    </div>

    <Divider />

    <!-- Template -->
    <div>
      <h5 class="mb-2 text-sm font-semibold text-surface-200">Prompt Template</h5>
      <div class="p-3 font-mono text-sm rounded bg-surface-800 text-surface-300">
        {{ style.promptTemplate }}
      </div>
      <p class="mt-1 text-xs text-surface-500">
        Use <code class="px-1 rounded bg-surface-800">&lcub;&lcub;prompt&rcub;&rcub;</code> as placeholder for your prompt
      </p>
    </div>

    <Divider />

    <!-- LoRAs -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <h5 class="text-sm font-semibold text-surface-200">LoRAs ({{ style.loras.length }})</h5>
        <Button severity="secondary" variant="text" size="small">
          <fa :icon="['fal', 'plus']" size="sm" />
        </Button>
      </div>
      <div v-if="style.loras.length === 0" class="py-4 text-sm text-center text-surface-500"> No LoRAs added </div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="lora in style.loras" :key="lora.loraId" class="flex items-center justify-between p-2 rounded bg-surface-800">
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate text-surface-100">{{ lora.loraName }}</p>
            <p v-if="lora.loraTriggerWords" class="text-xs truncate text-surface-500">
              {{ lora.loraTriggerWords }}
            </p>
          </div>
          <div class="flex items-center gap-2 ml-2">
            <span class="text-xs text-surface-400">{{ lora.strength.toFixed(2) }}</span>
            <Button severity="danger" variant="text" size="small" class="!p-1">
              <fa :icon="['fal', 'times']" size="sm" />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <Divider />

    <!-- Examples -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <h5 class="text-sm font-semibold text-surface-200">Examples ({{ style.examples.length }})</h5>
        <Button @click="emit('addExample')" severity="secondary" variant="text" size="small">
          <fa :icon="['fal', 'plus']" size="sm" />
        </Button>
      </div>
      <div v-if="style.examples.length === 0" class="py-4 text-sm text-center text-surface-500"> No examples added </div>
      <div v-else class="flex flex-col gap-2">
        <div v-for="example in style.examples" :key="example.id" class="relative p-2 rounded bg-surface-800 group">
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <span class="text-xs font-medium text-primary-400">
                {{ example.exampleType === 'prompt' ? 'Prompt' : 'Image' }}
              </span>
              <p class="mt-1 text-sm break-words text-surface-300">
                {{ example.content }}
              </p>
            </div>
            <Button @click="emit('removeExample', example.id)" severity="danger" variant="text" size="small" class="!p-1 opacity-0 group-hover:opacity-100">
              <fa :icon="['fal', 'times']" size="sm" />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex gap-2">
      <Button @click="emit('delete')" severity="danger" variant="outlined" size="small" class="flex-1">
        <fa :icon="['fal', 'trash']" size="sm" class="mr-2" />
        Delete
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import type { StyleDetail } from '@/types';
import { useStylesStore } from '@/stores/styles';
import Button from 'primevue/button';
import Divider from 'primevue/divider';
import Inplace from 'primevue/inplace';
import InputText from 'primevue/inputtext';
import { useRoute } from 'vue-router';

const emit = defineEmits<{
  addExample: [];
  removeExample: [id: string];
  delete: [];
}>();

const stylesStore = useStylesStore();
const route = useRoute();

const style = ref<StyleDetail>({
  loras: [],
  examples: [],
  id: '',
  name: '',
  promptTemplate: '',
  defaultStrength: 0,
  strengthMin: 0,
  strengthMax: 0,
  isFavorite: false,
  usageCount: 0,
  createdAt: 0,
  updatedAt: 0,
});

const fileInput = ref<HTMLInputElement | null>(null);

function handleUploadClick() {
  fileInput.value?.click();
}

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    // Reset input so same file can be selected again
    target.value = '';
  }
}

async function loadStyle() {
  if (route.params.id) {
    const styleId = Array.isArray(route.params.id) ? route.params.id[0] : route.params.id;
    console.log('loading style:', styleId);
    await stylesStore.loadStyleDetail(styleId);

    if (stylesStore.selectedStyle) {
      style.value = stylesStore.selectedStyle;
    }
  }
}

// Load on mount
onMounted(() => {
  loadStyle();
});

// Reload when route param changes (component instance is reused)
watch(
  () => route.params.id,
  () => {
    loadStyle();
  },
);
</script>

<style lang="css" scoped>
@reference "tailwindcss";

:deep(.p-inplace-display) {
  @apply w-full;
}
</style>
