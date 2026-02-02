import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface LoraInfo {
  id: string
  strength: number
}

export interface GalleryImage {
  id: string
  filePath?: string  // Nullable for pending images
  thumbnailPath?: string
  createdAt: number
  width?: number  // Nullable for pending images
  height?: number  // Nullable for pending images
  fileSize?: number  // Nullable for pending images
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
  status: 'pending' | 'processing' | 'completed' | 'failed'
  sessionId?: string
  updatedAt: number
  loras?: LoraInfo[]  // LoRA adapters with strengths
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

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    images: [] as GalleryImage[],
    selectedImages: new Set<string>(),
    filters: {
      searchQuery: '',
      tags: [],
    } as GalleryFilters,
    isLoading: false,
    viewMode: 'all' as GalleryViewMode,
    currentFolderId: null as string | null,
  }),

  getters: {
    filteredImages(state): GalleryImage[] {
      let result = state.images

      // Search query filter
      if (state.filters.searchQuery) {
        const query = state.filters.searchQuery.toLowerCase()
        result = result.filter(
          (img) =>
            img.prompt.toLowerCase().includes(query) ||
            img.negativePrompt?.toLowerCase().includes(query)
        )
      }

      // Model filter
      if (state.filters.modelName) {
        result = result.filter((img) => img.modelName === state.filters.modelName)
      }

      // Favorite filter
      if (state.filters.isFavorite !== undefined) {
        result = result.filter((img) => img.isFavorite === state.filters.isFavorite)
      }

      // Tags filter
      if (state.filters.tags.length > 0) {
        result = result.filter((img) =>
          state.filters.tags.every((tag) => img.tags.includes(tag))
        )
      }

      // Date range filter
      if (state.filters.dateFrom) {
        result = result.filter((img) => img.createdAt >= state.filters.dateFrom!)
      }
      if (state.filters.dateTo) {
        result = result.filter((img) => img.createdAt <= state.filters.dateTo!)
      }

      return result
    },

    selectedImagesList(state): GalleryImage[] {
      return Array.from(state.selectedImages)
        .map((id) => state.images.find((img) => img.id === id))
        .filter((img): img is GalleryImage => img !== undefined)
    },
  },

  actions: {
    async loadImages(): Promise<void> {
      this.isLoading = true
      try {
        const result = await invoke<GalleryImage[]>('get_gallery_images', {
          limit: 100,
        })
        this.images = result
      } catch (error) {
        console.error('Failed to load gallery images:', error)
      } finally {
        this.isLoading = false
      }
    },

    async searchImages(query: string): Promise<void> {
      if (!query.trim()) {
        await this.loadImages()
        return
      }

      this.isLoading = true
      try {
        const result = await invoke<GalleryImage[]>('search_gallery_images', {
          query: query.trim(),
        })
        this.images = result
      } catch (error) {
        console.error('Failed to search images:', error)
      } finally {
        this.isLoading = false
      }
    },

    async toggleFavorite(imageId: string): Promise<boolean> {
      try {
        await invoke('toggle_favorite', { imageId })
        const image = this.images.find((img) => img.id === imageId)
        if (image) {
          image.isFavorite = !image.isFavorite
        }
        return true
      } catch (error) {
        console.error('Failed to toggle favorite:', error)
        return false
      }
    },

    async addTag(imageId: string, tag: string): Promise<boolean> {
      try {
        await invoke('add_image_tag', { imageId, tag })
        const image = this.images.find((img) => img.id === imageId)
        if (image && !image.tags.includes(tag)) {
          image.tags.push(tag)
        }
        return true
      } catch (error) {
        console.error('Failed to add tag:', error)
        return false
      }
    },

    async removeTag(imageId: string, tag: string): Promise<boolean> {
      try {
        await invoke('remove_image_tag', { imageId, tag })
        const image = this.images.find((img) => img.id === imageId)
        if (image) {
          image.tags = image.tags.filter((t) => t !== tag)
        }
        return true
      } catch (error) {
        console.error('Failed to remove tag:', error)
        return false
      }
    },

    async deleteImage(imageId: string): Promise<boolean> {
      try {
        await invoke('delete_gallery_image', { imageId })
        this.images = this.images.filter((img) => img.id !== imageId)
        this.selectedImages.delete(imageId)
        this.selectedImages = new Set(this.selectedImages)
        return true
      } catch (error) {
        console.error('Failed to delete image:', error)
        return false
      }
    },

    async deleteSelectedImages(): Promise<{ success: number; failed: number }> {
      const imageIds = Array.from(this.selectedImages)
      let success = 0
      let failed = 0

      for (const imageId of imageIds) {
        const result = await this.deleteImage(imageId)
        if (result) {
          success++
        } else {
          failed++
        }
      }

      return { success, failed }
    },

    toggleSelectImage(imageId: string): void {
      if (this.selectedImages.has(imageId)) {
        this.selectedImages.delete(imageId)
      } else {
        this.selectedImages.add(imageId)
      }
      this.selectedImages = new Set(this.selectedImages)
    },

    clearSelection(): void {
      this.selectedImages.clear()
      this.selectedImages = new Set(this.selectedImages)
    },

    selectAll(): void {
      this.filteredImages.forEach((img) => this.selectedImages.add(img.id))
      this.selectedImages = new Set(this.selectedImages)
    },

    setFilter(newFilters: Partial<GalleryFilters>): void {
      this.filters = { ...this.filters, ...newFilters }
    },

    clearFilters(): void {
      this.filters = {
        searchQuery: '',
        tags: [],
      }
    },

    // Folder-related actions
    async loadFolderImages(folderId: string, includeDescendants = true): Promise<void> {
      this.isLoading = true
      this.currentFolderId = folderId
      this.viewMode = 'folder'
      try {
        const result = await invoke<GalleryImage[]>('get_folder_images', {
          folderId,
          includeDescendants,
          limit: 100,
        })
        this.images = result
      } catch (error) {
        console.error('Failed to load folder images:', error)
      } finally {
        this.isLoading = false
      }
    },

    async loadUncategorizedImages(): Promise<void> {
      this.isLoading = true
      this.currentFolderId = null
      this.viewMode = 'uncategorized'
      try {
        const result = await invoke<GalleryImage[]>('get_uncategorized_images', {
          limit: 100,
        })
        this.images = result
      } catch (error) {
        console.error('Failed to load uncategorized images:', error)
      } finally {
        this.isLoading = false
      }
    },

    async loadAllImages(): Promise<void> {
      this.currentFolderId = null
      this.viewMode = 'all'
      await this.loadImages()
    },

    async addToFolder(imageIds: string[], folderId: string): Promise<boolean> {
      try {
        await invoke('add_images_to_folder', { imageIds, folderId })
        // Update local state
        for (const imageId of imageIds) {
          const image = this.images.find((img) => img.id === imageId)
          if (image && !image.folderIds.includes(folderId)) {
            image.folderIds.push(folderId)
          }
        }
        return true
      } catch (error) {
        console.error('Failed to add images to folder:', error)
        return false
      }
    },

    async removeFromFolder(imageIds: string[], folderId: string): Promise<boolean> {
      try {
        await invoke('remove_images_from_folder', { imageIds, folderId })
        // Update local state
        for (const imageId of imageIds) {
          const image = this.images.find((img) => img.id === imageId)
          if (image) {
            image.folderIds = image.folderIds.filter((id) => id !== folderId)
          }
        }
        // If viewing this folder, remove images from view
        if (this.viewMode === 'folder' && this.currentFolderId === folderId) {
          this.images = this.images.filter((img) => !imageIds.includes(img.id))
        }
        return true
      } catch (error) {
        console.error('Failed to remove images from folder:', error)
        return false
      }
    },

    // Update an image in the local state (used when modal edits tags/folders)
    updateImage(updatedImage: GalleryImage): void {
      const index = this.images.findIndex((img) => img.id === updatedImage.id)
      if (index !== -1) {
        this.images[index] = updatedImage
      }
    },
  },
})
