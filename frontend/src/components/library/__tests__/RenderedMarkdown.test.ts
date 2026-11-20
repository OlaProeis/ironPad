// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import RenderedMarkdown from '../RenderedMarkdown.vue'

describe('RenderedMarkdown', () => {
  it('renders a markdown heading as h1', () => {
    const wrapper = mount(RenderedMarkdown, {
      props: { source: '# Title' },
    })
    expect(wrapper.find('h1').exists()).toBe(true)
    expect(wrapper.find('h1').text()).toBe('Title')
  })

  it('does not inject script tags or onerror handlers from raw HTML in source', () => {
    const wrapper = mount(RenderedMarkdown, {
      props: {
        source: '<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>',
      },
    })
    const html = wrapper.html()
    expect(wrapper.find('script').exists()).toBe(false)
    expect(html).not.toMatch(/\bonerror\s*=/i)
  })

  it('renders empty source without throwing', () => {
    expect(() => mount(RenderedMarkdown, { props: { source: '' } })).not.toThrow()
    const wrapper = mount(RenderedMarkdown, { props: { source: '' } })
    expect(wrapper.get('[data-testid="rendered-markdown"]').exists()).toBe(true)
    expect(wrapper.find('.rendered-markdown__body').text()).toBe('')
  })

  it('leaves relative images unchanged when no file path is provided', () => {
    const wrapper = mount(RenderedMarkdown, {
      props: { source: '![Banner](docs/graphics/ironpad-banner.png)' },
    })
    expect(wrapper.get('img').attributes('src')).toBe('docs/graphics/ironpad-banner.png')
  })

  it('points images and file links at the original file', async () => {
    const wrapper = mount(RenderedMarkdown, {
      props: {
        source:
          '![Banner](docs/graphics/ironpad-banner.png)\n\n[Index](docs/index.md)\n\n[Web](https://example.com)',
        basePath: 'C:/repo/ai-context.md',
        linkRoot: 'C:/repo',
      },
    })
    expect(wrapper.get('img').attributes('src')).toBe(
      '/api/library/media?path=' + encodeURIComponent('c:/repo/docs/graphics/ironpad-banner.png')
    )
    const local = wrapper.get('a[data-library-path]')
    expect(local.attributes('data-library-path')).toBe('c:/repo/docs/index.md')
    await local.trigger('click')
    expect(wrapper.emitted('open-file')?.[0]).toEqual(['c:/repo/docs/index.md'])
    const web = wrapper.get('a[href="https://example.com"]')
    expect(web.attributes('target')).toBe('_blank')
  })
})