/**
 * Exponential Moving Average (EMA) smoothing
 * Reduces flicker in time-series charts
 */

export function ema(xs: number[], alpha = 0.2): number[] {
  if (!xs.length) return xs
  
  const out: number[] = [xs[0]]
  
  for (let i = 1; i < xs.length; i++) {
    out[i] = alpha * xs[i] + (1 - alpha) * out[i - 1]
  }
  
  return out
}

