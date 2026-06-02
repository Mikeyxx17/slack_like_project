import { ref, watch } from 'vue'

// ── 全局单例状态 ──
const username = ref('')
const email = ref('')
const token = ref(localStorage.getItem('slackchat-token') || '')
const isJoined = ref(false)
const currentChannel = ref('general')
const theme = ref(localStorage.getItem('slackchat-theme') || 'dark')
const showCreateModal = ref(false)
const authError = ref('')
const initializing = ref(true)

// 主题持久化 + 应用到 <html>
watch(theme, (val) => {
  localStorage.setItem('slackchat-theme', val)
  document.documentElement.setAttribute('data-theme', val)
}, { immediate: true })

// token 持久化
watch(token, (val) => {
  if (val) {
    localStorage.setItem('slackchat-token', val)
  } else {
    localStorage.removeItem('slackchat-token')
  }
})

// ── 启动时尝试恢复会话 ──
const initAuth = async () => {
  const saved = localStorage.getItem('slackchat-token')
  if (!saved) {
    initializing.value = false
    return
  }
  try {
    const res = await fetch('/api/me', {
      headers: { Authorization: `Bearer ${saved}` },
    })
    if (res.ok) {
      const data = await res.json()
      token.value = saved
      username.value = data.username
      email.value = data.email
      isJoined.value = true
    } else {
      localStorage.removeItem('slackchat-token')
      token.value = ''
    }
  } catch {
    // 网络不通时不清除 token，下次刷新再试
  }
  initializing.value = false
}

initAuth()

export function useAppState() {
  // ── 注册 ──
  const register = async (regUsername, regEmail, password) => {
    authError.value = ''
    try {
      const res = await fetch('/api/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: regUsername, email: regEmail, password }),
      })
      if (res.status === 201) {
        return { ok: true }
      }
      if (res.status === 409) {
        authError.value = '用户名或邮箱已被注册'
        return { ok: false, error: '用户名或邮箱已被注册' }
      }
      authError.value = '注册失败，请稍后重试'
      return { ok: false, error: '注册失败' }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  // ── 登录 ──
  const login = async (loginEmail, password) => {
    authError.value = ''
    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: loginEmail, password }),
      })
      if (res.ok) {
        const data = await res.json()
        token.value = data.token
        username.value = data.username
        email.value = loginEmail
        isJoined.value = true
        return { ok: true }
      }
      if (res.status === 401) {
        authError.value = '邮箱或密码错误'
        return { ok: false, error: '邮箱或密码错误' }
      }
      authError.value = '登录失败，请稍后重试'
      return { ok: false, error: '登录失败' }
    } catch {
      authError.value = '网络错误，请检查连接'
      return { ok: false, error: '网络错误' }
    }
  }

  // ── 快速体验（保留旧功能）──
  const join = (name) => {
    username.value = name.trim()
    if (username.value) {
      token.value = ''
      email.value = ''
      isJoined.value = true
    }
  }

  // ── 登出 ──
  const logout = () => {
    token.value = ''
    username.value = ''
    email.value = ''
    isJoined.value = false
    currentChannel.value = 'general'
  }

  const switchChannel = (channel) => {
    currentChannel.value = channel
  }

  return {
    username,
    email,
    token,
    isJoined,
    currentChannel,
    theme,
    showCreateModal,
    authError,
    initializing,
    login,
    register,
    join,
    logout,
    switchChannel,
  }
}
