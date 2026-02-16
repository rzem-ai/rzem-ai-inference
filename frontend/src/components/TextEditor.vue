<template>
  <EditorContent :editor="editor" class="prompt-editor bg-white border border-surface-300 overscroll-contain text-surface-900 w-full rounded-md px-3 py-2 cursor-text" />
</template>

<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';

const props = defineProps<{
  placeholder?: string;
  editorClass?: string;
}>();

const model = defineModel<string>();

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
      placeholder: props.placeholder || '',
    }),
  ],
  content: model.value || '',
  editorProps: {
    attributes: {
      class: 'focus:outline-none min-h-[60px] max-h-[80px] overflow-y-auto text-lg overscroll-contain',
    },
  },
  onUpdate: ({ editor: e }) => {
    model.value = e.getText();
  },
});

// Sync external changes to editor
watch(
  () => model.value,
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

.text-editor :deep(.tiptap p.is-editor-empty:first-child::before) {
  color: #94a3b8;
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}

.text-editor :deep(.tiptap) {
  min-height: inherit;
  max-height: inherit;
  overflow-y: auto;
}

.text-editor :deep(.tiptap p) {
  margin: 0;
  line-height: 1.5;
}

.prompt-editor :deep(.tiptap p.is-editor-empty:first-child::before) {
  color: #94a3b8;
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}
</style>
