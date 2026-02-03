<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-surface-700">
      <h3 class="text-lg font-semibold text-surface-100">
        {{ style ? 'Edit Style' : 'Create New Style' }}
      </h3>
      <Button @click="emit('close')" variant="text" size="small" severity="secondary">
        <fa :icon="['fal', 'times']" />
      </Button>
    </div>

    <!-- Form content -->
    <div class="flex-1 p-4 overflow-y-auto">
      <div class="flex flex-col gap-4">
        <!-- Name -->
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Name</label>
          <InputText v-model="formData.name" placeholder="Style name" fluid />
        </div>

        <!-- Description -->
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Description</label>
          <Textarea
            v-model="formData.description"
            placeholder="Optional description"
            rows="2"
            fluid />
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
            placeholder="Example: cinematic, highly detailed"
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
            <InputText
              v-model="previewPrompt"
              placeholder="Test prompt"
              class="flex-1"
              size="small" />
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
            <label class="block mb-2 text-sm font-medium text-surface-200">Default</label>
            <InputNumber
              v-model="formData.defaultStrength"
              :min="0"
              :max="2"
              :step="0.1"
              showButtons
              buttonLayout="horizontal"
              size="small"
              fluid />
          </div>
          <div>
            <label class="block mb-2 text-sm font-medium text-surface-200">Min</label>
            <InputNumber
              v-model="formData.strengthMin"
              :min="0"
              :max="2"
              :step="0.1"
              showButtons
              buttonLayout="horizontal"
              size="small"
              fluid />
          </div>
          <div>
            <label class="block mb-2 text-sm font-medium text-surface-200">Max</label>
            <InputNumber
              v-model="formData.strengthMax"
              :min="0"
              :max="2"
              :step="0.1"
              showButtons
              buttonLayout="horizontal"
              size="small"
              fluid />
          </div>
        </div>

        <!-- Favorite -->
        <div class="flex items-center gap-2">
          <Checkbox v-model="formData.isFavorite" binary input-id="favorite" />
          <label for="favorite" class="text-sm cursor-pointer text-surface-200">Mark as favorite</label>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="flex justify-end gap-2 p-4 border-t border-surface-700">
      <Button @click="emit('close')" severity="secondary" variant="outlined">Cancel</Button>
      <Button @click="handleSave" :disabled="!isValid">
        {{ style ? 'Update' : 'Create' }}
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, withDefaults } from 'vue';
import type { StyleInfo, StyleRequest } from '@/types';
import { useStylesStore } from '@/stores/styles';
import InputText from 'primevue/inputtext';
import Textarea from 'primevue/textarea';
import InputNumber from 'primevue/inputnumber';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import Divider from 'primevue/divider';

const props = withDefaults(defineProps<{
  style?: StyleInfo | null;
}>(), {
  style: null,
});

const emit = defineEmits<{
  save: [data: StyleRequest];
  close: [];
}>();

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

const isValid = computed(() => {
  return (
    formData.value.name.trim() !== '' &&
    formData.value.promptTemplate.trim() !== ''
  );
});

watch(() => props.style, (style) => {
  if (style) {
    // Editing existing style
    formData.value = {
      name: style.name,
      description: style.description,
      promptTemplate: style.promptTemplate,
      defaultStrength: style.defaultStrength,
      strengthMin: style.strengthMin,
      strengthMax: style.strengthMax,
      category: style.category,
      isFavorite: style.isFavorite,
    };
  } else {
    // Creating new style
    formData.value = {
      name: '',
      description: '',
      promptTemplate: '{{prompt}}',
      defaultStrength: 1.0,
      strengthMin: 0.5,
      strengthMax: 1.5,
      category: '',
      isFavorite: false,
    };
  }
  renderedPreview.value = '';
}, { immediate: true });

async function updatePreview() {
  try {
    renderedPreview.value = await stylesStore.previewTemplate(
      formData.value.promptTemplate,
      previewPrompt.value
    );
  } catch (error) {
    console.error('Failed to preview template:', error);
  }
}

function handleSave() {
  if (!isValid.value) return;
  emit('save', { ...formData.value });
}
</script>
