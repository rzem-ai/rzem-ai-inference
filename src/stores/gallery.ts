import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface GalleryImage {
  id: string
  filePath: string
  thumbnailPath?: string
  createdAt: number
  width: number
  height: number
  fileSize: number
  isFavorite: boolean
  prompt: string
  negativePrompt?: string
  modelName: string
  steps?: number
  cfgScale?: number
  seed?: number
  sampler?: string
  tags: string[]
}

export interface GalleryFilters {
  searchQuery: string
  modelName?: string
  isFavorite?: boolean
  tags: string[]
  dateFrom?: number
  dateTo?: number
}

export const useGalleryStore = defineStore('gallery', () => {
  // State
  const images = ref<GalleryImage[]>([])
  const selectedImages = ref<Set<string>>(new Set())
  const filters = ref<GalleryFilters>({
    searchQuery: '',
    tags: [],
  })
  const isLoading = ref(false)

  // Getters
  const filteredImages = computed(() => {
    let result = images.value

    // Search query filter
    if (filters.value.searchQuery) {
      const query = filters.value.searchQuery.toLowerCase()
      result = result.filter(
        (img) =>
          img.prompt.toLowerCase().includes(query) ||
          img.negativePrompt?.toLowerCase().includes(query)
      )
    }

    // Model filter
    if (filters.value.modelName) {
      result = result.filter((img) => img.modelName === filters.value.modelName)
    }

    // Favorite filter
    if (filters.value.isFavorite !== undefined) {
      result = result.filter((img) => img.isFavorite === filters.value.isFavorite)
    }

    // Tags filter
    if (filters.value.tags.length > 0) {
      result = result.filter((img) =>
        filters.value.tags.every((tag) => img.tags.includes(tag))
      )
    }

    // Date range filter
    if (filters.value.dateFrom) {
      result = result.filter((img) => img.createdAt >= filters.value.dateFrom!)
    }
    if (filters.value.dateTo) {
      result = result.filter((img) => img.createdAt <= filters.value.dateTo!)
    }

    return result
  })

  const selectedImagesList = computed(() =>
    Array.from(selectedImages.value)
      .map((id) => images.value.find((img) => img.id === id))
      .filter((img): img is GalleryImage => img !== undefined)
  )

  // Actions
  async function loadImages(): Promise<void> {
    isLoading.value = true
    try {
      const result = await invoke<GalleryImage[]>('get_gallery_images', {
        limit: 100,
      })
      images.value = result
    } catch (error) {
      console.error('Failed to load gallery images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function searchImages(query: string): Promise<void> {
    if (!query.trim()) {
      await loadImages()
      return
    }

    isLoading.value = true
    try {
      const result = await invoke<GalleryImage[]>('search_gallery_images', {
        query: query.trim(),
      })
      images.value = result
    } catch (error) {
      console.error('Failed to search images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function toggleFavorite(imageId: string): Promise<boolean> {
    try {
      await invoke('toggle_favorite', { imageId })
      const image = images.value.find((img) => img.id === imageId)
      if (image) {
        image.isFavorite = !image.isFavorite
      }
      return true
    } catch (error) {
      console.error('Failed to toggle favorite:', error)
      return false
    }
  }

  async function addTag(imageId: string, tag: string): Promise<boolean> {
    try {
      await invoke('add_image_tag', { imageId, tag })
      const image = images.value.find((img) => img.id === imageId)
      if (image && !image.tags.includes(tag)) {
        image.tags.push(tag)
      }
      return true
    } catch (error) {
      console.error('Failed to add tag:', error)
      return false
    }
  }

  async function removeTag(imageId: string, tag: string): Promise<boolean> {
    try {
      await invoke('remove_image_tag', { imageId, tag })
      const image = images.value.find((img) => img.id === imageId)
      if (image) {
        image.tags = image.tags.filter((t) => t !== tag)
      }
      return true
    } catch (error) {
      console.error('Failed to remove tag:', error)
      return false
    }
  }

  async function deleteImage(imageId: string): Promise<boolean> {
    try {
      await invoke('delete_gallery_image', { imageId })
      images.value = images.value.filter((img) => img.id !== imageId)
      selectedImages.value.delete(imageId)
      return true
    } catch (error) {
      console.error('Failed to delete image:', error)
      return false
    }
  }

  function selectImage(imageId: string): void {
    selectedImages.value.add(imageId)
  }

  function deselectImage(imageId: string): void {
    selectedImages.value.delete(imageId)
  }

  function toggleSelectImage(imageId: string): void {
    if (selectedImages.value.has(imageId)) {
      selectedImages.value.delete(imageId)
    } else {
      selectedImages.value.add(imageId)
    }
  }

  function clearSelection(): void {
    selectedImages.value.clear()
  }

  function selectAll(): void {
    filteredImages.value.forEach((img) => selectedImages.value.add(img.id))
  }

  function updateFilters(newFilters: Partial<GalleryFilters>): void {
    filters.value = { ...filters.value, ...newFilters }
  }

  function clearFilters(): void {
    filters.value = {
      searchQuery: '',
      tags: [],
    }
  }

  return {
    // State
    images,
    selectedImages,
    filters,
    isLoading,
    // Getters
    filteredImages,
    selectedImagesList,
    // Actions
    loadImages,
    searchImages,
    toggleFavorite,
    addTag,
    removeTag,
    deleteImage,
    selectImage,
    deselectImage,
    toggleSelectImage,
    clearSelection,
    selectAll,
    updateFilters,
    clearFilters,
  }
})
