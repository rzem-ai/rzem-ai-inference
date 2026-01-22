import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Tag {
  id: number
  name: string
  color?: string
  category?: string
  usageCount: number
}

export const useTagsStore = defineStore('tags', () => {
  // State
  const tags = ref<Tag[]>([])
  const isLoading = ref(false)

  // Getters
  const tagsByCategory = computed(() => {
    const groups: Record<string, Tag[]> = { uncategorized: [] }

    for (const tag of tags.value) {
      const category = tag.category || 'uncategorized'
      if (!groups[category]) {
        groups[category] = []
      }
      groups[category].push(tag)
    }

    return groups
  })

  const categories = computed(() => {
    const cats = new Set<string>()
    for (const tag of tags.value) {
      if (tag.category) {
        cats.add(tag.category)
      }
    }
    return Array.from(cats).sort()
  })

  const popularTags = computed(() => {
    return [...tags.value].sort((a, b) => b.usageCount - a.usageCount).slice(0, 10)
  })

  // Actions
  async function loadTags(): Promise<void> {
    isLoading.value = true
    try {
      const result = await invoke<Tag[]>('get_all_tags')
      tags.value = result
    } catch (error) {
      console.error('Failed to load tags:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function updateTag(
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
      await loadTags() // Refresh
      return true
    } catch (error) {
      console.error('Failed to update tag:', error)
      return false
    }
  }

  async function bulkAddTag(imageIds: string[], tagName: string): Promise<boolean> {
    try {
      await invoke('bulk_add_tag', { imageIds, tag: tagName })
      await loadTags() // Refresh counts
      return true
    } catch (error) {
      console.error('Failed to bulk add tag:', error)
      return false
    }
  }

  async function bulkRemoveTag(imageIds: string[], tagName: string): Promise<boolean> {
    try {
      await invoke('bulk_remove_tag', { imageIds, tag: tagName })
      await loadTags() // Refresh counts
      return true
    } catch (error) {
      console.error('Failed to bulk remove tag:', error)
      return false
    }
  }

  function getTagByName(name: string): Tag | undefined {
    return tags.value.find((t) => t.name === name)
  }

  function getTagColor(name: string): string | undefined {
    return getTagByName(name)?.color
  }

  return {
    // State
    tags,
    isLoading,
    // Getters
    tagsByCategory,
    categories,
    popularTags,
    // Actions
    loadTags,
    updateTag,
    bulkAddTag,
    bulkRemoveTag,
    getTagByName,
    getTagColor,
  }
})
