import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import AnimatedCounter from '../../components/shared/AnimatedCounter.vue'

describe('AnimatedCounter component', () => {
  it('renders initial value', () => {
    const wrapper = mount(AnimatedCounter, {
      props: {
        value: 42,
      },
    })
    expect(wrapper.text()).toBe('42')
  })

  it('renders with prefix and suffix', () => {
    const wrapper = mount(AnimatedCounter, {
      props: {
        value: 75,
        prefix: '$',
        suffix: ' USD',
      },
    })
    expect(wrapper.text()).toBe('$75 USD')
  })

  it('renders with decimals', () => {
    const wrapper = mount(AnimatedCounter, {
      props: {
        value: 3.14,
        decimals: 2,
      },
    })
    expect(wrapper.text()).toBe('3.14')
  })

  it('renders the counter span', () => {
    const wrapper = mount(AnimatedCounter, {
      props: {
        value: 100,
      },
    })
    expect(wrapper.find('span').exists()).toBe(true)
  })
})
