export interface PricePoint {
  timestamp: number
  price: number
}

export interface TokenMetrics {
  averagePrice: number
  volatility: number      // standard deviation (sample)
  maxPrice: number
  minPrice: number
}

export class TokenAnalysisCalculator {
  private data: PricePoint[]

  constructor(data: PricePoint[]) {
    this.data = (data || [])
      .filter(p => Number.isFinite(p?.timestamp) && Number.isFinite(p?.price))
      .sort((a, b) => a.timestamp - b.timestamp)
  }

  getAveragePrice(): number {
    const n = this.data.length
    if (n === 0) return 0
    const sum = this.data.reduce((acc, p) => acc + p.price, 0)
    return sum / n
  }

  getVolatility(): number {
    const n = this.data.length
    if (n < 2) return 0
    const avg = this.getAveragePrice()
    const variance = this.data.reduce((acc, p) => acc + (p.price - avg) ** 2, 0) / (n - 1) // sample std
    return Math.sqrt(variance)
  }

  getMaxPrice(): number {
    if (this.data.length === 0) return 0
    let max = -Infinity
    for (const p of this.data) if (p.price > max) max = p.price
    return Number.isFinite(max) ? max : 0
  }

  getMinPrice(): number {
    if (this.data.length === 0) return 0
    let min = Infinity
    for (const p of this.data) if (p.price < min) min = p.price
    return Number.isFinite(min) ? min : 0
  }

  computeMetrics(): TokenMetrics {
    return {
      averagePrice: this.getAveragePrice(),
      volatility: this.getVolatility(),
      maxPrice: this.getMaxPrice(),
      minPrice: this.getMinPrice(),
    }
  }
}
