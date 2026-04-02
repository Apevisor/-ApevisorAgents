export interface PairInfo {
  exchange: string
  pairAddress: string
  baseSymbol: string
  quoteSymbol: string
  liquidityUsd: number
  volume24hUsd: number
  priceUsd: number
}

export interface DexSuiteConfig {
  apis: Array<{ name: string; baseUrl: string; apiKey?: string }>
  timeoutMs?: number
}

export class DexSuite {
  constructor(private config: DexSuiteConfig) {}

  private async fetchFromApi<T>(
    api: { name: string; baseUrl: string; apiKey?: string },
    path: string
  ): Promise<T> {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), this.config.timeoutMs ?? 10000)
    try {
      const res = await fetch(`${api.baseUrl}${path}`, {
        headers: api.apiKey ? { Authorization: `Bearer ${api.apiKey}` } : {},
        signal: controller.signal,
      })
      if (!res.ok) throw new Error(`${api.name} ${path} ${res.status}`)
      return (await res.json()) as T
    } finally {
      clearTimeout(timer)
    }
  }

  /**
   * Retrieve aggregated pair info across all configured DEX APIs.
   */
  async getPairInfo(pairAddress: string): Promise<PairInfo[]> {
    const results: PairInfo[] = []
    await Promise.all(
      this.config.apis.map(async api => {
        try {
          const data = await this.fetchFromApi<any>(api, `/pair/${pairAddress}`)
          results.push({
            exchange: api.name,
            pairAddress,
            baseSymbol: data?.token0?.symbol ?? "?",
            quoteSymbol: data?.token1?.symbol ?? "?",
            liquidityUsd: Number(data?.liquidityUsd ?? 0),
            volume24hUsd: Number(data?.volume24hUsd ?? 0),
            priceUsd: Number(data?.priceUsd ?? 0),
          })
        } catch {
          // skip failed API
        }
      })
    )
    return results
  }

  /**
   * Compare a list of pairs across exchanges, returning the best volume and liquidity.
   */
  async comparePairs(
    pairs: string[]
  ): Promise<Record<string, { bestVolume?: PairInfo; bestLiquidity?: PairInfo }>> {
    const entries = await Promise.all(
      pairs.map(async addr => {
        const infos = await this.getPairInfo(addr)
        if (infos.length === 0) return [addr, {}]
        const bestVolume = infos.reduce((a, b) => (b.volume24hUsd > a.volume24hUsd ? b : a))
        const bestLiquidity = infos.reduce((a, b) => (b.liquidityUsd > a.liquidityUsd ? b : a))
        return [addr, { bestVolume, bestLiquidity }] as const
      })
    )
    return Object.fromEntries(entries)
  }
}
