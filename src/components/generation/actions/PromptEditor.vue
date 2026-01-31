<template>
  <div class="flex flex-col gap-1" :data-rows="rows">
    <div class="pr-6 border rounded-md border-surface-600 bg-surface-950">
      <EditorContent :editor="editor" class="w-full editor-content text-surface-300" />
      <!-- Wand Icon Button (Positioned in top-right of prompt) -->
      <div class="absolute z-10 justify-center px-1 border rounded top-2 right-2 text-primary-400 border-primary-200 bg-primary-100/50 hover:border hover:border-primary-400 hover:text-primary-600" v-tooltip.top="'AI Prompt Assistant'" @click="chatStore.togglePanel()">
        <fa :icon="['fal', 'wand-magic-sparkles']" class="w-4 h-4" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { useChatbotStore } from '@/stores/chatbot';

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
  label?: string;
  rows?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  submit: [];
}>();

const chatStore = useChatbotStore();

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
      placeholder: props.placeholder || 'Enter text...',
    }),
  ],
  content: props.modelValue,
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
  () => props.modelValue,
  (newValue) => {
    if (editor.value && newValue !== editor.value.getText()) {
      editor.value.commands.setContent(newValue || '');
    }
  },
);

// Watch for placeholder changes
watch(
  () => props.placeholder,
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

/* TipTap editor deep selectors */
.editor-content :deep(.tiptap) {
  @apply max-h-48 min-h-24 overflow-y-auto p-3 text-sm;
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
[data-rows='2'] .editor-content :deep(.tiptap) {
  @apply min-h-14;
}

[data-rows='3'] .editor-content :deep(.tiptap) {
  @apply min-h-20;
}

[data-rows='4'] .editor-content :deep(.tiptap) {
  @apply min-h-24;
}

[data-rows='6'] .editor-content :deep(.tiptap) {
  @apply min-h-36;
}
</style>
