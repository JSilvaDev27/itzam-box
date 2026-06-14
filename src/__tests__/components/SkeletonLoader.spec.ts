import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'

describe('SkeletonLoader component', () => {
  it('renders default full page skeleton when no variant is provided', () => {
    const wrapper = mount(SkeletonLoader)
    expect(wrapper.find('.skeleton-text-lg').exists()).toBe(true)
    expect(wrapper.findAll('.skeleton-card').length).toBe(4)
  })

  it('renders metric-grid variant with custom count', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'metric-grid',
        count: 3,
      },
    })
    expect(wrapper.find('.metrics-grid').exists()).toBe(true)
    expect(wrapper.findAll('.skeleton-card').length).toBe(3)
  })

  it('renders table-row variant with custom rows count', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'table-row',
        rows: 6,
      },
    })
    expect(wrapper.find('.section').exists()).toBe(true)
    expect(wrapper.findAll('.skeleton-row').length).toBe(6)
  })

  it('renders chart variant correctly', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'chart',
      },
    })
    expect(wrapper.find('[style*="height:196px"]').exists()).toBe(true)
  })

  it('renders card variant correctly', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'card',
        rows: 2,
      },
    })
    expect(wrapper.findAll('.skeleton-row').length).toBe(2)
  })

  it('renders text variant correctly', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'text',
        lines: 3,
      },
    })
    expect(wrapper.findAll('.skeleton-text-md').length).toBe(3)
  })

  it('renders header variant correctly', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'header',
      },
    })
    expect(wrapper.find('.skeleton-text-lg').exists()).toBe(true)
  })

  it('renders drawer variant with custom count', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'drawer',
        count: 3,
      },
    })
    expect(wrapper.findAll('.skeleton-text-xs').length).toBeGreaterThanOrEqual(3)
  })

  it('renders form variant with custom count', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'form',
        count: 2,
      },
    })
    // Form variant renders skeleton-text-xs elements for labels
    expect(wrapper.findAll('.skeleton-text-xs').length).toBeGreaterThanOrEqual(2)
  })

  it('renders list variant with custom count', () => {
    const wrapper = mount(SkeletonLoader, {
      props: {
        variant: 'list',
        count: 3,
      },
    })
    // List items have 32x32 icon skeletons
    expect(wrapper.findAll('[style*="width:32px"]').length).toBe(3)
  })
})
