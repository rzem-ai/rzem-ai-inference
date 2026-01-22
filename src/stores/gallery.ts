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
  folderIds: string[]
}

export interface GalleryFilters {
  searchQuery: string
  modelName?: string
  isFavorite?: boolean
  tags: string[]
  dateFrom?: number
  dateTo?: number
  folderId?: string
}

export type GalleryViewMode = 'all' | 'folder' | 'uncategorized'

export const useGalleryStore = defineStore('gallery', () => {
  // State
  const images = ref<GalleryImage[]>([])
  const selectedImages = ref<Set<string>>(new Set())
  const filters = ref<GalleryFilters>({
    searchQuery: '',
    tags: [],
  })
  const isLoading = ref(false)
  const viewMode = ref<GalleryViewMode>('all')
  const currentFolderId = ref<string | null>(null)

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

  async function deleteImages(imageId: string): Promise<boolean> {
    try {
      await invoke('delete_gallery_image', { imageId })
      images.value = images.value.filter((img) => img.id !== imageId)
      selectedImages.value.delete(imageId)
      selectedImages.value = new Set(selectedImages.value)
      return true
    } catch (error) {
      console.error('Failed to delete image:', error)
      return false
    }
  }

  function toggleSelectImage(imageId: string): void {
    if (selectedImages.value.has(imageId)) {
      selectedImages.value.delete(imageId)
    } else {
      selectedImages.value.add(imageId)
    }
    selectedImages.value = new Set(selectedImages.value)
  }

  function clearSelection(): void {
    selectedImages.value.clear()
    selectedImages.value = new Set(selectedImages.value)
  }

  function selectAll(): void {
    filteredImages.value.forEach((img) => selectedImages.value.add(img.id))
    selectedImages.value = new Set(selectedImages.value)
  }

  function setFilter(newFilters: Partial<GalleryFilters>): void {
    filters.value = { ...filters.value, ...newFilters }
  }

  function clearFilters(): void {
    filters.value = {
      searchQuery: '',
      tags: [],
    }
  }

  // Folder-related actions
  async function loadFolderImages(folderId: string, includeDescendants = true): Promise<void> {
    isLoading.value = true
    currentFolderId.value = folderId
    viewMode.value = 'folder'
    try {
      const result = await invoke<GalleryImage[]>('get_folder_images', {
        folderId,
        includeDescendants,
        limit: 100,
      })
      images.value = result
    } catch (error) {
      console.error('Failed to load folder images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function loadUncategorizedImages(): Promise<void> {
    isLoading.value = true
    currentFolderId.value = null
    viewMode.value = 'uncategorized'
    try {
      const result = await invoke<GalleryImage[]>('get_uncategorized_images', {
        limit: 100,
      })
      images.value = result
    } catch (error) {
      console.error('Failed to load uncategorized images:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function loadAllImages(): Promise<void> {
    currentFolderId.value = null
    viewMode.value = 'all'
    await loadImages()
  }

  async function addToFolder(imageIds: string[], folderId: string): Promise<boolean> {
    try {
      await invoke('add_images_to_folder', { imageIds, folderId })
      // Update local state
      for (const imageId of imageIds) {
        const image = images.value.find((img) => img.id === imageId)
        if (image && !image.folderIds.includes(folderId)) {
          image.folderIds.push(folderId)
        }
      }
      return true
    } catch (error) {
      console.error('Failed to add images to folder:', error)
      return false
    }
  }

  async function removeFromFolder(imageIds: string[], folderId: string): Promise<boolean> {
    try {
      await invoke('remove_images_from_folder', { imageIds, folderId })
      // Update local state
      for (const imageId of imageIds) {
        const image = images.value.find((img) => img.id === imageId)
        if (image) {
          image.folderIds = image.folderIds.filter((id) => id !== folderId)
        }
      }
      // If viewing this folder, remove images from view
      if (viewMode.value === 'folder' && currentFolderId.value === folderId) {
        images.value = images.value.filter((img) => !imageIds.includes(img.id))
      }
      return true
    } catch (error) {
      console.error('Failed to remove images from folder:', error)
      return false
    }
  }

  return {
    // State
    images,
    selectedImages,
    filters,
    isLoading,
    viewMode,
    currentFolderId,
    // Getters
    filteredImages,
    selectedImagesList,
    // Actions
    loadImages,
    searchImages,
    toggleFavorite,
    addTag,
    removeTag,
    deleteImages,
    toggleSelectImage,
    clearSelection,
    selectAll,
    setFilter,
    clearFilters,
    // Folder actions
    loadFolderImages,
    loadUncategorizedImages,
    loadAllImages,
    addToFolder,
    removeFromFolder,
  }
})
