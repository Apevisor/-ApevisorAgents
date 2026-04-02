import type { Signal } from "./signal_api_client"

/**
 * Processes raw signals into actionable events.
 */
export class SignalProcessor {
  /**
   * Filter signals by type and recency.
   */
  filter(signals: Signal[], type: string, sinceTimestamp: number): Signal[] {
    return signals.filter(s => s.type === type && s.timestamp > sinceTimestamp)
  }

  /**
   * Aggregate signals by type, counting occurrences.
   */
  aggregateByType(signals: Signal[]): Record<string, number> {
    return signals.reduce<Record<string, number>>((acc, s) => {
      acc[s.type] = (acc[s.type] ?? 0) + 1
      return acc
    }, {})
  }

  /**
   * Transform a signal into a human-readable summary string.
   */
  summarize(signal: Signal): string {
    const ts = new Date(signal.timestamp).toISOString()
    const payload = (() => {
      try {
        return JSON.stringify(signal.payload)
      } catch {
        return "[unserializable payload]"
      }
    })()
    return `[${ts}] ${signal.type.toUpperCase()}: ${payload}`
  }
}
