import { ref, watch } from 'vue'
import { useAppState } from './useAppState'
import { useChannels } from './useChannels'

// ── 全局单例状态 ──
const messages = ref([])
const connected = ref(false)
let socket = null
let initialized = false

export function useWebSocket() {
  const { currentChannel, isJoined } = useAppState()
  const { fetchChannels } = useChannels()

  const connect = () => {
    disconnect()
    if (!isJoined.value) return

    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = protocol + '//' + location.host + '/ws/' + currentChannel.value
    const ws = new WebSocket(url)
    socket = ws

    ws.onopen = () => {
      connected.value = true
      fetchChannels()
    }

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data)
        messages.value.push(msg)
      } catch (e) {
        console.error('消息解析失败:', e)
      }
    }

    ws.onclose = () => {
      if (socket === ws) socket = null
      connected.value = false
    }

    ws.onerror = () => {
      if (socket === ws) socket = null
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
    if (!socket || socket.readyState !== WebSocket.OPEN || !content.trim()) return
    const { username } = useAppState()
    socket.send(JSON.stringify({
      channel: currentChannel.value,
      username: username.value,
      content: content.trim(),
    }))
  }

  if (!initialized) {
    initialized = true

    watch(currentChannel, () => {
      messages.value = []
      if (isJoined.value) {
        connect()
      }
    })

    watch(isJoined, (joined) => {
      if (joined) {
        connect()
      } else {
        disconnect()
      }
    })

    // watcher 注册时 isJoined 可能已经是 true（ChatLayout 在登录后才挂载）
    if (isJoined.value) {
      connect()
    }
  }

  return { messages, connected, sendMessage, connect, disconnect }
}
