import { defineStore } from 'pinia';
import { getApiAsync } from '@/bridge';
import type { DiscoveredServer } from '@/types/inference';

let _pollTimer: ReturnType<typeof setInterval> | null = null;

export const useDiscoveryStore = defineStore('discovery', {
  state: () => ({
    servers: [] as DiscoveredServer[],
    connectionMode: 'local' as string,
    connectedServer: null as { host: string; port: number } | null,
    connecting: false,
    error: null as string | null,
  }),

  getters: {
    isRemote(state): boolean {
      return state.connectionMode === 'remote';
    },
  },

  actions: {
    async refreshServers() {
      try {
        const api = await getApiAsync();
        const res = await api.get_discovered_servers();
        if (res.status === 'success' && res.servers) {
          this.servers = res.servers;
        }
      } catch {
        // Discovery is best-effort
      }
    },

    async loadConnectionMode() {
      try {
        const api = await getApiAsync();
        const res = await api.get_connection_mode();
        if (res.status === 'success') {
          this.connectionMode = res.mode ?? 'local';
          this.connectedServer = res.connected_server ?? null;
        }
      } catch {
        // Non-critical
      }
    },

    async connectToServer(host: string, port: number) {
      if (this.connecting) return;
      this.connecting = true;
      this.error = null;
      try {
        const api = await getApiAsync();
        const res = await api.connect_to_server({ host, port });
        if (res.status === 'success') {
          this.connectionMode = res.mode ?? 'remote';
          this.connectedServer = { host, port };
        } else {
          this.error = res.message ?? 'Failed to connect';
        }
      } catch (e: any) {
        this.error = e.message ?? 'Failed to connect';
      } finally {
        this.connecting = false;
      }
    },

    async disconnectFromServer() {
      this.error = null;
      try {
        const api = await getApiAsync();
        const res = await api.disconnect_from_server();
        if (res.status === 'success') {
          this.connectionMode = res.mode ?? 'local';
          this.connectedServer = null;
        } else {
          this.error = res.message ?? 'Failed to disconnect';
        }
      } catch (e: any) {
        this.error = e.message ?? 'Failed to disconnect';
      }
    },

    startDiscoveryPolling() {
      if (_pollTimer) return;
      this.refreshServers();
      _pollTimer = setInterval(() => this.refreshServers(), 3000);
    },

    stopDiscoveryPolling() {
      if (_pollTimer) {
        clearInterval(_pollTimer);
        _pollTimer = null;
      }
    },
  },
});
