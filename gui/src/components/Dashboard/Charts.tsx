import React, { useMemo, useState } from 'react'
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, Legend, ReferenceLine } from 'recharts'
import { ema } from '@/utils/ema'
import { useAppStore } from '@/store/useAppStore'

interface ChartDataPoint {
  ts: number
  inRate: number
  outRate: number
  latency: number
  timeLabel: string
}

type Units = 'bytes' | 'KB' | 'MB'

export const Charts: React.FC = () => {
  const points = useAppStore(s => s.metricsSeries60s)
  const alpha = useAppStore(s => s.settings.chartSmoothing ?? 0.2)
  const [throughputUnits, setThroughputUnits] = useState<Units>('KB')
  const LATENCY_ALERT_THRESHOLD = 250 // ms

  // Derive rates and apply EMA smoothing
  const data = useMemo((): ChartDataPoint[] => {
    if (!points || points.length < 2) return []

    const out: Omit<ChartDataPoint, 'timeLabel'>[] = []
    
    for (let i = 1; i < points.length; i++) {
      const dt = Math.max(1, (points[i].ts - points[i - 1].ts) / 1000) // seconds
      const inRate = Math.max(0, (points[i].bytesIn - points[i - 1].bytesIn) / dt)
      const outRate = Math.max(0, (points[i].bytesOut - points[i - 1].bytesOut) / dt)
      const latency = points[i].latencyMs ?? 0
      
      out.push({
        ts: points[i].ts,
        inRate,
        outRate,
        latency,
      })
    }

    if (out.length === 0) return []

    // Apply EMA smoothing
    const sm = (xs: number[]) => ema(xs, alpha)
    const inS = sm(out.map(p => p.inRate))
    const outS = sm(out.map(p => p.outRate))
    const latS = sm(out.map(p => p.latency))

    // Format time labels and convert units
    return out.map((p, i) => {
      const date = new Date(p.ts)
      const minutes = date.getMinutes()
      const seconds = date.getSeconds()
      
      // Convert rates based on selected units (rates are already in bytes/s)
      let inRate = inS[i]
      let outRate = outS[i]
      if (throughputUnits === 'KB') {
        inRate = inRate / 1024
        outRate = outRate / 1024
      } else if (throughputUnits === 'MB') {
        inRate = inRate / (1024 * 1024)
        outRate = outRate / (1024 * 1024)
      }
      // 'bytes' unit: keep as-is
      
      return {
        ...p,
        inRate,
        outRate,
        latency: latS[i],
        timeLabel: `${minutes}:${seconds.toString().padStart(2, '0')}`,
      }
    })
  }, [points, alpha, throughputUnits])
  
  // Find max throughput for auto-scaling axis
  const maxThroughput = useMemo(() => {
    if (!data.length) return 0
    return Math.max(...data.map(d => Math.max(d.inRate, d.outRate)))
  }, [data])
  
  // Auto-scale Y-axis ticks
  const getYAxisDomain = (max: number) => {
    if (max === 0) return [0, 1]
    const rounded = Math.ceil(max * 1.1) // 10% padding
    return [0, rounded]
  }

  if (!data.length) {
    return (
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '32px',
        textAlign: 'center',
        color: '#B0B0B0',
        fontSize: '14px',
      }}>
        <p style={{ margin: '0 0 12px', fontSize: '16px', fontWeight: 500 }}>
          No data yet
        </p>
        <p style={{ margin: 0, fontSize: '14px', color: '#757575' }}>
          Connect to a peer to see live throughput and latency metrics
        </p>
      </div>
    )
  }

  const chartStyle = {
    backgroundColor: '#1E1E1E',
    borderRadius: '12px',
    padding: '24px',
    marginBottom: '24px',
  }

  return (
    <div>
      {/* Throughput Chart */}
      <div style={chartStyle}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <h3 style={{ margin: 0, fontSize: '18px', fontWeight: 600, color: '#E0E0E0' }}>
            Throughput (last 60s)
          </h3>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <span style={{ fontSize: '12px', color: '#B0B0B0' }}>Units:</span>
            {(['bytes', 'KB', 'MB'] as Units[]).map(unit => (
              <button
                key={unit}
                onClick={() => setThroughputUnits(unit)}
                style={{
                  padding: '4px 12px',
                  backgroundColor: throughputUnits === unit ? '#1DE9B6' : '#333',
                  color: throughputUnits === unit ? '#000' : '#E0E0E0',
                  border: 'none',
                  borderRadius: '4px',
                  fontSize: '12px',
                  cursor: 'pointer',
                  fontWeight: throughputUnits === unit ? 600 : 400,
                }}
              >
                {unit}/s
              </button>
            ))}
          </div>
        </div>
        <ResponsiveContainer width="100%" height={220}>
          <LineChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
            <XAxis 
              dataKey="timeLabel" 
              stroke="#757575"
              style={{ fontSize: '12px' }}
            />
            <YAxis 
              stroke="#757575"
              style={{ fontSize: '12px' }}
              domain={getYAxisDomain(maxThroughput)}
              label={{ 
                value: `${throughputUnits}/s`, 
                angle: -90, 
                position: 'insideLeft', 
                style: { fill: '#B0B0B0' } 
              }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#121212',
                border: '1px solid #333',
                borderRadius: '6px',
                color: '#E0E0E0',
              }}
              labelStyle={{ color: '#B0B0B0' }}
            />
            <Legend />
            <Line 
              type="monotone" 
              dataKey="inRate" 
              stroke="#1DE9B6" 
              strokeWidth={2}
              dot={false}
              name="Bytes In"
            />
            <Line 
              type="monotone" 
              dataKey="outRate" 
              stroke="#2196F3" 
              strokeWidth={2}
              dot={false}
              name="Bytes Out"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Latency Chart */}
      <div style={{
        ...chartStyle,
        background: data.some(d => d.latency > LATENCY_ALERT_THRESHOLD)
          ? 'linear-gradient(to right, rgba(255, 152, 0, 0.05) 0%, rgba(255, 152, 0, 0.05) 100%)'
          : undefined,
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600, color: '#E0E0E0' }}>
          Latency (last 60s)
          {data.some(d => d.latency > LATENCY_ALERT_THRESHOLD) && (
            <span style={{ 
              marginLeft: '12px', 
              fontSize: '12px', 
              color: '#FF9800',
              fontWeight: 400,
            }}>
              ⚠️ High latency detected (&gt;{LATENCY_ALERT_THRESHOLD}ms)
            </span>
          )}
        </h3>
        <ResponsiveContainer width="100%" height={220}>
          <LineChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
            <XAxis 
              dataKey="timeLabel" 
              stroke="#757575"
              style={{ fontSize: '12px' }}
            />
            <YAxis 
              stroke="#757575"
              style={{ fontSize: '12px' }}
              label={{ value: 'ms', angle: -90, position: 'insideLeft', style: { fill: '#B0B0B0' } }}
            />
            <ReferenceLine 
              y={LATENCY_ALERT_THRESHOLD} 
              stroke="#FF9800" 
              strokeDasharray="3 3"
              label={{ value: `${LATENCY_ALERT_THRESHOLD}ms threshold`, position: 'topRight', fill: '#FF9800' }}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#121212',
                border: '1px solid #333',
                borderRadius: '6px',
                color: '#E0E0E0',
              }}
              labelStyle={{ color: '#B0B0B0' }}
            />
            <Line 
              type="monotone" 
              dataKey="latency" 
              stroke="#FF9800" 
              strokeWidth={2}
              dot={false}
              name="Latency"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}

