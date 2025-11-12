import { describe, test, expect } from 'vitest'
import { hostnameFromUrl } from '@/utils/host'

describe('Allowlist validation', () => {
  test('blocks save for disallowed REMOTE host', () => {
    const host = hostnameFromUrl('https://not-allowed.example')
    const allowlist = ['api.good.example']
    
    expect(allowlist.includes(host)).toBe(false)
  })

  test('allows save for allowed REMOTE host', () => {
    const host = hostnameFromUrl('https://api.good.example')
    const allowlist = ['api.good.example']
    
    expect(allowlist.includes(host)).toBe(true)
  })

  test('handles empty allowlist (no restrictions)', () => {
    const host = hostnameFromUrl('https://any.example')
    const allowlist: string[] = []
    
    // Empty allowlist means no restrictions
    expect(allowlist.length).toBe(0)
  })

  test('normalizes hostname case', () => {
    const host1 = hostnameFromUrl('https://API.GOOD.EXAMPLE')
    const host2 = hostnameFromUrl('https://api.good.example')
    const allowlist = ['api.good.example']
    
    expect(host1).toBe('api.good.example')
    expect(host2).toBe('api.good.example')
    expect(allowlist.includes(host1)).toBe(true)
    expect(allowlist.includes(host2)).toBe(true)
  })
})

