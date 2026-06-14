import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../../components/shared/EmptyState.vue'

describe('EmptyState component', () => {
  it('renders title and description correctly', () => {
    const wrapper = mount(EmptyState, {
      props: {
        title: 'No containers found',
        description: 'Create a container to get started',
      },
    })

    expect(wrapper.find('.empty-state-title').text()).toBe('No containers found')
    expect(wrapper.find('.empty-state-desc').text()).toBe('Create a container to get started')
  })

  it('emits action event when action button is clicked', async () => {
    const wrapper = mount(EmptyState, {
      props: {
        title: 'Empty List',
        actionLabel: 'Add Item',
      },
    })

    const button = wrapper.find('.btn-primary')
    expect(button.exists()).toBe(true)
    await button.trigger('click')

    expect(wrapper.emitted().action).toBeTruthy()
  })

  it('emits secondary event when secondary button is clicked', async () => {
    const wrapper = mount(EmptyState, {
      props: {
        title: 'Empty List',
        secondaryLabel: 'Cancel',
      },
    })

    const button = wrapper.find('.btn-secondary')
    expect(button.exists()).toBe(true)
    await button.trigger('click')

    expect(wrapper.emitted().secondary).toBeTruthy()
  })
})
