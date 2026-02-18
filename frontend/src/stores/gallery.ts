import { defineStore } from 'pinia';
import type { PywebviewAPI } from '@/types/pywebview';
import type { GalleryImage, Folder, Tag } from '@/types/inference';
import { waitForPywebview, getApi, mockApi } from '@/bridge';

let _api: PywebviewAPI | null = null;
let _initPromise: Promise<void> | null = null;

export const useGalleryStore = defineStore('gallery', {
  state: () => ({
    images: [] as GalleryImage[],
    total: 0,
    loading: false,

    folders: [] as Folder[],
    tags: [] as Tag[],

    // Current filters
    currentFolderId: null as string | null,
    currentTagId: null as number | null,
    searchQuery: '',
    favoritesOnly: false,
    page: 0,
    pageSize: 50,
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
      _initPromise = this._doInit();
      return _initPromise;
    },

    async _doInit() {
      try {
        await waitForPywebview();
        _api = getApi() ?? mockApi;
      } catch {
        // Running in browser mode -- use mock
        _api = mockApi;
      }
      await Promise.all([this.loadImages(), this.loadFolders(), this.loadTags()]);
    },

    async loadImages(reset = true) {
      if (!_api) return;
      this.loading = true;
      if (reset) {
        this.page = 0;
        this.images = [];
      }

      try {
        const res = await _api.get_gallery_images({
          limit: this.pageSize,
          offset: this.page * this.pageSize,
          folder_id: this.currentFolderId ?? undefined,
          tag_id: this.currentTagId ?? undefined,
          search: this.searchQuery || undefined,
          favorites_only: this.favoritesOnly,
        });

        if (res.status === 'success') {
          if (reset) {
            this.images = res.images ?? [];
          } else {
            this.images.push(...(res.images ?? []));
          }
          this.total = res.total ?? 0;
        }
      } catch (e) {
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
      if (!_api) return;
      const res = await _api.toggle_favorite({ image_id: imageId });
      if (res.status === 'success' && res.image) {
        const idx = this.images.findIndex(i => i.id === imageId);
        if (idx !== -1) {
          this.images[idx] = res.image;
        }
      }
    },

    async deleteImage(imageId: string) {
      if (!_api) return;
      const res = await _api.delete_image({ image_id: imageId });
      if (res.status === 'success') {
        this.images = this.images.filter(i => i.id !== imageId);
        this.total--;
      }
    },

    // ── Folders ──

    async loadFolders() {
      if (!_api) return;
      const res = await _api.get_folders();
      if (res.status === 'success') {
        this.folders = res.folders ?? [];
      }
    },

    async createFolder(id: string, name: string, parentId?: string) {
      if (!_api) return;
      const res = await _api.create_folder({ id, name, parent_id: parentId });
      if (res.status === 'success') {
        await this.loadFolders();
      }
      return res;
    },

    async deleteFolder(folderId: string) {
      if (!_api) return;
      const res = await _api.delete_folder({ folder_id: folderId });
      if (res.status === 'success') {
        this.folders = this.folders.filter(f => f.id !== folderId);
        if (this.currentFolderId === folderId) {
          this.currentFolderId = null;
          await this.loadImages();
        }
      }
    },

    async addImageToFolder(imageId: string, folderId: string) {
      if (!_api) return;
      await _api.add_image_to_folder({ image_id: imageId, folder_id: folderId });
    },

    async removeImageFromFolder(imageId: string, folderId: string) {
      if (!_api) return;
      await _api.remove_image_from_folder({ image_id: imageId, folder_id: folderId });
      if (this.currentFolderId === folderId) {
        this.images = this.images.filter(i => i.id !== imageId);
        this.total--;
      }
    },

    // ── Tags ──

    async loadTags() {
      if (!_api) return;
      const res = await _api.get_tags();
      if (res.status === 'success') {
        this.tags = res.tags ?? [];
      }
    },

    async createTag(name: string, color?: string, category?: string) {
      if (!_api) return;
      const res = await _api.create_tag({ name, color, category });
      if (res.status === 'success') {
        await this.loadTags();
      }
      return res;
    },

    async deleteTag(tagId: number) {
      if (!_api) return;
      const res = await _api.delete_tag({ tag_id: tagId });
      if (res.status === 'success') {
        this.tags = this.tags.filter(t => t.id !== tagId);
        if (this.currentTagId === tagId) {
          this.currentTagId = null;
          await this.loadImages();
        }
      }
    },

    async addTagToImage(imageId: string, tagId: number) {
      if (!_api) return;
      await _api.add_tag_to_image({ image_id: imageId, tag_id: tagId });
    },

    async removeTagFromImage(imageId: string, tagId: number) {
      if (!_api) return;
      await _api.remove_tag_from_image({ image_id: imageId, tag_id: tagId });
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

    async toggleFavoritesFilter() {
      this.favoritesOnly = !this.favoritesOnly;
      await this.loadImages();
    },
  },
});
