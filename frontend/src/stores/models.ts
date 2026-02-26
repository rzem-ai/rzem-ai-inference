import { defineStore } from 'pinia';
import { marked } from 'marked';
import { getApiAsync } from '@/bridge';
import type { BundleType, ModelBundle } from '@/types/inference';

export const useModelsStore = defineStore('models', {
  state: () => ({
    bundleTypes: [] as BundleType[],
    bundles: [] as ModelBundle[],
    loading: false,
    selectedBundleId: null as string | null,
    searchQuery: '',
  }),

  getters: {
    /** Map from type id → BundleType for quick lookups. */
    typeMap(state): Record<string, BundleType> {
      const map: Record<string, BundleType> = {};
      for (const t of state.bundleTypes) {
        map[t.id] = t;
      }
      return map;
    },

    selectedBundle(state): ModelBundle | null {
      return state.bundles.find((b) => b.id === state.selectedBundleId) ?? null;
    },

    favoriteBundles(state): ModelBundle[] {
      return state.bundles.filter((b) => b.favorite);
    },

    filteredBundles(state): ModelBundle[] {
      if (!state.searchQuery) return state.bundles;
      const q = state.searchQuery.toLowerCase();
      return state.bundles.filter(
        (b) =>
          b.label.toLowerCase().includes(q) ||
          b.description.toLowerCase().includes(q) ||
          b.transformer_type.toLowerCase().includes(q),
      );
    },

    bundlesByType(): { label: string; type: string; icon: string | null; items: ModelBundle[] }[] {
      const groups = new Map<string, ModelBundle[]>();
      for (const b of this.filteredBundles) {
        const list = groups.get(b.transformer_type) ?? [];
        list.push(b);
        groups.set(b.transformer_type, list);
      }
      return Array.from(groups.entries()).map(([type, items]) => ({
        label: this.typeMap[type]?.label ?? type,
        type,
        icon: this.typeMap[type]?.icon ?? null,
        items: items.sort((a, b) => (b.favorite ?? 0) - (a.favorite ?? 0)),
      }));
    },

    /** Rendered HTML from the selected bundle's type guide markdown. */
    guideHtml(): string | null {
      const bundle = this.selectedBundle;
      if (!bundle) return null;
      const bt = this.typeMap[bundle.transformer_type];
      if (!bt?.guide) return null;
      return marked.parse(bt.guide) as string;
    },
  },

  actions: {
    /** Get the display label for a transformer type id. */
    getTypeLabel(typeId: string): string {
      return this.typeMap[typeId]?.label ?? typeId;
    },

    async loadBundleTypes() {
      const api = await getApiAsync();
      try {
        const res = await api.get_bundle_types();
        if (res.status === 'success' && res.bundle_types) {
          this.bundleTypes = res.bundle_types;
        }
      } catch (e) {
        console.error('Failed to load bundle types:', e);
      }
    },

    async loadBundles() {
      const api = await getApiAsync();
      this.loading = true;
      try {
        const res = await api.get_bundles({ include_hidden: true });
        if (res.status === 'success' && res.bundles) {
          this.bundles = res.bundles;
        }
      } finally {
        this.loading = false;
      }
    },

    selectBundle(bundleId: string | null) {
      this.selectedBundleId = bundleId;
    },

    async toggleFavorite(bundleId: string) {
      const api = await getApiAsync();
      const res = await api.toggle_bundle_favorite({ bundle_id: bundleId });
      if (res.status === 'success' && res.bundle) {
        const idx = this.bundles.findIndex((b) => b.id === bundleId);
        if (idx >= 0) this.bundles[idx] = res.bundle;
      }
    },

    async toggleHidden(bundleId: string) {
      const api = await getApiAsync();
      const res = await api.toggle_bundle_hidden({ bundle_id: bundleId });
      if (res.status === 'success' && res.bundle) {
        const idx = this.bundles.findIndex((b) => b.id === bundleId);
        if (idx >= 0) this.bundles[idx] = res.bundle;
      }
    },
  },
});
