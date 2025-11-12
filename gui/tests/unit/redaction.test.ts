import { redactSecrets } from '../../electron/main/logging'

describe('Redaction', () => {
  it('should redact bearer tokens', () => {
    const input = 'Authorization: bearer abc123def456'
    const output = redactSecrets(input)
    expect(output).toContain('bearer')
    expect(output).not.toContain('abc123def456')
    expect(output).toContain('***REDACTED***')
  })

  it('should redact token= values', () => {
    const input = 'token=secret123token456'
    const output = redactSecrets(input)
    expect(output).toContain('token=')
    expect(output).not.toContain('secret123token456')
    expect(output).toContain('***REDACTED***')
  })

  it('should redact privKey values', () => {
    const input = 'privKey=abc123'
    const output = redactSecrets(input)
    expect(output).toContain('privKey=')
    expect(output).not.toContain('abc123')
    expect(output).toContain('***REDACTED***')
  })

  it('should redact authorization headers', () => {
    const input = 'authorization: Bearer xyz789'
    const output = redactSecrets(input)
    expect(output).toContain('authorization')
    expect(output).not.toContain('xyz789')
    expect(output).toContain('***REDACTED***')
  })

  it('should handle multiple secrets in one string', () => {
    const input = 'token=secret1 bearer secret2 privKey=secret3'
    const output = redactSecrets(input)
    expect(output).not.toContain('secret1')
    expect(output).not.toContain('secret2')
    expect(output).not.toContain('secret3')
    expect(output.match(/\*\*\*REDACTED\*\*\*/g)?.length).toBeGreaterThanOrEqual(3)
  })

  it('should not redact non-secret content', () => {
    const input = 'This is a normal log message with no secrets'
    const output = redactSecrets(input)
    expect(output).toBe(input)
  })
})

