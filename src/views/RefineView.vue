<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Refine</h1>
      <p>Model Hub and LoRA Library</p>
    </div>
    <div class="workspace-content">
      <p>Model and LoRA management will go here</p>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

.workspace {
  @apply flex flex-col h-full bg-white;
}

.workspace-header {
  @apply py-6 px-8 border-b border-gray-200;

  h1 {
    @apply m-0 text-2xl font-semibold;
  }

  p {
    @apply mt-1 mb-0 text-gray-500 text-sm;
  }
}

.workspace-content {
  @apply flex-1 p-8 overflow-y-auto;
}
</style>
