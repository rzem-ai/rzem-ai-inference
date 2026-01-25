<template>
  <div style="width: 100%; height: 600px; position: relative">
    <!-- Example 1: As a background only (original usage) -->
    <FloatingLines
      v-if="false"
      :enabled-waves="['top', 'middle', 'bottom']"
      :line-count="[10, 15, 20]"
      :line-distance="[8, 6, 4]"
      :bend-radius="5.0"
      :bend-strength="-0.5"
      :interactive="true"
      :parallax="true" />

    <!-- Example 2: As a panel with content (background shows through) -->
    <FloatingLines
      :enabled-waves="['top', 'middle', 'bottom']"
      :line-count="[10, 15, 20]"
      :line-distance="[8, 6, 4]"
      :bend-radius="5.0"
      :bend-strength="-0.5"
      :interactive="true"
      :parallax="true">
      <!-- Glass-morphism panel that lets background show through -->
      <div class="glass-panel">
        <h2 class="text-2xl font-bold text-white mb-4">Floating Lines Panel</h2>
        <p class="text-white/80">
          This content sits on top of the animated background. The glass effect lets the floating lines show through.
        </p>
      </div>
    </FloatingLines>
  </div>
</template>

<script setup>
import FloatingLines from '../FloatingLines.vue';
</script>

<style scoped>
.glass-panel {
  /* Glass-morphism effect to let background show through */
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 2rem;
  margin: 2rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}
</style>
