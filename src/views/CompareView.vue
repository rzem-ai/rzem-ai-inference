<template>
  <div class="workspace">
    <div class="workspace-header">
      <h1>Compare</h1>
      <p>Smart Gallery and Image Comparison</p>
    </div>
    <div class="workspace-content">
      <p>Gallery and comparison tools will go here</p>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: white;
}

.workspace-header {
  padding: 1.5rem 2rem;
  border-bottom: 1px solid #e9ecef;
}

.workspace-header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.workspace-header p {
  margin: 0.25rem 0 0 0;
  color: #6c757d;
  font-size: 0.875rem;
}

.workspace-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}
</style>
