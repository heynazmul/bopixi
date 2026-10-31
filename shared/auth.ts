export interface LocalUser {
  id: string
  name: string
  email: string
  role: 'admin'
  emailVerified: boolean
  image?: string | null
}

export const DEFAULT_USER: LocalUser = {
  id: 'local-user',
  name: 'User',
  email: 'user@local.dev',
  role: 'admin',
  emailVerified: true,
  image: null,
}

export const DEFAULT_SESSION = {
  id: 'local-session',
  userId: DEFAULT_USER.id,
  token: 'local-token',
  expiresAt: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString(),
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
}
