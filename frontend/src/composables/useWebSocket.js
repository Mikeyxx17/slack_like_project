import { ref, watch, onUnmounted } from 'vue'
import { useAppState } from './useAppState'

// ── 全局单例状态 ──
const messages = ref([])
const connected = ref(false)
let socket = null

export function useWebSocket() {
  const { currentChannel, isJoined } = useAppState()

  const connect = () => {
    disconnect()
    if (!isJoined.value) return

    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = protocol + '//' + location.host + '/ws/' + currentChannel.value
    socket = new WebSocket(url)

    socket.onopen = () => {
      connected.value = true
    }

    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data)
        messages.value.push(msg)
      } catch (e) {
        console.error('消息解析失败:', e)
      }
    }

    socket.onclose = () => {
      connected.value = false
    }

    socket.onerror = () => {
      connected.value = false
    }
  }

  const disconnect = () => {
    if (socket) {
      socket.close()
      socket = null
    }
    connected.value = false
  }

  const sendMessage = (content) => {
    if (!socket || !content.trim()) return
    const { username } = useAppState()
    socket.send(JSON.stringify({
      channel: currentChannel.value,
      username: username.value,
      content: content.trim(),
    }))
  }

  // 频道切换时清空消息并重连
  watch(currentChannel, () => {
    messages.value = []
    if (isJoined.value) {
      connect()
    }
  })

  // 加入聊天时建立连接
  watch(isJoined, (joined) => {
    if (joined) {
      connect()
    } else {
      disconnect()
    }
  })

  onUnmounted(() => {
    disconnect()
  })

  return { messages, connected, sendMessage, connect, disconnect }
}
