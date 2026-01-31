<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="shrink">
      <div class="flex items-start justify-between px-4 pt-6 pb-0">
        <div class="flex flex-col gap-y-1">
          <h2 class="text-xl font-medium text-surface-50">History</h2>
        </div>
        <div class="">{{ reversedHistoryJobs.length }}</div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="reversedHistoryJobs.length === 0" class="flex flex-col items-center justify-center flex-1 gap-3 p-4 text-surface-500">
      <fa :icon="['fal', 'clock-rotate-left']" size="2x" />
      <p class="text-xs text-center">No generation history yet</p>
    </div>

    <!-- History List -->
    <div v-else class="flex-1 overflow-y-auto">
      <div class="flex flex-col gap-2 p-2">
        <HistoryPanelItem v-for="job in reversedHistoryJobs" :key="job.id" :history-job="job" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useQueueStore } from '@/stores/queue';
import HistoryPanelItem from './HistoryPanelItem.vue';

const queueStore = useQueueStore();

defineEmits<{
  restoreImage: [imagePath: string];
}>();

// Show all completed/failed jobs (both from active queue and history)
// Reverse so newest appears at the top
const reversedHistoryJobs = computed(() => {
  // Get completed/failed jobs from active queue
  const completedActive = queueStore.jobs.filter((j) => j.status === 'completed' || j.status === 'failed');

  // Combine with history jobs and reverse
  return completedActive.reverse();
});
</script>

<style scoped>
@reference "tailwindcss";

/* Custom scrollbar */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}

/* Line clamp utility */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
