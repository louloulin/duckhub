/**
 * 认证工具函数
 * 用于管理用户认证状态和令牌
 */

export interface AuthTokens {
  access_token: string
  refresh_token?: string
  token_type: string
  expires_in: number
}

export interface UserInfo {
  id: string
  username: string
  email: string
  display_name: string
  roles: string[]
  permissions: string[]
}

/**
 * 获取存储的认证令牌
 */
export function getAuthToken(): string | null {
  return localStorage.getItem('auth_token')
}

/**
 * 设置认证令牌
 */
export function setAuthToken(token: string): void {
  localStorage.setItem('auth_token', token)
}

/**
 * 移除认证令牌
 */
export function removeAuthToken(): void {
  localStorage.removeItem('auth_token')
  localStorage.removeItem('user_info')
}

/**
 * 获取用户信息
 */
export function getUserInfo(): UserInfo | null {
  const userInfoStr = localStorage.getItem('user_info')
  if (userInfoStr) {
    try {
      return JSON.parse(userInfoStr)
    } catch (error) {
      console.error('解析用户信息失败:', error)
      return null
    }
  }
  return null
}

/**
 * 设置用户信息
 */
export function setUserInfo(userInfo: UserInfo): void {
  localStorage.setItem('user_info', JSON.stringify(userInfo))
}

/**
 * 检查是否已认证
 */
export function isAuthenticated(): boolean {
  const token = getAuthToken()
  return !!token
}

/**
 * 自动获取新的认证令牌
 */
async function getNewToken(): Promise<string | null> {
  try {
    const response = await fetch('/api/v1/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        username: 'admin',
        password: 'admin123'
      }),
    })

    if (response.ok) {
      const data = await response.json()
      if (data.success) {
        return data.data.access_token
      }
    }

    return null
  } catch (error) {
    console.error('获取新令牌失败:', error)
    return null
  }
}

/**
 * 初始化认证状态
 * 如果没有令牌，则自动获取新的admin令牌进行开发测试
 */
export async function initializeAuth(): Promise<void> {
  const existingToken = getAuthToken()

  if (!existingToken) {
    console.log('正在获取认证令牌...')
    const newToken = await getNewToken()

    if (newToken) {
      setAuthToken(newToken)

      // 设置默认用户信息
      const defaultUserInfo: UserInfo = {
        id: '6aef00d1-3a83-4087-89c4-995d2b481589',
        username: 'admin',
        email: 'admin@duckhub.com',
        display_name: 'Administrator',
        roles: ['admin'],
        permissions: ['*']
      }

      setUserInfo(defaultUserInfo)

      console.log('已初始化认证状态 (开发模式)')
    } else {
      console.error('无法获取认证令牌')
    }
  }
}

/**
 * 登录函数
 */
export async function login(username: string, password: string): Promise<boolean> {
  try {
    const response = await fetch('/api/v1/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    })
    
    if (response.ok) {
      const data = await response.json()
      if (data.success) {
        setAuthToken(data.data.access_token)
        setUserInfo(data.data.user)
        return true
      }
    }
    
    return false
  } catch (error) {
    console.error('登录失败:', error)
    return false
  }
}

/**
 * 登出函数
 */
export function logout(): void {
  removeAuthToken()
  window.location.href = '/'
}
