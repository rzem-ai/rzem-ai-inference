<script setup lang="ts">
import Accordion from 'primevue/accordion';
import AccordionTab from 'primevue/accordiontab';
import Button from 'primevue/button';
import Chip from 'primevue/chip';
import Textarea from 'primevue/textarea';
import { ref } from 'vue';

// Props
defineProps<{
  availableColumns: string[];
}>();

// Emits
const emit = defineEmits<{
  templateChange: [template: string];
}>();

// State
const template = ref('');
const showHelp = ref(false);

// Watch template changes
function handleTemplateChange() {
  emit('templateChange', template.value);
}

// Expose setTemplate method for parent to call
defineExpose({
  setTemplate(newTemplate: string) {
    template.value = newTemplate;
    emit('templateChange', newTemplate);
  },
});

// Insert variable at cursor
function insertVariable(column: string) {
  const textarea = document.querySelector('.template-textarea') as HTMLTextAreaElement;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const text = template.value;

  const before = text.substring(0, start);
  const after = text.substring(end);
  const variable = `{{ ${column} }}`;

  template.value = before + variable + after;

  // Move cursor after inserted variable
  setTimeout(() => {
    textarea.focus();
    const newPosition = start + variable.length;
    textarea.setSelectionRange(newPosition, newPosition);
  }, 0);

  handleTemplateChange();
}

// Example templates
const examples = [
  'A {{ style }} painting of {{ subject }}',
  'A {{ adjective }} {{ subject }} in {{ setting }}',
  '{{ subject }}, {{ style }} art, {{ lighting }} lighting',
];
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h3 class="m-0 text-lg font-semibold">Template</h3>
      <Button
        icon="pi pi-question-circle"
        text
        rounded
        @click="showHelp = !showHelp"
        v-tooltip.top="'Show syntax help'"
      />
    </div>

    <!-- Template input -->
    <Textarea
      v-model="template"
      @input="handleTemplateChange"
      class="font-mono text-base"
      rows="3"
      placeholder="Enter template using {{ variable }} syntax..."
    />

    <!-- Available variables -->
    <div v-if="availableColumns.length > 0" class="flex flex-col gap-2">
      <span class="text-sm font-medium text-gray-400">Available variables:</span>
      <div class="flex flex-wrap gap-2">
        <Chip
          v-for="col in availableColumns"
          :key="col"
          :label="`{{ ${col} }}`"
          @click="insertVariable(col)"
          class="cursor-pointer transition-transform hover:scale-105"
        />
      </div>
    </div>

    <!-- Help section -->
    <Accordion v-if="showHelp" class="mt-2">
      <AccordionTab header="Template Syntax Help">
        <div class="text-sm">
          <h4 class="mt-4 mb-2 text-base first:mt-0">Basic Syntax</h4>
          <ul class="my-2 pl-6">
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">&#123;&#123; "&#123;&#123; variable &#125;&#125;" &#125;&#125;</code> - Insert variable value</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">&#123;&#123; "&#123;&#123; variable | upper &#125;&#125;" &#125;&#125;</code> - Uppercase filter</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">&#123;&#123; "&#123;&#123; variable | lower &#125;&#125;" &#125;&#125;</code> - Lowercase filter</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">&#123;&#123; '&#123;&#123; variable | default("fallback") &#125;&#125;' &#125;&#125;</code> - Default value if empty</li>
          </ul>

          <h4 class="mt-4 mb-2 text-base">Examples</h4>
          <div v-for="example in examples" :key="example" class="flex items-center gap-2 my-2 p-2 rounded bg-surface-ground">
            <code class="flex-1 font-mono text-[0.85rem]">&#123;&#123; example &#125;&#125;</code>
            <Button
              icon="pi pi-copy"
              text
              size="small"
              @click="template = example; handleTemplateChange()"
            />
          </div>

          <h4 class="mt-4 mb-2 text-base">Supported Filters</h4>
          <ul class="my-2 pl-6">
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">upper</code> - Convert to uppercase</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">lower</code> - Convert to lowercase</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">trim</code> - Remove whitespace</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">title</code> - Title case</li>
            <li class="mb-1"><code class="px-1.5 py-0.5 rounded bg-surface-ground font-mono text-[0.85rem]">default("value")</code> - Fallback value</li>
          </ul>
        </div>
      </AccordionTab>
    </Accordion>
  </div>
</template>

<style scoped>
/* querySelector reference for insertVariable functionality */
.template-textarea {
  /* Intentionally empty - class used for DOM selection only */
}
</style>
