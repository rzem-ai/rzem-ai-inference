import { defineStore } from 'pinia';
import type { GalleryImage, Folder, Tag } from '@/types/inference';
import { getApiAsync } from '@/bridge';

let _initPromise: Promise<void> | null = null;

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    images: [] as GalleryImage[],
    total: 0,
    loading: false,
    error: null as string | null,

    folders: [] as Folder[],
    tags: [] as Tag[],

    // Current filters
    currentFolderId: null as string | null,
    currentTagId: null as number | null,
    searchQuery: '',
    favoritesOnly: false,
    page: 0,
    pageSize: 50,

    // Sort
    sortBy: 'created_at' as string,
    sortOrder: 'asc' as 'asc' | 'desc',
  }),

  getters: {
    hasMore(state): boolean {
      return state.images.length < state.total;
    },
  },

  actions: {
    /**
     * Initialize the store: wait for the pywebview bridge, then load all data.
     * Safe to call from multiple components -- only runs once.
     */
    async init() {
      if (_initPromise) return _initPromise;
      _initPromise = Promise.all([this.loadImages(), this.loadFolders(), this.loadTags()]).then(() => {});
      return _initPromise;
    },

    async loadImages(reset = true) {
      const api = await getApiAsync();
      this.loading = true;
      if (reset) {
        this.page = 0;
        this.images = [];
      }

      try {
        const res = await api.get_gallery_images({
          limit: this.pageSize,
          offset: this.page * this.pageSize,
          folder_id: this.currentFolderId ?? undefined,
          tag_id: this.currentTagId ?? undefined,
          search: this.searchQuery || undefined,
          favorites_only: this.favoritesOnly,
          sort_by: this.sortBy,
          sort_order: this.sortOrder,
        });

        if (res.status === 'success') {
          if (reset) {
            this.images = res.images ?? [];
          } else {
            this.images.push(...(res.images ?? []));
          }
          this.total = res.total ?? 0;
        }
      } catch (e: any) {
        this.error = e.message ?? 'Failed to load images';
        console.error('[gallery] Failed to load images:', e);
      } finally {
        this.loading = false;
      }
    },

    async loadMore() {
      if (!this.hasMore || this.loading) return;
      this.page++;
      await this.loadImages(false);
    },

    async toggleFavorite(imageId: string) {
      const api = await getApiAsync();
      const res = await api.toggle_favorite({ image_id: imageId });
      if (res.status === 'success' && res.image) {
        const idx = this.images.findIndex(i => i.id === imageId);
        if (idx !== -1) {
          this.images[idx] = res.image;
        }
      }
    },

    async batchSaveImages(imageIds: string[]) {
      const api = await getApiAsync();
      return await api.batch_save_images({ image_ids: imageIds });
    },

    async deleteImage(imageId: string) {
      const api = await getApiAsync();
      const res = await api.delete_image({ image_id: imageId });
      if (res.status === 'success') {
        this.images = this.images.filter(i => i.id !== imageId);
        this.total--;
      }
    },

    // ── Folders ──

    async loadFolders() {
      const api = await getApiAsync();
      const res = await api.get_folders();
      if (res.status === 'success') {
        this.folders = res.folders ?? [];
      }
    },

    async createFolder(id: string, name: string, parentId?: string) {
      const api = await getApiAsync();
      const res = await api.create_folder({ id, name, parent_id: parentId });
      if (res.status === 'success') {
        await this.loadFolders();
      }
      return res;
    },

    async deleteFolder(folderId: string) {
      const api = await getApiAsync();
      const res = await api.delete_folder({ folder_id: folderId });
      if (res.status === 'success') {
        this.folders = this.folders.filter(f => f.id !== folderId);
        if (this.currentFolderId === folderId) {
          this.currentFolderId = null;
          await this.loadImages();
        }
      }
    },

    async addImageToFolder(imageId: string, folderId: string) {
      const api = await getApiAsync();
      await api.add_image_to_folder({ image_id: imageId, folder_id: folderId });
    },

    async removeImageFromFolder(imageId: string, folderId: string) {
      const api = await getApiAsync();
      await api.remove_image_from_folder({ image_id: imageId, folder_id: folderId });
      if (this.currentFolderId === folderId) {
        this.images = this.images.filter(i => i.id !== imageId);
        this.total--;
      }
    },

    // ── Tags ──

    async loadTags() {
      const api = await getApiAsync();
      const res = await api.get_tags();
      if (res.status === 'success') {
        this.tags = res.tags ?? [];
      }
    },

    async createTag(name: string, color?: string, category?: string) {
      const api = await getApiAsync();
      const res = await api.create_tag({ name, color, category });
      if (res.status === 'success') {
        await this.loadTags();
      }
      return res;
    },

    async deleteTag(tagId: number) {
      const api = await getApiAsync();
      const res = await api.delete_tag({ tag_id: tagId });
      if (res.status === 'success') {
        this.tags = this.tags.filter(t => t.id !== tagId);
        if (this.currentTagId === tagId) {
          this.currentTagId = null;
          await this.loadImages();
        }
      }
    },

    async addTagToImage(imageId: string, tagId: number) {
      const api = await getApiAsync();
      await api.add_tag_to_image({ image_id: imageId, tag_id: tagId });
    },

    async removeTagFromImage(imageId: string, tagId: number) {
      const api = await getApiAsync();
      await api.remove_tag_from_image({ image_id: imageId, tag_id: tagId });
    },

    // ── State setters ──

    setFavoritesOnly(val: boolean) {
      this.favoritesOnly = val;
    },

    setCurrentTagId(id: number | null) {
      this.currentTagId = id;
    },

    setCurrentFolderId(id: string | null) {
      this.currentFolderId = id;
    },

    // ── Filter helpers ──

    async filterByFolder(folderId: string | null) {
      this.currentFolderId = folderId;
      await this.loadImages();
    },

    async filterByTag(tagId: number | null) {
      this.currentTagId = tagId;
      await this.loadImages();
    },

    async searchImages(query: string) {
      this.searchQuery = query;
      await this.loadImages();
    },

    async setSort(sortBy: string, sortOrder?: 'asc' | 'desc') {
      this.sortBy = sortBy;
      if (sortOrder) {
        this.sortOrder = sortOrder;
      }
      await this.loadImages();
    },

    async toggleSortOrder() {
      this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
      await this.loadImages();
    },

    async toggleFavoritesFilter() {
      this.favoritesOnly = !this.favoritesOnly;
      await this.loadImages();
    },
  },
});
