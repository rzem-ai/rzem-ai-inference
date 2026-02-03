<template>
  <GenerationAction icon="pen-to-square" label="Image Description">
    <div class="relative">
      <div class="flex flex-col gap-1" :data-rows="4">
        <div class="relative pr-6 border rounded-md border-surface-600 bg-surface-950">
          <CustomScrollbar class="editor-scrollbar">
            <EditorContent :editor="editor" class="w-full editor-content text-surface-300" />
          </CustomScrollbar>
          <!-- Wand Icon Button (Positioned in top-right of prompt) -->
          <div
            class="absolute z-10 justify-center px-1 border rounded top-2 right-2 text-primary-400 border-primary-200 bg-primary-100/50 hover:border hover:border-primary-400 hover:text-primary-600"
            v-tooltip.top="'AI Prompt Assistant'"
            @click="chatStore.togglePanel()">
            <fa :icon="['fal', 'wand-magic-sparkles']" class="w-4 h-4" />
          </div>
        </div>
      </div>
    </div>

    <!-- Configuration validation message -->
    <Message v-if="!generationStore.isValidConfiguration && prompt.trim().length > 0" severity="warn" size="small">
      <template #icon>
        <fa :icon="['fal', 'triangle-exclamation']" size="lg" />
      </template>
      Please select a model bundle or configure all components (Model, T5, CLIP, VAE) in the Quality section
    </Message>
  </GenerationAction>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useGenerationStore } from '@/stores/generation';
import { watch, onUnmounted } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { useChatbotStore } from '@/stores/chatbot';
import CustomScrollbar from '@/components/CustomScrollbar.vue';
import GenerationAction from './GenerationAction.vue';

import Message from 'primevue/message';

// const props = defineProps<{
//   modelValue: string;
//   placeholder?: string;
//   label?: string;
//   rows?: number;
// }>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  submit: [];
  generate: [];
}>();

const chatStore = useChatbotStore();
const generationStore = useGenerationStore();

const prompt = computed({
  get: () => generationStore.currentParams.prompt,
  set: (value: string) => {
    generationStore.currentParams.prompt = value;
  },
});

const placeholder = ref('');

// Initialize TipTap editor
const editor = useEditor({
  extensions: [
    StarterKit.configure({
      // Disable most formatting for simple text input
      heading: false,
      codeBlock: false,
      horizontalRule: false,
      blockquote: false,
      bulletList: false,
      orderedList: false,
      listItem: false,
      code: false,
      bold: false,
      italic: false,
      strike: false,
    }),
    Placeholder.configure({
      placeholder: placeholder.value || 'Enter text...',
    }),
  ],
  content: prompt.value,
  editorProps: {
    attributes: {
      class: 'focus:outline-none',
    },
    handleKeyDown: (_view, event) => {
      // Emit submit on Enter (without shift)
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        emit('submit');
        return true;
      }
      return false;
    },
  },
  onUpdate: ({ editor }) => {
    const text = editor.getText();
    emit('update:modelValue', text);
  },
});

// Watch for external value changes and sync to editor
watch(
  () => prompt.value,
  (newValue) => {
    if (editor.value && newValue !== editor.value.getText()) {
      editor.value.commands.setContent(newValue || '');
    }
  },
);

// Watch for placeholder changes
watch(
  () => placeholder.value,
  (newPlaceholder) => {
    if (editor.value) {
      editor.value.extensionManager.extensions
        .filter((ext) => ext.name === 'placeholder')
        .forEach((ext) => {
          (ext.options as any).placeholder = newPlaceholder || 'Enter text...';
          editor.value?.view.dispatch(editor.value.state.tr);
        });
    }
  },
);

onUnmounted(() => {
  if (editor.value) {
    editor.value.destroy();
  }
});
</script>

<style scoped>
@reference "tailwindcss";

/* CustomScrollbar container */
.editor-scrollbar {
  @apply max-h-48 min-h-24;
  min-height: 0; /* Critical for proper scrolling */
}

/* TipTap editor deep selectors */
.editor-content :deep(.tiptap) {
  @apply p-3 pr-4 text-sm;
  /* Removed overflow-y-auto - CustomScrollbar handles scrolling now */
}

.editor-content :deep(.tiptap:focus) {
  @apply outline-none;
}

.editor-content :deep(.tiptap p) {
  @apply m-0;
}

/* Placeholder styling */
.editor-content :deep(.tiptap.is-empty::before) {
  @apply float-left h-0 pointer-events-none;
  content: attr(data-placeholder);
}

/* Support for different row heights */
[data-rows='2'] .editor-scrollbar {
  @apply min-h-14;
}

[data-rows='3'] .editor-scrollbar {
  @apply min-h-20;
}

[data-rows='4'] .editor-scrollbar {
  @apply min-h-24;
}

[data-rows='6'] .editor-scrollbar {
  @apply min-h-36;
}

/* Custom scrollbar styling */
:deep(.ps__rail-y) {
  @apply bg-gray-200!;
  width: 12px !important;
}

:deep(.ps__rail-y:hover),
:deep(.ps__rail-y:focus) {
  @apply bg-gray-300!;
}

:deep(.ps__thumb-y) {
  @apply bg-gray-400!;
  width: 10px !important;
  border-radius: 6px !important;
}

:deep(.ps__thumb-y:hover),
:deep(.ps__thumb-y:focus) {
  @apply bg-gray-600!;
}
</style>
