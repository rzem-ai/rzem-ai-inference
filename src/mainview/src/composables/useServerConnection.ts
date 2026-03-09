import { useDiscoveryStore } from '@/stores/discovery';
import { useInferenceStore } from '@/stores/inference';
import { useSettingsStore } from '@/stores/settings';

export function useServerConnection() {
  const discoveryStore = useDiscoveryStore();
  const inferenceStore = useInferenceStore();
  const settingsStore = useSettingsStore();

  async function connect(host: string, port: number) {
    await discoveryStore.connectToServer(host, port);
    if (discoveryStore.isRemote) {
      settingsStore.setConnectionMode('remote');
      inferenceStore.engineReady = true;
      inferenceStore.startPolling();
      await settingsStore.loadGpuInfo();
      await settingsStore.loadEngineStatus();
    }
  }

  async function disconnect() {
    await discoveryStore.disconnectFromServer();
    settingsStore.setConnectionMode('local');
    settingsStore.setRemoteEngineInfo(null);
    inferenceStore.engineReady = false;
    inferenceStore.stopPolling();
    await settingsStore.loadGpuInfo();
    await settingsStore.loadEngineStatus();
  }

  return { connect, disconnect };
}
