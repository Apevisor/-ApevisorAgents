export interface WebSocketConfig {
  url: string
  protocols?: string[]
  reconnectIntervalMs?: number
}

export interface SocketMessage {
  topic: string
  payload: any
  timestamp: number
}

export class ResilientWebSocket {
  private socket?: WebSocket
  private readonly url: string
  private readonly protocols?: string[]
  private readonly reconnectInterval: number

  constructor(cfg: WebSocketConfig) {
    this.url = cfg.url
    this.protocols = cfg.protocols
    this.reconnectInterval = cfg.reconnectIntervalMs ?? 5000
  }

  connect(onMessage: (msg: SocketMessage) => void, onOpen?: () => void, onClose?: () => void): void {
    const ws = this.protocols ? new WebSocket(this.url, this.protocols) : new WebSocket(this.url)
    this.socket = ws

    ws.onopen = () => onOpen?.()
    ws.onmessage = evt => {
      try {
        const msg = JSON.parse(evt.data) as SocketMessage
        onMessage(msg)
      } catch {
        // ignore invalid JSON
      }
    }
    ws.onclose = () => {
      onClose?.()
      setTimeout(() => this.connect(onMessage, onOpen, onClose), this.reconnectInterval)
    }
    ws.onerror = () => {
      ws.close()
    }
  }

  send(topic: string, payload: any): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ topic, payload, timestamp: Date.now() }))
    }
  }

  disconnect(): void {
    this.socket?.close()
    this.socket = undefined
  }
}
