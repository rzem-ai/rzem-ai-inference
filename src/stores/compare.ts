import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GalleryImage } from './gallery'

export const useCompareStore = defineStore('compare', () => {
  // State
  const compareImages = ref<GalleryImage[]>([])
  const maxCompareImages = 4

  // Getters
  const canAddMore = computed(() => compareImages.value.length < maxCompareImages)

  const compareCount = computed(() => compareImages.value.length)

  // Actions
  function addToCompare(image: GalleryImage): boolean {
    if (compareImages.value.length >= maxCompareImages) {
      return false
    }

    if (compareImages.value.some((img) => img.id === image.id)) {
      return false // Already in compare
    }

    compareImages.value.push(image)
    return true
  }

  function removeFromCompare(imageId: string): void {
    compareImages.value = compareImages.value.filter((img) => img.id !== imageId)
  }

  function clearCompare(): void {
    compareImages.value = []
  }

  return {
    // State
    compareImages,
    maxCompareImages,
    // Getters
    canAddMore,
    compareCount,
    // Actions
    addToCompare,
    removeFromCompare,
    clearCompare,
  }
})
