import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ContextMenu from '../../components/shared/ContextMenu.vue'
import { useContextMenu } from '../../composables/useContextMenu'

describe('ContextMenu component', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('does not render when state is hidden', () => {
    const { state } = useContextMenu()
    state.value.visible = false
    const wrapper = mount(ContextMenu, {
      global: { stubs: { Teleport: true } }
    })
    expect(wrapper.find('.ctx-menu').exists()).toBe(false)
  })

  it('renders correctly when visible and triggers actions', async () => {
    const { state } = useContextMenu()
    const actionSpy = vi.fn()
    state.value = {
      visible: true,
      x: 150,
      y: 200,
      items: [
        { id: '1', label: 'Item 1', icon: 'fa-play', action: actionSpy },
        { id: 'div1', label: '', divider: true },
        { id: '2', label: 'Disabled Item', disabled: true }
      ]
    }

    const wrapper = mount(ContextMenu, {
      global: { stubs: { Teleport: true } }
    })

    const menu = wrapper.find('.ctx-menu')
    expect(menu.exists()).toBe(true)
    expect(menu.attributes('style')).toContain('left: 150px')
    expect(menu.attributes('style')).toContain('top: 200px')

    const items = wrapper.findAll('.ctx-item')
    expect(items.length).toBe(2)
    expect(items[0].text()).toContain('Item 1')
    expect(items[1].classes()).toContain('ctx-disabled')

    // Click item
    await items[0].trigger('click')
    expect(actionSpy).toHaveBeenCalled()
  })

  it('closes on click outside, scroll, or resize and cleans up on unmount', () => {
    const { state } = useContextMenu()
    state.value.visible = true
    const wrapper = mount(ContextMenu, {
      global: { stubs: { Teleport: true } }
    })
    
    // Click outside
    const div = document.createElement('div')
    document.body.appendChild(div)
    div.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    document.body.removeChild(div)
    expect(state.value.visible).toBe(false)
    
    // Scroll
    state.value.visible = true
    document.dispatchEvent(new Event('scroll'))
    expect(state.value.visible).toBe(false)

    // Resize
    state.value.visible = true
    window.dispatchEvent(new Event('resize'))
    expect(state.value.visible).toBe(false)

    // Unmount
    wrapper.unmount()
  })
})
