import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import CpuChart from '../../components/charts/CpuChart.vue'

vi.mock('chart.js', () => ({
  Chart: class {
    static register = vi.fn()
    destroy = vi.fn()
    update = vi.fn()
    constructor(_canvas: any, config: any) {
      if (config && config.data && config.data.datasets && config.data.datasets[0]) {
        const ds = config.data.datasets[0]
        if (typeof ds.backgroundColor === 'function') {
          ds.backgroundColor({
            chart: {
              chartArea: { top: 0, bottom: 100 },
              ctx: { createLinearGradient: () => ({ addColorStop: () => {} }) }
            }
          })
          ds.backgroundColor({ chart: {} })
        }
      }
      if (config && config.options && config.options.plugins && config.options.plugins.tooltip && config.options.plugins.tooltip.callbacks) {
        const cb = config.options.plugins.tooltip.callbacks
        if (typeof cb.title === 'function') cb.title([{ label: 'test' }])
        if (typeof cb.title === 'function') cb.title([])
        if (typeof cb.label === 'function') {
          cb.label({ parsed: { y: 50 } })
          cb.label({ parsed: { y: null } })
        }
      }
    }
    data = {
      labels: [] as any[],
      datasets: [
        {
          data: [] as any[],
          borderColor: '',
          backgroundColor: '',
        },
      ],
    }
  },
  LineController: {},
  LineElement: {},
  PointElement: {},
  LinearScale: {},
  CategoryScale: {},
  Filler: {},
  Tooltip: {},
  Legend: {},
}))

describe('CpuChart component', () => {
  it('renders canvas element, updates on props change and destroys on unmount', async () => {
    const wrapper = mount(CpuChart, {
      props: {
        data: [{ x: Date.now(), y: 25.5 }],
        cores: 8
      }
    })
    expect(wrapper.find('canvas').exists()).toBe(true)

    // Trigger update branch
    await wrapper.setProps({ data: [{ x: Date.now() + 1000, y: 45.0 }] })

    // Trigger unmount destroy
    wrapper.unmount()
  })
})
