export interface PricePoint {
  timestamp: number
  priceUsd: number
}

export interface TrendResult {
  startTime: number
  endTime: number
  trend: "upward" | "downward" | "neutral"
  changePct: number
}

const round2 = (n: number) => Math.round(n * 100) / 100

/**
 * Segment a time-ordered price series into trend blocks with simple sign-based detection.
 * - Ignores invalid points
 * - Ensures at least `minSegmentLength` samples per segment
 * - Safe for zero/negative baselines
 */
export function analyzePriceTrends(points: PricePoint[], minSegmentLength: number = 5): TrendResult[] {
  const results: TrendResult[] = []
  const minLen = Math.max(2, Math.floor(minSegmentLength))

  const data = points
    .filter(p => Number.isFinite(p.timestamp) && Number.isFinite(p.priceUsd))
    .sort((a, b) => a.timestamp - b.timestamp)

  if (data.length < minLen) return results

  let segStart = 0
  let dir = 0 // -1 down, 0 flat/unknown, 1 up

  for (let i = 1; i < data.length; i++) {
    const delta = data[i].priceUsd - data[i - 1].priceUsd
    const stepDir = delta > 0 ? 1 : delta < 0 ? -1 : 0
    if (dir === 0) dir = stepDir

    const segLen = i - segStart + 1
    const directionChanged = stepDir !== 0 && dir !== 0 && stepDir !== dir
    const reachedEnd = i === data.length - 1

    if ((directionChanged && segLen >= minLen) || (reachedEnd && segLen >= minLen)) {
      const start = data[segStart]
      const end = data[i]
      const base = start.priceUsd
      const changePct = base > 0 ? ((end.priceUsd - base) / base) * 100 : 0
      results.push({
        startTime: start.timestamp,
        endTime: end.timestamp,
        trend: changePct > 0 ? "upward" : changePct < 0 ? "downward" : "neutral",
        changePct: round2(changePct),
      })
      segStart = i
      dir = stepDir
      continue
    }

    // If direction flipped but the segment is too short, shift the start forward
    if (directionChanged && segLen < minLen) {
      segStart = i - 1
      dir = stepDir
    }
  }

  return results
}
