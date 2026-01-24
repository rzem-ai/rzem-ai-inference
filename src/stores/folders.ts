import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Folder {
  id: string
  name: string
  parentId?: string
  color?: string
  icon?: string
  sortOrder: number
  createdAt: number
  updatedAt: number
}

export interface FolderNode extends Folder {
  children: FolderNode[]
  imageCount: number
  totalImageCount: number
  path: string[]
}

export type FolderViewType = 'all' | 'uncategorized' | 'folder'

export const useFoldersStore = defineStore('folders', () => {
  // State
  const folders = ref<FolderNode[]>([])
  const currentFolderId = ref<string | null>(null)
  const currentViewType = ref<FolderViewType>('all')
  const expandedFolderIds = ref<Set<string>>(new Set())
  const isLoading = ref(false)

  // Getters
  const currentFolder = computed(() => {
    if (!currentFolderId.value) return null
    return findFolderById(folders.value, currentFolderId.value)
  })

  const currentBreadcrumb = computed(() => {
    if (!currentFolder.value) return []
    return [...currentFolder.value.path, currentFolder.value.name]
  })

  const flatFolders = computed(() => {
    const result: FolderNode[] = []
    const flatten = (nodes: FolderNode[]) => {
      for (const node of nodes) {
        result.push(node)
        if (node.children.length > 0) {
          flatten(node.children)
        }
      }
    }
    flatten(folders.value)
    return result
  })

  // Helper to find folder by ID recursively
  function findFolderById(nodes: FolderNode[], id: string): FolderNode | null {
    for (const node of nodes) {
      if (node.id === id) return node
      if (node.children.length > 0) {
        const found = findFolderById(node.children, id)
        if (found) return found
      }
    }
    return null
  }

  // Actions
  async function loadFolders(): Promise<void> {
    isLoading.value = true
    try {
      const result = await invoke<FolderNode[]>('get_folder_tree')
      folders.value = result
    } catch (error) {
      console.error('Failed to load folders:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function createFolder(
    name: string,
    parentId?: string,
    color?: string,
    icon?: string
  ): Promise<Folder | null> {
    try {
      const folder = await invoke<Folder>('create_folder', {
        name,
        parentId,
        color,
        icon,
      })
      await loadFolders() // Refresh tree
      return folder
    } catch (error) {
      console.error('Failed to create folder:', error)
      return null
    }
  }

  async function updateFolder(
    id: string,
    updates: { name?: string; color?: string; icon?: string }
  ): Promise<boolean> {
    try {
      await invoke('update_folder', {
        id,
        name: updates.name,
        color: updates.color,
        icon: updates.icon,
      })
      await loadFolders() // Refresh tree
      return true
    } catch (error) {
      console.error('Failed to update folder:', error)
      return false
    }
  }

  async function deleteFolder(id: string): Promise<boolean> {
    try {
      await invoke('delete_folder', { id })
      // Clear selection if deleted folder was selected
      if (currentFolderId.value === id) {
        setCurrentFolder(null)
      }
      await loadFolders() // Refresh tree
      return true
    } catch (error) {
      console.error('Failed to delete folder:', error)
      return false
    }
  }

  async function moveFolder(id: string, newParentId?: string): Promise<boolean> {
    try {
      await invoke('move_folder', { id, newParentId })
      await loadFolders() // Refresh tree
      return true
    } catch (error) {
      console.error('Failed to move folder:', error)
      return false
    }
  }

  async function addImagesToFolder(imageIds: string[], folderId: string): Promise<boolean> {
    try {
      await invoke('add_images_to_folder', { imageIds, folderId })
      await loadFolders() // Refresh counts
      return true
    } catch (error) {
      console.error('Failed to add images to folder:', error)
      return false
    }
  }

  async function removeImagesFromFolder(imageIds: string[], folderId: string): Promise<boolean> {
    try {
      await invoke('remove_images_from_folder', { imageIds, folderId })
      await loadFolders() // Refresh counts
      return true
    } catch (error) {
      console.error('Failed to remove images from folder:', error)
      return false
    }
  }

  async function reorderFolders(folderIds: string[]): Promise<boolean> {
    try {
      await invoke('reorder_folders', { folderIds })
      await loadFolders() // Refresh tree
      return true
    } catch (error) {
      console.error('Failed to reorder folders:', error)
      return false
    }
  }

  function setCurrentFolder(folderId: string | null): void {
    currentFolderId.value = folderId
    currentViewType.value = folderId ? 'folder' : 'all'
  }

  function setViewType(viewType: FolderViewType): void {
    currentViewType.value = viewType
    if (viewType !== 'folder') {
      currentFolderId.value = null
    }
  }

  function toggleExpanded(folderId: string): void {
    if (expandedFolderIds.value.has(folderId)) {
      expandedFolderIds.value.delete(folderId)
    } else {
      expandedFolderIds.value.add(folderId)
    }
    expandedFolderIds.value = new Set(expandedFolderIds.value)
  }

  function expandToFolder(folderId: string): void {
    const folder = findFolderById(folders.value, folderId)
    if (folder) {
      // Expand all ancestors
      for (const ancestorName of folder.path) {
        const ancestor = flatFolders.value.find((f) => f.name === ancestorName)
        if (ancestor) {
          expandedFolderIds.value.add(ancestor.id)
        }
      }
      expandedFolderIds.value = new Set(expandedFolderIds.value)
    }
  }

  function collapseAll(): void {
    expandedFolderIds.value.clear()
    expandedFolderIds.value = new Set(expandedFolderIds.value)
  }

  function expandAll(): void {
    flatFolders.value.forEach((f) => {
      if (f.children.length > 0) {
        expandedFolderIds.value.add(f.id)
      }
    })
    expandedFolderIds.value = new Set(expandedFolderIds.value)
  }

  return {
    // State
    folders,
    currentFolderId,
    currentViewType,
    expandedFolderIds,
    isLoading,
    // Getters
    currentFolder,
    currentBreadcrumb,
    flatFolders,
    // Actions
    loadFolders,
    createFolder,
    updateFolder,
    deleteFolder,
    moveFolder,
    addImagesToFolder,
    removeImagesFromFolder,
    reorderFolders,
    setCurrentFolder,
    setViewType,
    toggleExpanded,
    expandToFolder,
    collapseAll,
    expandAll,
  }
})
