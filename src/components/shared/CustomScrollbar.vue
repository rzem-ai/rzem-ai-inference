<template>
  <component :is="tag" ref="scrollbar" class="ps">
    <slot />
  </component>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import type { Ref } from 'vue';
import PerfectScrollbar from 'perfect-scrollbar';

export type PerfectScrollbarEmitsKeys =
  | 'scroll'
  | 'ps-scroll-y'
  | 'ps-scroll-x'
  | 'ps-scroll-up'
  | 'ps-scroll-down'
  | 'ps-scroll-left'
  | 'ps-scroll-right'
  | 'ps-y-reach-start'
  | 'ps-y-reach-end'
  | 'ps-x-reach-start'
  | 'ps-x-reach-end';

export type PerfectScrollbarEmits = {
  [EventName in PerfectScrollbarEmitsKeys]: [value: Event];
};

const eventListeners: Record<PerfectScrollbarEmitsKeys, (event: Event) => void> = {
  scroll: createEventListener('scroll'),
  'ps-scroll-y': createEventListener('ps-scroll-y'),
  'ps-scroll-x': createEventListener('ps-scroll-x'),
  'ps-scroll-up': createEventListener('ps-scroll-up'),
  'ps-scroll-down': createEventListener('ps-scroll-down'),
  'ps-scroll-left': createEventListener('ps-scroll-left'),
  'ps-scroll-right': createEventListener('ps-scroll-right'),
  'ps-y-reach-start': createEventListener('ps-y-reach-start'),
  'ps-y-reach-end': createEventListener('ps-y-reach-end'),
  'ps-x-reach-start': createEventListener('ps-x-reach-start'),
  'ps-x-reach-end': createEventListener('ps-x-reach-end'),
};

const props = withDefaults(
  defineProps<{
    /** HTML tag to render (default: 'div') */
    tag?: string;
    /** PerfectScrollbar options */
    options?: PerfectScrollbar.Options;
  }>(),
  {
    tag: 'div',
    options: () => ({
      suppressScrollX: true,
      wheelPropagation: true,
    }),
  },
);

const emit = defineEmits<PerfectScrollbarEmits>();
const scrollbar = ref<HTMLElement | null>(null);
const ps: Ref<null | PerfectScrollbar> = ref(null);

defineExpose({
  ps,
});

watch(
  () => props.options,
  () => {
    destroyInstance();
    createInstance();
  },
  { deep: true },
);

onBeforeUnmount(() => {
  destroyInstance();
});

onMounted(() => {
  if (scrollbar.value) {
    createInstance();
  }
});

function createInstance() {
  if (scrollbar.value) {
    ps.value = new PerfectScrollbar(scrollbar.value, props.options);
    toggleListeners();
  }
}

function destroyInstance() {
  if (ps.value) {
    toggleListeners(false);
    ps.value.destroy();
    ps.value = null;
  }
}

function createEventListener(eventName: PerfectScrollbarEmitsKeys) {
  return function (event: Event) {
    emit(eventName as any, event);
  };
}

function toggleListeners(addListeners: boolean = true) {
  if (!ps.value?.element) {
    return;
  }

  Object.entries(eventListeners).forEach(([eventName, listener]) => {
    if (addListeners) {
      ps.value?.element.addEventListener(eventName, listener);
    } else {
      ps.value?.element.removeEventListener(eventName, listener);
    }
  });
}
</script>

<style scoped>
@reference "tailwindcss";

.custom-scrollbar {
  position: relative;
  height: 100%;
}
</style>
