export interface Signal {
  id: string
  type: string
  timestamp: number
  payload: Record<string, any>
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

/**
 * Lightweight HTTP client for fetching signals from a backend service.
 */
export class SignalApiClient {
  constructor(private baseUrl: string, private apiKey?: string) {}

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = { "Content-Type": "application/json" }
    if (this.apiKey) headers["Authorization"] = `Bearer ${this.apiKey}`
    return headers
  }

  async fetchAllSignals(): Promise<ApiResponse<Signal[]>> {
    try {
      const res = await fetch(`${this.baseUrl}/signals`, {
        method: "GET",
        headers: this.getHeaders(),
      })
      if (!res.ok) return { success: false, error: `HTTP ${res.status}` }
      const data: Signal[] = await res.json()
      return { success: true, data }
    } catch (err: any) {
      return { success: false, error: err?.message ?? String(err) }
    }
  }

  async fetchSignalById(id: string): Promise<ApiResponse<Signal>> {
    try {
      const res = await fetch(`${this.baseUrl}/signals/${encodeURIComponent(id)}`, {
        method: "GET",
        headers: this.getHeaders(),
      })
      if (!res.ok) return { success: false, error: `HTTP ${res.status}` }
      const data: Signal = await res.json()
      return { success: true, data }
    } catch (err: any) {
      return { success: false, error: err?.message ?? String(err) }
    }
  }
}
