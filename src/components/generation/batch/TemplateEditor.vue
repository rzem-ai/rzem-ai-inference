<script setup lang="ts">
import Accordion from 'primevue/accordion';
import AccordionTab from 'primevue/accordiontab';
import Button from 'primevue/button';
import Chip from 'primevue/chip';
import Textarea from 'primevue/textarea';
import { ref, computed } from 'vue';

// Props
const props = defineProps<{
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
  <div class="template-editor">
    <div class="editor-header">
      <h3 class="section-title">Template</h3>
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
      class="template-textarea"
      rows="3"
      placeholder="Enter template using {{ variable }} syntax..."
    />

    <!-- Available variables -->
    <div v-if="availableColumns.length > 0" class="variables-section">
      <span class="variables-label">Available variables:</span>
      <div class="variables-chips">
        <Chip
          v-for="col in availableColumns"
          :key="col"
          :label="`{{ ${col} }}`"
          @click="insertVariable(col)"
          class="variable-chip"
        />
      </div>
    </div>

    <!-- Help section -->
    <Accordion v-if="showHelp" class="help-accordion">
      <AccordionTab header="Template Syntax Help">
        <div class="help-content">
          <h4>Basic Syntax</h4>
          <ul>
            <li><code>&#123;&#123; "&#123;&#123; variable &#125;&#125;" &#125;&#125;</code> - Insert variable value</li>
            <li><code>&#123;&#123; "&#123;&#123; variable | upper &#125;&#125;" &#125;&#125;</code> - Uppercase filter</li>
            <li><code>&#123;&#123; "&#123;&#123; variable | lower &#125;&#125;" &#125;&#125;</code> - Lowercase filter</li>
            <li><code>&#123;&#123; '&#123;&#123; variable | default("fallback") &#125;&#125;' &#125;&#125;</code> - Default value if empty</li>
          </ul>

          <h4>Examples</h4>
          <div v-for="example in examples" :key="example" class="example">
            <code>&#123;&#123; example &#125;&#125;</code>
            <Button
              icon="pi pi-copy"
              text
              size="small"
              @click="template = example; handleTemplateChange()"
            />
          </div>

          <h4>Supported Filters</h4>
          <ul>
            <li><code>upper</code> - Convert to uppercase</li>
            <li><code>lower</code> - Convert to lowercase</li>
            <li><code>trim</code> - Remove whitespace</li>
            <li><code>title</code> - Title case</li>
            <li><code>default("value")</code> - Fallback value</li>
          </ul>
        </div>
      </AccordionTab>
    </Accordion>
  </div>
</template>

<style scoped>
.template-editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-color);
}

.template-textarea {
  font-family: 'Courier New', monospace;
  font-size: 0.95rem;
}

.variables-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.variables-label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-color-secondary);
}

.variables-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.variable-chip {
  cursor: pointer;
  transition: transform 0.2s;
}

.variable-chip:hover {
  transform: scale(1.05);
}

.help-accordion {
  margin-top: 0.5rem;
}

.help-content {
  font-size: 0.9rem;
}

.help-content h4 {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
  font-size: 0.95rem;
  color: var(--text-color);
}

.help-content ul {
  margin: 0.5rem 0;
  padding-left: 1.5rem;
}

.help-content li {
  margin-bottom: 0.25rem;
}

.help-content code {
  background: var(--surface-ground);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 0.85rem;
}

.example {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.5rem 0;
  padding: 0.5rem;
  background: var(--surface-ground);
  border-radius: var(--border-radius);
}

.example code {
  flex: 1;
  background: transparent;
  padding: 0;
}
</style>
