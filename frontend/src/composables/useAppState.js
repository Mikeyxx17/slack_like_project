import { ref, watch } from 'vue'

// ── 全局单例状态 ──
const username = ref('')
const isJoined = ref(false)
const currentChannel = ref('general')
const theme = ref(localStorage.getItem('slackchat-theme') || 'dark')
const showCreateModal = ref(false)

// 主题持久化 + 应用到 <html>
watch(theme, (val) => {
  localStorage.setItem('slackchat-theme', val)
  document.documentElement.setAttribute('data-theme', val)
}, { immediate: true })

export function useAppState() {
  const join = (name) => {
    username.value = name.trim()
    if (username.value) {
      isJoined.value = true
    }
  }

  const switchChannel = (channel) => {
    currentChannel.value = channel
  }

  return {
    username,
    isJoined,
    currentChannel,
    theme,
    showCreateModal,
    join,
    switchChannel,
  }
}
