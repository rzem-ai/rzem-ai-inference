<template>
  <div
    ref="scrollerRef"
    class="overflow-y-auto"
    style="height: 100%"
    @scroll="onScroll">
    <!-- Top spacer: pushes visible items to correct scroll position -->
    <div :style="{ height: topSpace + 'px' }" aria-hidden="true" />

    <!-- Only the visible rows are rendered -->
    <div v-for="(item, i) in visibleItems" :key="first + i">
      <slot :item="item" :index="first + i" :first="first + i === 0" />
    </div>

    <!-- Bottom spacer: maintains correct total scroll height -->
    <div :style="{ height: bottomSpace + 'px' }" aria-hidden="true" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

const BUFFER = 3;

const props = defineProps<{
  items: any[];
  itemSize: number;
}>();

const emit = defineEmits<{
  scroll: [event: Event];
}>();

const scrollerRef = ref<HTMLElement>();
const scrollTop = ref(0);
const viewportHeight = ref(0);
let observer: ResizeObserver | null = null;

const first = computed(() => {
  if (props.itemSize <= 0 || props.items.length === 0) return 0;
  return Math.max(0, Math.floor(scrollTop.value / props.itemSize) - BUFFER);
});

const last = computed(() => {
  if (props.itemSize <= 0 || props.items.length === 0) return 0;
  const end = Math.ceil((scrollTop.value + viewportHeight.value) / props.itemSize) + BUFFER;
  return Math.min(props.items.length - 1, end);
});

const visibleItems = computed(() => {
  if (props.items.length === 0) return [];
  return props.items.slice(first.value, last.value + 1);
});

const topSpace = computed(() => first.value * Math.max(props.itemSize, 0));

const bottomSpace = computed(() => {
  if (props.items.length === 0) return 0;
  return Math.max(0, (props.items.length - last.value - 1) * Math.max(props.itemSize, 0));
});

function onScroll(e: Event) {
  const el = e.target as HTMLElement;
  scrollTop.value = el.scrollTop;
  emit('scroll', e);
}

onMounted(() => {
  if (!scrollerRef.value) return;
  viewportHeight.value = scrollerRef.value.clientHeight;
  observer = new ResizeObserver(([entry]) => {
    viewportHeight.value = entry.contentRect.height;
  });
  observer.observe(scrollerRef.value);
});

onUnmounted(() => observer?.disconnect());
</script>
