// widget: Embeddable activity widget (minified by `minify`) (commit platform, pkgs/widget)
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ActivityWidget from '../ActivityWidget.vue'

describe('ActivityWidget', () => {
  it('shows repo, weekly commits and CI state', () => {
    const wrapper = mount(ActivityWidget, { props: { repo: 'o/r', commits: 1, ci: 'failing' } })
    expect(wrapper.text()).toContain('o/r')
    expect(wrapper.text()).toContain('1 commit this week')
    expect(wrapper.attributes('data-ci')).toBe('failing')
  })

  it('pluralizes', () => {
    const wrapper = mount(ActivityWidget, { props: { repo: 'o/r', commits: 3, ci: 'passing' } })
    expect(wrapper.text()).toContain('3 commits this week')
  })
})
