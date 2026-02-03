<template>
  <Dialog
    :visible="visible"
    modal
    :header="style ? 'Edit Style' : 'Create New Style'"
    :style="{ width: '600px' }"
    @update:visible="emit('update:visible', $event)">
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
          <label class="block mb-2 text-sm font-medium text-surface-200">Default Strength</label>
          <InputNumber
            v-model="formData.defaultStrength"
            :min="0"
            :max="2"
            :step="0.1"
            showButtons
            buttonLayout="horizontal"
            fluid />
        </div>
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Min Strength</label>
          <InputNumber
            v-model="formData.strengthMin"
            :min="0"
            :max="2"
            :step="0.1"
            showButtons
            buttonLayout="horizontal"
            fluid />
        </div>
        <div>
          <label class="block mb-2 text-sm font-medium text-surface-200">Max Strength</label>
          <InputNumber
            v-model="formData.strengthMax"
            :min="0"
            :max="2"
            :step="0.1"
            showButtons
            buttonLayout="horizontal"
            fluid />
        </div>
      </div>

      <!-- Favorite -->
      <div class="flex items-center gap-2">
        <Checkbox v-model="formData.isFavorite" binary input-id="favorite" />
        <label for="favorite" class="text-sm cursor-pointer text-surface-200">Mark as favorite</label>
      </div>
    </div>

    <template #footer>
      <Button @click="emit('close')" severity="secondary" variant="outlined">Cancel</Button>
      <Button @click="handleSave" :disabled="!isValid">
        {{ style ? 'Update' : 'Create' }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, withDefaults } from 'vue';
import type { StyleInfo, StyleRequest } from '@/types';
import { useStylesStore } from '@/stores/styles';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Textarea from 'primevue/textarea';
import InputNumber from 'primevue/inputnumber';
import Checkbox from 'primevue/checkbox';
import Button from 'primevue/button';
import Divider from 'primevue/divider';

const props = withDefaults(defineProps<{
  visible: boolean;
  style?: StyleInfo | null;
}>(), {
  style: null,
});

const emit = defineEmits<{
  'update:visible': [value: boolean];
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

watch(() => props.visible, (visible) => {
  if (visible) {
    if (props.style) {
      // Editing existing style
      formData.value = {
        name: props.style.name,
        description: props.style.description,
        promptTemplate: props.style.promptTemplate,
        defaultStrength: props.style.defaultStrength,
        strengthMin: props.style.strengthMin,
        strengthMax: props.style.strengthMax,
        category: props.style.category,
        isFavorite: props.style.isFavorite,
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
  }
});

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
