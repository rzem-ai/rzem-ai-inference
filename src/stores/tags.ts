import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface Tag {
  id: number
  name: string
  color?: string
  category?: string
  usageCount: number
}

export const useTagsStore = defineStore('tags', {
  state: () => ({
    tags: [] as Tag[],
    isLoading: false,
    isInitialized: false,
    error: null as string | null,
  }),

  getters: {
    tagsByCategory(state): Record<string, Tag[]> {
      const groups: Record<string, Tag[]> = { uncategorized: [] }

      for (const tag of state.tags) {
        const category = tag.category || 'uncategorized'
        if (!groups[category]) {
          groups[category] = []
        }
        groups[category].push(tag)
      }

      return groups
    },

    categories(state): string[] {
      const cats = new Set<string>()
      for (const tag of state.tags) {
        if (tag.category) {
          cats.add(tag.category)
        }
      }
      return Array.from(cats).sort()
    },

    popularTags(state): Tag[] {
      return [...state.tags].sort((a, b) => b.usageCount - a.usageCount).slice(0, 10)
    },
  },

  actions: {
    // Standard initialization method
    async initialize(): Promise<void> {
      // Guard: only initialize once
      if (this.isInitialized) {
        return
      }

      this.error = null

      try {
        await this.loadTags()
        this.isInitialized = true
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('[TagsStore] Initialization failed:', err)
        throw err
      }
    },

    // Force reload (for manual refresh)
    async reload(): Promise<void> {
      this.isInitialized = false
      await this.initialize()
    },

    async loadTags(): Promise<void> {
      this.isLoading = true
      try {
        const result = await invoke<Tag[]>('get_all_tags')
        this.tags = result
      } catch (error) {
        console.error('Failed to load tags:', error)
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async updateTag(
      id: number,
      updates: { name?: string; color?: string; category?: string }
    ): Promise<boolean> {
      try {
        await invoke('update_tag', {
          id,
          name: updates.name,
          color: updates.color,
          category: updates.category,
        })
        await this.loadTags() // Refresh
        return true
      } catch (error) {
        console.error('Failed to update tag:', error)
        return false
      }
    },

    async bulkAddTag(imageIds: string[], tagName: string): Promise<boolean> {
      try {
        await invoke('bulk_add_tag', { imageIds, tag: tagName })
        await this.loadTags() // Refresh counts
        return true
      } catch (error) {
        console.error('Failed to bulk add tag:', error)
        return false
      }
    },

    async bulkRemoveTag(imageIds: string[], tagName: string): Promise<boolean> {
      try {
        await invoke('bulk_remove_tag', { imageIds, tag: tagName })
        await this.loadTags() // Refresh counts
        return true
      } catch (error) {
        console.error('Failed to bulk remove tag:', error)
        return false
      }
    },

    async deleteTag(tagId: number): Promise<boolean> {
      try {
        await invoke('delete_tag', { tagId })
        // Remove from local state
        this.tags = this.tags.filter((t) => t.id !== tagId)
        return true
      } catch (error) {
        console.error('Failed to delete tag:', error)
        return false
      }
    },

    getTagByName(name: string): Tag | undefined {
      return this.tags.find((t) => t.name === name)
    },

    getTagColor(name: string): string | undefined {
      return this.getTagByName(name)?.color
    },
  },
})
