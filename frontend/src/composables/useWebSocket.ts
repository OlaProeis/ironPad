import { ref, onMounted, onUnmounted } from 'vue'
import type { WsMessage, WsConnectedPayload } from '../types'

export interface UseWebSocketOptions {
  onFileCreated?: (path: string) => void
  onFileModified?: (path: string) => void
  onFileDeleted?: (path: string) => void
  onFileRenamed?: (from: string, to: string) => void
  onFileLocked?: (path: string, clientId: string, lockType: string) => void
  onFileUnlocked?: (path: string) => void
  onGitConflict?: (files: string[]) => void
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const connected = ref(false)
  const clientId = ref<string | null>(null)
  let ws: WebSocket | null = null
  let reconnectTimeout: number | null = null
  let reconnectAttempts = 0
  const MAX_RECONNECT_DELAY = 30000 // 30 seconds max

  function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//${window.location.host}/ws`

    ws = new WebSocket(wsUrl)

    ws.onopen = () => {
      connected.value = true
      reconnectAttempts = 0 // Reset backoff on successful connection
      console.log('WebSocket connected')
    }

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as WsMessage
        handleMessage(msg)
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onclose = () => {
      connected.value = false
      clientId.value = null
      // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s max
      const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), MAX_RECONNECT_DELAY)
      reconnectAttempts++
      console.log(`WebSocket disconnected, reconnecting in ${delay / 1000}s...`)
      reconnectTimeout = window.setTimeout(connect, delay)
    }

    ws.onerror = (e) => {
      console.error('WebSocket error:', e)
    }
  }

  function handleMessage(msg: WsMessage) {
    switch (msg.type) {
      case 'Connected': {
        const payload = msg.payload as WsConnectedPayload
        clientId.value = payload.client_id
        console.log('WebSocket client ID:', payload.client_id)
        break
      }
      case 'FileCreated': {
        const payload = msg.payload as { path: string }
        options.onFileCreated?.(payload.path)
        break
      }
      case 'FileModified': {
        const payload = msg.payload as { path: string }
        options.onFileModified?.(payload.path)
        break
      }
      case 'FileDeleted': {
        const payload = msg.payload as { path: string }
        options.onFileDeleted?.(payload.path)
        break
      }
      case 'FileRenamed': {
        const payload = msg.payload as { from: string; to: string }
        options.onFileRenamed?.(payload.from, payload.to)
        break
      }
      case 'FileLocked': {
        const payload = msg.payload as { path: string; client_id: string; lock_type: string }
        options.onFileLocked?.(payload.path, payload.client_id, payload.lock_type)
        break
      }
      case 'FileUnlocked': {
        const payload = msg.payload as { path: string }
        options.onFileUnlocked?.(payload.path)
        break
      }
      case 'GitConflict': {
        const payload = msg.payload as { files: string[] }
        options.onGitConflict?.(payload.files)
        break
      }
      case 'Ping':
        // Heartbeat, no action needed
        break
    }
  }

  function send(message: object) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message))
    }
  }

  function lockFile(path: string, lockType: 'editor' | 'task_view') {
    send({ type: 'lock_file', path, lock_type: lockType })
  }

  function unlockFile(path: string) {
    send({ type: 'unlock_file', path })
  }

  function disconnect() {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
    if (ws) {
      ws.close()
      ws = null
    }
  }

  onMounted(() => {
    connect()
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    connected,
    clientId,
    send,
    lockFile,
    unlockFile,
    disconnect
  }
}
