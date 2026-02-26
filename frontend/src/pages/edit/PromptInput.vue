<template>
  <EditorContent :editor="editor" class="prompt-editor w-full bg-slate-100 rounded-md px-3 py-2 border border-surface-300 cursor-text" />
</template>

<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { useEditStore } from '@/stores/edit';

const store = useEditStore();

const emit = defineEmits<{
  submit: [];
}>();

const editor = useEditor({
  extensions: [
    StarterKit.configure({
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
      placeholder: 'Describe the edit you want to make...',
    }),
  ],
  content: store.params.prompt,
  editorProps: {
    attributes: {
      class: 'focus:outline-none min-h-24 max-h-48 overflow-y-auto text-lg overscroll-contain',
    },
    handleKeyDown: (_view, event) => {
      if (event.key === 'Enter' && event.ctrlKey) {
        event.preventDefault();
        emit('submit');
        return true;
      }
      return false;
    },
  },
  onUpdate: ({ editor: e }) => {
    store.params.prompt = e.getText();
  },
});

watch(
  () => store.params.prompt,
  (newValue) => {
    if (editor.value && newValue !== editor.value.getText()) {
      editor.value.commands.setContent(newValue || '');
    }
  },
);

onBeforeUnmount(() => {
  editor.value?.destroy();
});
</script>

<style scoped>
@reference "tailwindcss";

.prompt-editor :deep(.tiptap p.is-editor-empty:first-child::before) {
  color: #94a3b8;
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}
</style>
