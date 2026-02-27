<template>
  <EditorContent :editor="editor" class="prompt-editor w-full bg-slate-100 rounded-md px-3 py-2 border border-surface-300 cursor-text" />
</template>

<script setup lang="ts">
import { watch, onBeforeUnmount, computed } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { StyleTemplateDecoration } from '@/extensions/StyleTemplateDecoration';
import { useInferenceStore } from '@/stores/inference';

const store = useInferenceStore();

const emit = defineEmits<{
  submit: [];
}>();

const placeholderText = computed(() =>
  store.stylePromptTemplate ? 'Enter your prompt...' : 'Describe what you want to generate...',
);

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
      placeholder: () => placeholderText.value,
    }),
    StyleTemplateDecoration,
  ],
  content: store.params.prompt,
  editorProps: {
    attributes: {
      class: 'focus:outline-none min-h-35 max-h-75 overflow-y-auto text-lg overscroll-contain',
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
    store.applyParams({ prompt: e.getText() });
  },
});

// Sync store prompt → editor content
watch(
  () => store.params.prompt,
  (newValue) => {
    if (editor.value && newValue !== editor.value.getText()) {
      editor.value.commands.setContent(newValue || '');
    }
  },
);

// Sync style template → decoration widgets
watch(
  () => store.stylePromptTemplate,
  (template) => {
    if (!editor.value) return;
    if (!template) {
      editor.value.commands.setStyleTemplate('', '');
      return;
    }
    const idx = template.indexOf('{prompt}');
    if (idx === -1) {
      // No {prompt} placeholder — treat entire template as prefix
      editor.value.commands.setStyleTemplate(template, '');
      return;
    }
    const prefix = template.slice(0, idx);
    const suffix = template.slice(idx + '{prompt}'.length);
    editor.value.commands.setStyleTemplate(prefix, suffix);
  },
  { immediate: true },
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

.prompt-editor :deep(.style-fragment) {
  color: #94a3b8;
  pointer-events: none;
  user-select: none;
}
</style>
