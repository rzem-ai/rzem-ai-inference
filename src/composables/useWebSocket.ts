/**
 * WebSocket composable for real-time job updates
 *
 * Provides a unified interface for receiving job updates in both local and client modes:
 * - Local/Server mode: Listens to Tauri events
 * - Client mode: Connects to remote WebSocket server
 */

import { ref } from 'vue'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { getRuntimeMode, isClientMode } from './useRuntimeConfig'

export interface JobUpdatePayload {
  job_id: string
  status: string
  progress?: number
  result_path?: string
  error?: string
  stats?: any
}

export interface JobProgressPayload {
  job_id: string
  stage: string
  stage_progress: number
  overall_progress: number
  message: string
  eta_seconds?: number
  current_step?: number
  total_steps?: number
  preview_data?: string
}

type JobUpdateCallback = (payload: JobUpdatePayload) => void
type JobProgressCallback = (payload: JobProgressPayload) => void

/**
 * WebSocket connection manager for client mode
 */
class WebSocketClient {
  private ws: WebSocket | null = null
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 2000
  private subscriptions = new Set<string>()
  private updateCallbacks: JobUpdateCallback[] = []
  private progressCallbacks: JobProgressCallback[] = []
  private serverUrl: string

  constructor(private wsUrl: string) {
    // Extract HTTP server URL from WebSocket URL
    // ws://localhost:8080/api/v1/ws -> http://localhost:8080
    this.serverUrl = wsUrl
      .replace('ws://', 'http://')
      .replace('wss://', 'https://')
      .replace('/api/v1/ws', '')

    this.connect()
  }

  private connect() {
    try {
      this.ws = new WebSocket(this.wsUrl)

      this.ws.onopen = () => {
        console.log('WebSocket connected')
        this.reconnectAttempts = 0

        // Resubscribe to all jobs after reconnection
        this.subscriptions.forEach(jobId => {
          this.send({ type: 'Subscribe', job_id: jobId })
        })
      }

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data)
          this.handleMessage(message)
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err)
        }
      }

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error)
      }

      this.ws.onclose = () => {
        console.log('WebSocket closed')
        this.reconnect()
      }
    } catch (err) {
      console.error('Failed to connect to WebSocket:', err)
      this.reconnect()
    }
  }

  private reconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached')
      return
    }

    this.reconnectAttempts++
    console.log(`Reconnecting in ${this.reconnectDelay}ms (attempt ${this.reconnectAttempts})`)

    setTimeout(() => {
      this.connect()
    }, this.reconnectDelay)
  }

  private send(message: any) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
    }
  }

  private handleMessage(message: any) {
    switch (message.type) {
      case 'Connected':
        console.log('WebSocket connection established:', message.connection_id)
        break

      case 'Subscribed':
        console.log('Subscribed to job:', message.job_id)
        break

      case 'JobProgress':
        // Map server message to job progress payload
        const progressPayload: JobProgressPayload = {
          job_id: message.job_id,
          stage: message.stage || 'loading_models',
          stage_progress: message.stage_progress || 0,
          overall_progress: message.progress || 0,
          message: message.message || '',
          current_step: message.current_step,
          total_steps: message.total_steps,
          preview_data: message.preview_data,
        }
        this.progressCallbacks.forEach(cb => cb(progressPayload))
        break

      case 'JobComplete':
        // Map server message to job update payload
        // Convert relative URL to full URL (e.g., /api/v1/files/x.png -> http://server:8080/api/v1/files/x.png)
        const fullUrl = message.result_url.startsWith('http')
          ? message.result_url
          : `${this.serverUrl}${message.result_url}`

        const completePayload: JobUpdatePayload = {
          job_id: message.job_id,
          status: 'completed',
          progress: 1.0,
          result_path: fullUrl,
        }
        this.updateCallbacks.forEach(cb => cb(completePayload))
        break

      case 'JobFailed':
        const failedPayload: JobUpdatePayload = {
          job_id: message.job_id,
          status: 'failed',
          error: message.error,
        }
        this.updateCallbacks.forEach(cb => cb(failedPayload))
        break

      case 'Pong':
        // Heartbeat response
        break

      default:
        console.warn('Unknown WebSocket message type:', message.type)
    }
  }

  subscribe(jobId: string) {
    this.subscriptions.add(jobId)
    this.send({ type: 'Subscribe', job_id: jobId })
  }

  unsubscribe(jobId: string) {
    this.subscriptions.delete(jobId)
    this.send({ type: 'Unsubscribe', job_id: jobId })
  }

  onUpdate(callback: JobUpdateCallback) {
    this.updateCallbacks.push(callback)
  }

  onProgress(callback: JobProgressCallback) {
    this.progressCallbacks.push(callback)
  }

  close() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }
}

// Global WebSocket client instance (only in client mode)
let wsClient: WebSocketClient | null = null

/**
 * Initialize WebSocket connection for job updates
 *
 * In local/server mode: Returns Tauri event listeners
 * In client mode: Returns WebSocket client
 */
export function useJobUpdates() {
  const updateCallbacks = ref<JobUpdateCallback[]>([])
  const progressCallbacks = ref<JobProgressCallback[]>([])

  if (isClientMode()) {
    // Client mode: Use WebSocket
    if (!wsClient) {
      // Get WebSocket URL from runtime config
      // For now, we'll construct it from the current runtime config
      // This should be done after runtime config is initialized
      const mode = getRuntimeMode()
      if (mode === 'client') {
        // Will be initialized when runtime config is available
        // For now, return a promise-based interface
      }
    }

    return {
      onJobUpdate: (callback: JobUpdateCallback) => {
        if (wsClient) {
          wsClient.onUpdate(callback)
        } else {
          updateCallbacks.value.push(callback)
        }
      },
      onJobProgress: (callback: JobProgressCallback) => {
        if (wsClient) {
          wsClient.onProgress(callback)
        } else {
          progressCallbacks.value.push(callback)
        }
      },
      subscribeToJob: (jobId: string) => {
        if (wsClient) {
          wsClient.subscribe(jobId)
        }
      },
      unsubscribeFromJob: (jobId: string) => {
        if (wsClient) {
          wsClient.unsubscribe(jobId)
        }
      },
      cleanup: () => {
        if (wsClient) {
          wsClient.close()
          wsClient = null
        }
      }
    }
  } else {
    // Local/Server mode: Use Tauri events
    const unlisteners: UnlistenFn[] = []

    return {
      onJobUpdate: async (callback: JobUpdateCallback) => {
        const unlisten = await listen<JobUpdatePayload>('job-update', (event) => {
          callback(event.payload)
        })
        unlisteners.push(unlisten)
      },
      onJobProgress: async (callback: JobProgressCallback) => {
        const unlisten = await listen<JobProgressPayload>('job-progress', (event) => {
          callback(event.payload)
        })
        unlisteners.push(unlisten)
      },
      subscribeToJob: (_jobId: string) => {
        // No-op in local mode (all events are broadcast)
      },
      unsubscribeFromJob: (_jobId: string) => {
        // No-op in local mode
      },
      cleanup: () => {
        unlisteners.forEach(unlisten => unlisten())
      }
    }
  }
}

/**
 * Initialize WebSocket client (call this once at app startup after runtime config is loaded)
 */
export async function initWebSocket(wsUrl: string) {
  if (isClientMode() && !wsClient) {
    wsClient = new WebSocketClient(wsUrl)
  }
}
