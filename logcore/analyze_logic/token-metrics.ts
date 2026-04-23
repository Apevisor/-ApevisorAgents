export interface PricePoint {
  timestamp: number
  price: number
}

export interface TokenMetrics {
  averagePrice: number
  volatility: number
  maxPrice: number
  minPrice: number
  returnsVolatility: number
  priceChange: number
}

export class TokenAnalysisCalculator {
  private data: PricePoint[]

  constructor(data: PricePoint[]) {
    this.data = (data || [])
      .filter(p => Number.isFinite(p?.timestamp) && Number.isFinite(p?.price) && p.price > 0)
      .sort((a, b) => a.timestamp - b.timestamp)
  }

  computeMetrics(): TokenMetrics {
    const n = this.data.length

    if (n === 0) {
      return {
        averagePrice: 0,
        volatility: 0,
        maxPrice: 0,
        minPrice: 0,
        returnsVolatility: 0,
        priceChange: 0
      }
    }

    let mean = 0
    let m2 = 0
    let max = -Infinity
    let min = Infinity

    let prevPrice = this.data[0].price
    let returnsM2 = 0
    let returnsMean = 0
    let returnsCount = 0

    for (let i = 0; i < n; i++) {
      const price = this.data[i].price

      // Welford for price volatility
      const delta = price - mean
      mean += delta / (i + 1)
      m2 += delta * (price - mean)

      if (price > max) max = price
      if (price < min) min = price

      // returns volatility (log returns)
      if (i > 0) {
        const logReturn = Math.log(price / prevPrice)

        returnsCount++
        const deltaR = logReturn - returnsMean
        returnsMean += deltaR / returnsCount
        returnsM2 += deltaR * (logReturn - returnsMean)
      }

      prevPrice = price
    }

    const variance = n > 1 ? m2 / (n - 1) : 0
    const returnsVariance = returnsCount > 1 ? returnsM2 / (returnsCount - 1) : 0

    const firstPrice = this.data[0].price
    const lastPrice = this.data[n - 1].price

    return {
      averagePrice: mean,
      volatility: Math.sqrt(variance),
      maxPrice: Number.isFinite(max) ? max : 0,
      minPrice: Number.isFinite(min) ? min : 0,
      returnsVolatility: Math.sqrt(returnsVariance),
      priceChange: firstPrice > 0 ? (lastPrice - firstPrice) / firstPrice : 0
    }
  }
}
