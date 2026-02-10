export interface PywebviewAPI {
  get_system_info(): Promise<{
    platform: string;
    platform_version: string;
    python_version: string;
    machine: string;
    processor: string;
  }>;
  greet(name: string): Promise<string>;
  increment_counter(): Promise<number>;
  get_counter(): Promise<number>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}
