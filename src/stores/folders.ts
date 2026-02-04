import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export interface Folder {
  id: string;
  name: string;
  parentId?: string;
  color?: string;
  icon?: string;
  sortOrder: number;
  createdAt: number;
  updatedAt: number;
}

export interface FolderNode extends Folder {
  children: FolderNode[];
  imageCount: number;
  totalImageCount: number;
  path: string[];
}

export type FolderViewType = 'all' | 'uncategorized' | 'folder';

const _findFolderById = function (nodes: FolderNode[], id: string): FolderNode | null {
  for (const node of nodes) {
    if (node.id === id) return node;
    if (node.children.length > 0) {
      const found = _findFolderById(node.children, id);
      if (found) {
        return found;
      }
    }
  }
  return null;
};

export const useFoldersStore = defineStore('folders', {
  state: () => ({
    folders: [] as FolderNode[],
    currentFolderId: null as string | null,
    currentViewType: 'all' as FolderViewType,
    expandedFolderIds: new Set<string>(),
    isLoading: false,
    isInitialized: false,
    error: null as string | null,
  }),

  getters: {
    currentFolder(state): FolderNode | null {
      if (!state.currentFolderId) {
        return null;
      }

      return _findFolderById(state.folders, state.currentFolderId);
    },

    currentBreadcrumb(): string[] {
      const folder = this.currentFolder;
      if (!folder) return [];
      return [...folder.path, folder.name];
    },

    flatFolders(state): FolderNode[] {
      const result: FolderNode[] = [];
      const flatten = (nodes: FolderNode[]) => {
        for (const node of nodes) {
          result.push(node);
          if (node.children.length > 0) {
            flatten(node.children);
          }
        }
      };
      flatten(state.folders);
      return result;
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
        await this.loadFolders()
        this.isInitialized = true
      } catch (err) {
        this.error = err instanceof Error ? err.message : String(err)
        console.error('[FoldersStore] Initialization failed:', err)
        throw err
      }
    },

    // Force reload (for manual refresh)
    async reload(): Promise<void> {
      this.isInitialized = false
      await this.initialize()
    },

    // Helper to find folder by ID recursively
    findFolderById(nodes: FolderNode[], id: string): FolderNode | null {
      return _findFolderById(nodes, id);
    },

    async loadFolders(): Promise<void> {
      this.isLoading = true;
      try {
        const result = await invoke<FolderNode[]>('get_folder_tree');
        this.folders = result;
      } catch (error) {
        console.error('Failed to load folders:', error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async createFolder(name: string, parentId?: string, color?: string, icon?: string): Promise<Folder | null> {
      try {
        const folder = await invoke<Folder>('create_folder', {
          name,
          parentId,
          color,
          icon,
        });
        await this.loadFolders(); // Refresh tree
        return folder;
      } catch (error) {
        console.error('Failed to create folder:', error);
        return null;
      }
    },

    async updateFolder(id: string, updates: { name?: string; color?: string; icon?: string }): Promise<boolean> {
      try {
        await invoke('update_folder', {
          id,
          name: updates.name,
          color: updates.color,
          icon: updates.icon,
        });
        await this.loadFolders(); // Refresh tree
        return true;
      } catch (error) {
        console.error('Failed to update folder:', error);
        return false;
      }
    },

    async deleteFolder(id: string): Promise<boolean> {
      try {
        await invoke('delete_folder', { id });
        // Clear selection if deleted folder was selected
        if (this.currentFolderId === id) {
          this.setCurrentFolder(null);
        }
        await this.loadFolders(); // Refresh tree
        return true;
      } catch (error) {
        console.error('Failed to delete folder:', error);
        return false;
      }
    },

    async moveFolder(id: string, newParentId?: string): Promise<boolean> {
      try {
        await invoke('move_folder', { id, newParentId });
        await this.loadFolders(); // Refresh tree
        return true;
      } catch (error) {
        console.error('Failed to move folder:', error);
        return false;
      }
    },

    async addImagesToFolder(imageIds: string[], folderId: string): Promise<boolean> {
      try {
        await invoke('add_images_to_folder', { imageIds, folderId });
        await this.loadFolders(); // Refresh counts
        return true;
      } catch (error) {
        console.error('Failed to add images to folder:', error);
        return false;
      }
    },

    async removeImagesFromFolder(imageIds: string[], folderId: string): Promise<boolean> {
      try {
        await invoke('remove_images_from_folder', { imageIds, folderId });
        await this.loadFolders(); // Refresh counts
        return true;
      } catch (error) {
        console.error('Failed to remove images from folder:', error);
        return false;
      }
    },

    async reorderFolders(folderIds: string[]): Promise<boolean> {
      try {
        await invoke('reorder_folders', { folderIds });
        await this.loadFolders(); // Refresh tree
        return true;
      } catch (error) {
        console.error('Failed to reorder folders:', error);
        return false;
      }
    },

    setCurrentFolder(folderId: string | null): void {
      this.currentFolderId = folderId;
      this.currentViewType = folderId ? 'folder' : 'all';
    },

    setViewType(viewType: FolderViewType): void {
      this.currentViewType = viewType;
      if (viewType !== 'folder') {
        this.currentFolderId = null;
      }
    },

    toggleExpanded(folderId: string): void {
      if (this.expandedFolderIds.has(folderId)) {
        this.expandedFolderIds.delete(folderId);
      } else {
        this.expandedFolderIds.add(folderId);
      }
      this.expandedFolderIds = new Set(this.expandedFolderIds);
    },

    expandToFolder(folderId: string): void {
      const folder = this.findFolderById(this.folders, folderId);
      if (folder) {
        // Expand all ancestors
        for (const ancestorName of folder.path) {
          const ancestor = this.flatFolders.find((f) => f.name === ancestorName);
          if (ancestor) {
            this.expandedFolderIds.add(ancestor.id);
          }
        }
        this.expandedFolderIds = new Set(this.expandedFolderIds);
      }
    },

    collapseAll(): void {
      this.expandedFolderIds.clear();
      this.expandedFolderIds = new Set(this.expandedFolderIds);
    },

    expandAll(): void {
      this.flatFolders.forEach((f) => {
        if (f.children.length > 0) {
          this.expandedFolderIds.add(f.id);
        }
      });
      this.expandedFolderIds = new Set(this.expandedFolderIds);
    },
  },
});
