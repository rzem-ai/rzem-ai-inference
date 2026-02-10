import type { ApiResponse, InferenceEvent, ModelPreset } from "./inference";

export interface PywebviewAPI {
  // ── System ──
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
  health_check(): Promise<{ status: string }>;

  // ── Inference engine ──
  start_engine(args?: {
    device?: string;
    vram_limit_gb?: number;
  }): Promise<ApiResponse>;
  stop_engine(): Promise<ApiResponse>;
  engine_ready(): Promise<ApiResponse<{ ready: boolean }>>;

  // ── Jobs ──
  submit_job(args: Record<string, any>): Promise<ApiResponse<{ job_id?: string }>>;
  cancel_job(args: { job_id: string }): Promise<ApiResponse>;
  poll_events(): Promise<ApiResponse<{ events?: InferenceEvent[] }>>;

  // ── Images & presets ──
  get_image_base64(args: {
    image_path: string;
  }): Promise<ApiResponse<{ data_url?: string }>>;
  get_model_presets(): Promise<ApiResponse<{ presets?: ModelPreset[] }>>;
}

declare global {
  interface Window {
    pywebview?: {
      api: PywebviewAPI;
    };
  }
}
