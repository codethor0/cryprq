/**
 * Generate support token for issue reporting
 * Format: CRYPRQ-{version}-{short-uuid}
 */

export const supportToken = (appVersion: string): string => {
  const uuid = crypto.randomUUID()
  const shortUuid = uuid.split('-')[0].toUpperCase()
  return `CRYPRQ-${appVersion}-${shortUuid}`
}

