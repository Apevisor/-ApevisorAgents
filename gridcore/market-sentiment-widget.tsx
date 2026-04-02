import React from "react"

interface MarketSentimentWidgetProps {
  sentimentScore: number // 0..100
  trend: "Bullish" | "Bearish" | "Neutral"
  dominantToken: string
  totalVolume24h: number
}

const clamp = (n: number, min = 0, max = 100) => Math.max(min, Math.min(max, n))

const getSentimentColor = (score: number): string => {
  const s = clamp(score)
  if (s >= 70) return "#4caf50"
  if (s >= 40) return "#ff9800"
  return "#f44336"
}

export const MarketSentimentWidget: React.FC<MarketSentimentWidgetProps> = ({
  sentimentScore,
  trend,
  dominantToken,
  totalVolume24h,
}) => {
  return (
    <div className="p-4 bg-white rounded-2xl shadow-sm border border-gray-100">
      <h3 className="text-lg font-semibold mb-3">Market Sentiment</h3>
      <div className="flex items-center gap-4">
        <div
          className="flex items-center justify-center w-16 h-16 rounded-full text-white font-bold"
          style={{ backgroundColor: getSentimentColor(sentimentScore) }}
        >
          {clamp(sentimentScore)}%
        </div>
        <ul className="text-sm space-y-1">
          <li>
            <strong>Trend:</strong> {trend}
          </li>
          <li>
            <strong>Dominant Token:</strong> {dominantToken}
          </li>
          <li>
            <strong>24h Volume:</strong> {"$"}
            {Number.isFinite(totalVolume24h) ? totalVolume24h.toLocaleString() : "0"}
          </li>
        </ul>
      </div>
    </div>
  )
}

export default MarketSentimentWidget
