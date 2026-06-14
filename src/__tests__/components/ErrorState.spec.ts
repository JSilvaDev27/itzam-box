import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ErrorState from '../../components/shared/ErrorState.vue'

describe('ErrorState component', () => {
  beforeEach(() => {
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
      writable: true,
      configurable: true,
    })
  })

  it('renders default values correctly', () => {
    const wrapper = mount(ErrorState, {
      props: {
        message: 'Something went wrong'
      }
    })
    expect(wrapper.find('.error-state-title').text()).toBe('Something went wrong')
    expect(wrapper.find('.error-state-suggestion').exists()).toBe(false)
  })

  it('renders suggestion, detail and custom icon', async () => {
    const wrapper = mount(ErrorState, {
      props: {
        message: 'Failed to connect',
        suggestion: 'Check connection',
        detail: 'Timeout after 10s',
        icon: 'fa-solid fa-server'
      }
    })
    expect(wrapper.find('.error-state-icon i').classes()).toContain('fa-server')
    expect(wrapper.find('.error-state-suggestion').text()).toBe('Check connection')
    
    // Toggle details
    expect(wrapper.find('.error-detail-text').exists()).toBe(false)
    await wrapper.find('.error-detail-toggle').trigger('click')
    expect(wrapper.find('.error-detail-text').text()).toBe('Timeout after 10s')
  })

  it('emits retry event on button click', async () => {
    const wrapper = mount(ErrorState, {
      props: {
        message: 'Error'
      }
    })
    await wrapper.find('.btn-primary').trigger('click')
    expect(wrapper.emitted('retry')).toBeTruthy()
  })

  it('copies error details on copy click', async () => {
    const wrapper = mount(ErrorState, {
      props: {
        message: 'Error',
        detail: 'Internal trace'
      }
    })
    const copyBtn = wrapper.findAll('.btn-ghost').find(b => b.text().includes('Copy Error'))
    expect(copyBtn).toBeDefined()
    await copyBtn?.trigger('click')
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('Internal trace')
  })

  it('falls back to document.execCommand when clipboard API fails', async () => {
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: vi.fn().mockRejectedValue(new Error('Blocked')),
      },
      writable: true,
      configurable: true,
    })
    const execMock = vi.fn().mockReturnValue(true)
    Object.defineProperty(document, 'execCommand', {
      value: execMock,
      writable: true,
      configurable: true,
    })

    const wrapper = mount(ErrorState, {
      props: {
        message: 'Error',
        detail: 'Fallback trace'
      }
    })
    const copyBtn = wrapper.findAll('.btn-ghost').find(b => b.text().includes('Copy Error'))
    await copyBtn?.trigger('click')
    expect(execMock).toHaveBeenCalledWith('copy')
  })
})
