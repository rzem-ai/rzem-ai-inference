<template>
  <div class="inline-editable-text" :class="{ 'is-editing': isEditing }">
    <!-- Display mode -->
    <div
      v-if="!isEditing"
      class="inline-editable-text__display"
      @click="startEdit"
      :class="displayClass"
    >
      <slot name="display" :value="modelValue">
        <span v-if="modelValue" class="inline-editable-text__value">{{ modelValue }}</span>
        <span v-else class="inline-editable-text__placeholder">{{ placeholder }}</span>
      </slot>
    </div>

    <!-- Edit mode -->
    <div v-else class="inline-editable-text__edit">
      <InputText
        ref="inputRef"
        v-model="internalValue"
        :fluid="fluid"
        :class="inputClass"
        @blur="handleBlur"
        @keydown.enter="handleEnter"
        @keydown.esc="handleEscape"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import InputText from 'primevue/inputtext';

interface Props {
  modelValue: string | null;
  placeholder?: string;
  fluid?: boolean;
  displayClass?: string;
  inputClass?: string;
  autoSaveOnBlur?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Click to edit',
  fluid: true,
  displayClass: '',
  inputClass: '',
  autoSaveOnBlur: true,
});

const emit = defineEmits<{
  'update:modelValue': [value: string | null];
  save: [value: string | null];
  cancel: [];
}>();

const isEditing = ref(false);
const internalValue = ref(props.modelValue ?? '');
const inputRef = ref<InstanceType<typeof InputText> | null>(null);

function startEdit() {
  isEditing.value = true;
  internalValue.value = props.modelValue ?? '';

  nextTick(() => {
    inputRef.value?.$el.focus();
  });
}

function handleBlur() {
  if (props.autoSaveOnBlur) {
    save();
  } else {
    cancel();
  }
}

function handleEnter() {
  save();
}

function handleEscape() {
  cancel();
}

function save() {
  const newValue = internalValue.value.trim() === '' ? null : internalValue.value;
  if (newValue !== props.modelValue) {
    emit('update:modelValue', newValue);
    emit('save', newValue);
  }
  isEditing.value = false;
}

function cancel() {
  internalValue.value = props.modelValue ?? '';
  isEditing.value = false;
  emit('cancel');
}
</script>

<style scoped>
@reference "tailwindcss";

.inline-editable-text__display {
  @apply cursor-pointer transition-colors duration-150;
  @apply hover:bg-gray-800/50 rounded px-2 py-1 -mx-2 -my-1;
  @apply min-w-0 flex items-center;
}

.inline-editable-text__value {
  @apply flex-1 min-w-0;
}

.inline-editable-text__placeholder {
  @apply text-gray-500 italic flex-1 min-w-0;
}

.inline-editable-text__edit {
  @apply w-full;
}
</style>
