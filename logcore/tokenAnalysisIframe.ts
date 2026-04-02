import type { TokenMetrics } from "./tokenAnalysisCalculator"

export interface IframeConfig {
  containerId: string
  srcUrl: string
  metrics: TokenMetrics
  refreshIntervalMs?: number
}

export class TokenAnalysisIframe {
  private iframeEl: HTMLIFrameElement | null = null
  private intervalId?: number
  private readonly targetOrigin: string

  constructor(private config: IframeConfig) {
    this.targetOrigin = (() => {
      try {
        return new URL(config.srcUrl).origin
      } catch {
        return "*"
      }
    })()
  }

  init(): void {
    if (this.iframeEl) return
    const container = document.getElementById(this.config.containerId)
    if (!container) throw new Error("Container not found: " + this.config.containerId)

    const iframe = document.createElement("iframe")
    iframe.src = this.config.srcUrl
    iframe.width = "100%"
    iframe.height = "100%"
    iframe.onload = () => this.postMetrics()
    container.appendChild(iframe)
    this.iframeEl = iframe

    if (this.config.refreshIntervalMs && this.config.refreshIntervalMs > 0) {
      this.intervalId = window.setInterval(() => this.postMetrics(), this.config.refreshIntervalMs)
    }
  }

  destroy(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = undefined
    }
    this.iframeEl?.remove()
    this.iframeEl = null
  }

  private postMetrics(): void {
    if (!this.iframeEl?.contentWindow) return
    this.iframeEl.contentWindow.postMessage(
      { type: "TOKEN_ANALYSIS_METRICS", payload: this.config.metrics },
      this.targetOrigin
    )
  }
}
