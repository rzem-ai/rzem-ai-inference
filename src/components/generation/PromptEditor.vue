<template>
  <div class="prompt-editor" :data-rows="rows">
    <label v-if="label">{{ label }}</label>
    <div class="editor-container">
      <EditorContent
        :editor="editor"
        class="editor-content"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
  label?: string;
  rows?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'submit': [];
}>();

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

.prompt-editor {
  @apply flex flex-col gap-1;

  label {
    @apply text-xs font-medium tracking-wide mb-1;
    color: #64748b;
  }
}

.editor-container {
  @apply rounded-lg border;
  background-color: #2d3748;
  border-color: #374151;
  transition: border-color 0.2s, box-shadow 0.2s;

  &:focus-within {
    border-color: #3b82f6;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }
}

.editor-content {
  @apply w-full;

  :deep(.tiptap) {
    @apply p-3 min-h-24 max-h-48 overflow-y-auto text-sm;
    color: #e2e8f0;

    &:focus {
      @apply outline-none;
    }

    p {
      @apply m-0;
    }

    /* Placeholder styling */
    &.is-empty::before {
      @apply float-left h-0 pointer-events-none;
      content: attr(data-placeholder);
      color: #64748b;
    }
  }
}

/* Support for different row heights */
.prompt-editor[data-rows="2"] .editor-content :deep(.tiptap) {
  @apply min-h-14;
}

.prompt-editor[data-rows="3"] .editor-content :deep(.tiptap) {
  @apply min-h-20;
}

.prompt-editor[data-rows="4"] .editor-content :deep(.tiptap) {
  @apply min-h-24;
}

.prompt-editor[data-rows="6"] .editor-content :deep(.tiptap) {
  @apply min-h-36;
}
</style>
