import { describe, it, expect } from 'vitest'
import { useTimeSeries } from '../../composables/useTimeSeries'

describe('useTimeSeries composable', () => {
  it('should initialize empty arrays', () => {
    const { cpuHistory, ramHistory } = useTimeSeries()
    expect(cpuHistory.value).toEqual([])
    expect(ramHistory.value).toEqual([])
  })

  it('should push values and respect maxPoints', () => {
    const { cpuHistory, ramHistory, push } = useTimeSeries(3)
    push(10, 20)
    expect(cpuHistory.value.length).toBe(1)
    expect(cpuHistory.value[0].y).toBe(10)
    expect(ramHistory.value[0].y).toBe(20)

    push(30, 40)
    push(50, 60)
    expect(cpuHistory.value.length).toBe(3)

    // Over limits
    push(70, 80)
    expect(cpuHistory.value.length).toBe(3)
    expect(cpuHistory.value[0].y).toBe(30) // shifted
    expect(cpuHistory.value[2].y).toBe(70)
  })

  it('should clear history', () => {
    const { cpuHistory, ramHistory, push, clear } = useTimeSeries()
    push(10, 20)
    expect(cpuHistory.value.length).toBe(1)
    clear()
    expect(cpuHistory.value).toEqual([])
    expect(ramHistory.value).toEqual([])
  })
})
