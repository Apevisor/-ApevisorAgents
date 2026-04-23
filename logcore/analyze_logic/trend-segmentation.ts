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

type Direction = -1 | 0 | 1

const round2 = (value: number): number => Math.round(value * 100) / 100

const toDirection = (delta: number, epsilon: number): Direction => {
  if (delta > epsilon) return 1
  if (delta < -epsilon) return -1
  return 0
}

const classifyTrend = (changePct: number, neutralThresholdPct: number): TrendResult["trend"] => {
  if (changePct > neutralThresholdPct) return "upward"
  if (changePct < -neutralThresholdPct) return "downward"
  return "neutral"
}

const isValidPoint = (point: PricePoint | null | undefined): point is PricePoint => {
  return Boolean(
    point &&
      Number.isFinite(point.timestamp) &&
      Number.isFinite(point.priceUsd)
  )
}

const normalizePoints = (points: PricePoint[]): PricePoint[] => {
  if (!Array.isArray(points) || points.length === 0) return []

  const sorted = points
    .filter(isValidPoint)
    .slice()
    .sort((a, b) => a.timestamp - b.timestamp)

  if (sorted.length <= 1) return sorted

  const normalized: PricePoint[] = [sorted[0]]

  for (let i = 1; i < sorted.length; i++) {
    const current = sorted[i]
    const last = normalized[normalized.length - 1]

    if (current.timestamp === last.timestamp) {
      normalized[normalized.length - 1] = current
      continue
    }

    normalized.push(current)
  }

  return normalized
}

const getChangePct = (startPrice: number, endPrice: number): number => {
  if (!Number.isFinite(startPrice) || !Number.isFinite(endPrice) || startPrice <= 0) {
    return 0
  }

  return ((endPrice - startPrice) / startPrice) * 100
}

const buildTrendResult = (
  start: PricePoint,
  end: PricePoint,
  neutralThresholdPct: number
): TrendResult => {
  const rawChangePct = getChangePct(start.priceUsd, end.priceUsd)
  const roundedChangePct = round2(rawChangePct)

  return {
    startTime: start.timestamp,
    endTime: end.timestamp,
    trend: classifyTrend(rawChangePct, neutralThresholdPct),
    changePct: roundedChangePct,
  }
}

const mergeShortTailIfNeeded = (
  segments: TrendResult[],
  data: PricePoint[],
  lastSegmentStart: number,
  minSegmentLength: number,
  neutralThresholdPct: number
): TrendResult[] => {
  const remainingLength = data.length - lastSegmentStart
  if (remainingLength <= 0) return segments
  if (remainingLength >= minSegmentLength) {
    const start = data[lastSegmentStart]
    const end = data[data.length - 1]
    return [...segments, buildTrendResult(start, end, neutralThresholdPct)]
  }

  if (segments.length === 0) {
    if (data.length >= 2) {
      return [buildTrendResult(data[0], data[data.length - 1], neutralThresholdPct)]
    }
    return segments
  }

  const merged = segments.slice()
  const previous = merged[merged.length - 1]
  const previousStart = previous.startTime
  const startIndex = data.findIndex(point => point.timestamp === previousStart)

  if (startIndex === -1) return merged

  merged[merged.length - 1] = buildTrendResult(
    data[startIndex],
    data[data.length - 1],
    neutralThresholdPct
  )

  return merged
}

export function analyzePriceTrends(
  points: PricePoint[],
  minSegmentLength: number = 5
): TrendResult[] {
  const normalized = normalizePoints(points)
  const minLen = Math.max(2, Math.floor(minSegmentLength))
  const epsilon = 1e-12
  const neutralThresholdPct = 0.01

  if (normalized.length < minLen) return []

  const results: TrendResult[] = []

  let segmentStartIndex = 0
  let segmentDirection: Direction = 0

  for (let i = 1; i < normalized.length; i++) {
    const previous = normalized[i - 1]
    const current = normalized[i]

    const delta = current.priceUsd - previous.priceUsd
    const stepDirection = toDirection(delta, epsilon)

    if (segmentDirection === 0 && stepDirection !== 0) {
      segmentDirection = stepDirection
    }

    const currentSegmentLength = i - segmentStartIndex + 1
    const directionChanged =
      stepDirection !== 0 &&
      segmentDirection !== 0 &&
      stepDirection !== segmentDirection

    if (directionChanged && currentSegmentLength >= minLen) {
      const segmentStart = normalized[segmentStartIndex]
      const segmentEnd = normalized[i - 1]

      if (segmentEnd.timestamp > segmentStart.timestamp) {
        results.push(buildTrendResult(segmentStart, segmentEnd, neutralThresholdPct))
      }

      segmentStartIndex = i - 1
      segmentDirection = stepDirection
      continue
    }

    if (directionChanged && currentSegmentLength < minLen) {
      segmentDirection = stepDirection
    }
  }

  return mergeShortTailIfNeeded(
    results,
    normalized,
    segmentStartIndex,
    minLen,
    neutralThresholdPct
  )
}
