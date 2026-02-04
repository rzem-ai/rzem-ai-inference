import { defineStore } from 'pinia'
import type { GalleryImage } from './gallery'

export const useCompareStore = defineStore('compare', {
  state: () => ({
    compareImages: [] as GalleryImage[],
    maxCompareImages: 4,
    isInitialized: false,
  }),

  getters: {
    canAddMore(state): boolean {
      return state.compareImages.length < state.maxCompareImages
    },

    compareCount(state): number {
      return state.compareImages.length
    },
  },

  actions: {
    // Minimal initialization (no backend calls needed)
    async initialize(): Promise<void> {
      if (this.isInitialized) return
      // No-op - compare state is local UI state
      this.isInitialized = true
    },

    addToCompare(image: GalleryImage): boolean {
      if (this.compareImages.length >= this.maxCompareImages) {
        return false
      }

      if (this.compareImages.some((img) => img.id === image.id)) {
        return false // Already in compare
      }

      this.compareImages.push(image)
      return true
    },

    removeFromCompare(imageId: string): void {
      this.compareImages = this.compareImages.filter((img) => img.id !== imageId)
    },

    clearCompare(): void {
      this.compareImages = []
    },
  },
})
