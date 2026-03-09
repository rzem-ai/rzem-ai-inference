import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue';

interface VirtualGridOptions {
  /** Card aspect ratio as height/width (e.g. 4/3 for portrait, 3/4 for landscape) */
  aspectRatio: number;
  /** Extra height below the card image area (e.g. info text section) in px */
  extraHeight?: number;
  /** Gap between cards in px */
  gap?: number;
  /** Padding inside the grid container in px */
  padding?: number;
}

/** Breakpoint-based column count matching the styles grid layout */
function getColumns(width: number): number {
  if (width >= 1920) return 10;
  if (width >= 1536) return 8;
  if (width >= 1024) return 6;
  return 4;
}

/**
 * Composable for virtualised grid layout.
 * Observes a container element, computes responsive column count,
 * chunks items into rows, and calculates row height for VirtualScroller.
 */
export function useVirtualGrid<T>(
  items: Ref<T[]>,
  containerRef: Ref<HTMLElement | undefined>,
  options: VirtualGridOptions,
) {
  const gap = options.gap ?? 12;
  const padding = options.padding ?? 4;
  const extraHeight = options.extraHeight ?? 0;

  const containerWidth = ref(0);
  let observer: ResizeObserver | null = null;

  const columns = computed(() => getColumns(containerWidth.value));

  const columnWidth = computed(() => {
    const totalGap = (columns.value - 1) * gap;
    const totalPadding = padding * 2;
    return Math.floor((containerWidth.value - totalGap - totalPadding) / columns.value);
  });

  const rowHeight = computed(() => {
    const cardHeight = Math.ceil(columnWidth.value * options.aspectRatio);
    return Math.max(cardHeight + extraHeight + gap, 1);
  });

  const rows = computed(() => {
    const result: T[][] = [];
    const list = items.value;
    const cols = columns.value;
    for (let i = 0; i < list.length; i += cols) {
      result.push(list.slice(i, i + cols));
    }
    return result;
  });

  const gridStyle = computed(() => ({
    display: 'grid',
    gridTemplateColumns: `repeat(${columns.value}, 1fr)`,
    gap: `${gap}px`,
    padding: `${padding}px`,
  }));

  onMounted(() => {
    if (!containerRef.value) return;
    containerWidth.value = containerRef.value.clientWidth;
    observer = new ResizeObserver(([entry]) => {
      containerWidth.value = entry.contentRect.width;
    });
    observer.observe(containerRef.value);
  });

  onUnmounted(() => observer?.disconnect());

  return { columns, columnWidth, rowHeight, rows, gridStyle };
}
