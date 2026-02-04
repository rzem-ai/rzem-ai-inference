<template>
  <div class="flex flex-col items-center justify-center h-full p-8 text-center">
    <div class="flex flex-col gap-4">
      <!-- Name -->
      <div>
        <label class="block mb-2 text-sm font-medium text-surface-200">Name</label>
        <InputText v-model="formData.name" placeholder="Style name" fluid />
      </div>

      <!-- Description -->
      <div>
        <label class="block mb-2 text-sm font-medium text-surface-200">Description</label>
        <Textarea v-model="formData.description" placeholder="Optional description" rows="2" fluid />
      </div>

      <!-- Category -->
      <div>
        <label class="block mb-2 text-sm font-medium text-surface-200">Category</label>
        <InputText v-model="formData.category" placeholder="e.g., lora, character, scenery" fluid />
      </div>

      <!-- Prompt Template -->
      <div>
        <label class="block mb-2 text-sm font-medium text-surface-200">Prompt Template</label>
        <Textarea
          v-model="formData.promptTemplate"
          placeholder="Use {{prompt}} as placeholder. Example: cinematic, &lcub;&lcub;prompt&rcub;&rcub;, highly detailed"
          rows="3"
          fluid />
        <p class="mt-1 text-xs text-surface-500">
          Use <code class="px-1 rounded bg-surface-800">&lcub;&lcub;prompt&rcub;&rcub;</code> as placeholder for user input
        </p>
      </div>

      <!-- Template Preview -->
      <div v-if="formData.promptTemplate">
        <label class="block mb-2 text-sm font-medium text-surface-200">Preview</label>
        <div class="flex gap-2">
          <InputText v-model="previewPrompt" placeholder="Test prompt" class="flex-1" size="small" />
          <Button @click="updatePreview" size="small">Preview</Button>
        </div>
        <div v-if="renderedPreview" class="p-3 mt-2 text-sm rounded bg-surface-800 text-surface-300">
          {{ renderedPreview }}
        </div>
      </div>

      <Divider />

      <!-- Strength Settings -->
      <div class="grid grid-cols-3 gap-4">
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Default Strength</label>
          <InputNumber v-model="formData.defaultStrength" :min="0" :max="2" :step="0.1" showButtons buttonLayout="horizontal" fluid />
        </div>
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Min Strength</label>
          <InputNumber v-model="formData.strengthMin" :min="0" :max="2" :step="0.1" showButtons buttonLayout="horizontal" fluid />
        </div>
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Max Strength</label>
          <InputNumber v-model="formData.strengthMax" :min="0" :max="2" :step="0.1" showButtons buttonLayout="horizontal" fluid />
        </div>
      </div>

      <!-- Favorite -->
      <div class="flex items-center gap-2">
        <Checkbox v-model="formData.isFavorite" binary input-id="favorite" />
        <label for="favorite" class="text-sm cursor-pointer text-surface-200">Mark as favorite</label>
      </div>
    </div>

    <Button severity="secondary" variant="outlined">Cancel</Button>
    <Button> Create </Button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { StyleRequest } from '@/types';
import { useStylesStore } from '@/stores/styles';
import Button from 'primevue/button';
import Divider from 'primevue/divider';

const stylesStore = useStylesStore();

const formData = ref<StyleRequest>({
  name: '',
  description: '',
  promptTemplate: '{{prompt}}',
  defaultStrength: 1.0,
  strengthMin: 0.5,
  strengthMax: 1.5,
  category: '',
  isFavorite: false,
});

const previewPrompt = ref('sunset over mountains');
const renderedPreview = ref('');

async function updatePreview() {
  try {
    renderedPreview.value = await stylesStore.previewTemplate(formData.value.promptTemplate, previewPrompt.value);
  } catch (error) {
    console.error('Failed to preview template:', error);
  }
}
</script>
