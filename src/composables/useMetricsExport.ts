// ItzamBox — Metrics Export Composable (CSV / JSON / PNG)
// Copyright (C) 2026 SodigTech — GPL-3.0

import type { MetricsDataPoint } from './useMetricsHistory'

export type ExportFormat = 'csv' | 'json' | 'png'

export function useMetricsExport() {
  function formatTimestamp(ts: number): string {
    const d = new Date(ts)
    return d.toISOString()
  }

  function generateCSV(
    data: MetricsDataPoint[],
    secondaryData: MetricsDataPoint[] | null,
    label: string,
    secondaryLabel: string | null,
  ): string {
    const header = secondaryData
      ? `timestamp,${label},${secondaryLabel}`
      : `timestamp,${label}`

    const rows = data.map((p, i) => {
      const ts = formatTimestamp(p.x)
      const primary = p.y.toFixed(2)
      const secondary = secondaryData?.[i]?.y.toFixed(2) ?? ''
      return secondaryData ? `${ts},${primary},${secondary}` : `${ts},${primary}`
    })

    return `\uFEFF${header}\n${rows.join('\n')}`
  }

  function generateJSON(
    data: MetricsDataPoint[],
    secondaryData: MetricsDataPoint[] | null,
    label: string,
    secondaryLabel: string | null,
    metricType: string,
    timeRange: string,
  ): string {
    const points = data.map((p, i) => ({
      timestamp: formatTimestamp(p.x),
      unix_ms: p.x,
      [label]: p.y,
      ...(secondaryData?.[i] ? { [secondaryLabel ?? 'secondary']: secondaryData[i].y } : {}),
    }))

    const payload = {
      metric: metricType,
      timeRange,
      generatedAt: new Date().toISOString(),
      totalPoints: points.length,
      data: points,
    }

    return JSON.stringify(payload, null, 2)
  }

  function downloadBlob(content: string, filename: string, mime: string) {
    const blob = new Blob([content], { type: mime })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    a.click()
    URL.revokeObjectURL(url)
  }

  function exportCSV(
    data: MetricsDataPoint[],
    secondaryData: MetricsDataPoint[] | null,
    metricType: string,
  ) {
    const label = metricType === 'network' ? 'RX_MBps' : metricType === 'disk' ? 'Read_MBps' : `${metricType}_percent`
    const secondaryLabel = metricType === 'network' ? 'TX_MBps' : metricType === 'disk' ? 'Write_MBps' : null
    const csv = generateCSV(data, secondaryData, label, secondaryLabel)
    const dateStr = new Date().toISOString().slice(0, 10)
    downloadBlob(csv, `itzambox_${metricType}_${dateStr}.csv`, 'text/csv;charset=utf-8')
  }

  function exportJSON(
    data: MetricsDataPoint[],
    secondaryData: MetricsDataPoint[] | null,
    metricType: string,
    timeRange: string,
  ) {
    const label = metricType === 'network' ? 'rx_mbps' : metricType === 'disk' ? 'read_mbps' : `${metricType}_percent`
    const secondaryLabel = metricType === 'network' ? 'tx_mbps' : metricType === 'disk' ? 'write_mbps' : null
    const json = generateJSON(data, secondaryData, label, secondaryLabel, metricType, timeRange)
    const dateStr = new Date().toISOString().slice(0, 10)
    downloadBlob(json, `itzambox_${metricType}_${dateStr}.json`, 'application/json')
  }

  function exportPNG(chartElementId: string, metricType: string) {
    const el = document.getElementById(chartElementId)
    if (!el) return

    // Use SVG element directly
    const svgEl = el.querySelector('svg')
    if (!svgEl) return

    const clone = svgEl.cloneNode(true) as SVGElement
    const serializer = new XMLSerializer()
    const svgStr = serializer.serializeToString(clone)
    const svgBlob = new Blob([svgStr], { type: 'image/svg+xml;charset=utf-8' })
    const url = URL.createObjectURL(svgBlob)

    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')
    const img = new Image()

    img.onload = () => {
      canvas.width = img.width * 2
      canvas.height = img.height * 2
      ctx!.scale(2, 2)
      // Fill background
      ctx!.fillStyle = '#12161f'
      ctx!.fillRect(0, 0, img.width, img.height)
      ctx!.drawImage(img, 0, 0)
      canvas.toBlob((blob) => {
        if (!blob) return
        const pngUrl = URL.createObjectURL(blob)
        const a = document.createElement('a')
        const dateStr = new Date().toISOString().slice(0, 10)
        a.href = pngUrl
        a.download = `itzambox_${metricType}_${dateStr}.png`
        a.click()
        URL.revokeObjectURL(pngUrl)
      }, 'image/png')
      URL.revokeObjectURL(url)
    }

    img.src = url
  }

  return { exportCSV, exportJSON, exportPNG }
}
