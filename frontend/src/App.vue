<!--
  App.vue — 主应用组件（登录界面 + 聊天主界面）

  与后端交互点：
    1. WebSocket 连接（实时聊天核心）
       joinChat() → new WebSocket(`ws://host/ws/${channel}`)
       对应后端 handler.rs → ws_handler() → handle_socket()
       数据流:
         发送: 用户输入 → socket.send(JSON) → 后端 receiver → 存 DB → broadcast
         接收: broadcast → rx.recv → sender.send(JSON) → socket.onmessage → messages.push

    2. REST API
       fetchChannels() → fetch('/api/channels')
       对应后端 handler.rs → get_channels() → GET /api/channels
       用于加载侧边栏频道列表

    3. 频道切换
       switchChannel() → 关闭旧 WebSocket → 更新 currentChannel → 重新 joinChat()
       每次切换都是一次完整的「断开-重连」周期，后端自动推送该频道的历史消息
-->
<template>
  <!-- 登录界面 -->
  <div v-if="!isJoined" class="min-h-screen flex items-center justify-center relative overflow-hidden"
    style="background: linear-gradient(135deg, #0f0c29, #302b63, #24243e)">

    <!-- 装饰性背景光晕 -->
    <div class="absolute top-1/4 -left-32 w-96 h-96 bg-purple-500/20 rounded-full blur-3xl animate-pulse"></div>
    <div class="absolute bottom-1/4 -right-32 w-96 h-96 bg-blue-500/20 rounded-full blur-3xl animate-pulse"
      style="animation-delay: 1s"></div>
    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-indigo-500/10 rounded-full blur-3xl"></div>

    <!-- 登录卡片 -->
    <div class="relative z-10 w-full max-w-md mx-4">
      <div class="card bg-base-100/80 backdrop-blur-xl shadow-2xl border border-base-300/50 hover:shadow-primary/10 hover:border-primary/20 transition-all duration-700">
        <div class="card-body items-center text-center gap-6 p-10">

          <!-- Logo -->
          <div class="mb-2">
            <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-purple-500/30 transform hover:scale-105 transition-transform duration-300">
              <svg class="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
            </div>
          </div>

          <h1 class="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
            SlackChat
          </h1>
          <p class="text-base-content/60 -mt-2 text-sm">团队协作，即时沟通</p>

          <div class="divider my-0"></div>

          <!-- 用户名输入 -->
          <div class="form-control w-full">
            <label class="label pb-1">
              <span class="label-text text-base-content/70 text-sm font-medium">输入昵称加入聊天</span>
            </label>
            <div class="relative">
              <span class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
              </span>
              <input
                v-model="username"
                class="input input-bordered w-full pl-10 focus:input-primary transition-all duration-300"
                placeholder="你的昵称..."
                maxlength="20"
                @keyup.enter="joinChat"
              />
            </div>
          </div>

          <!-- 加入按钮 -->
          <button
            class="btn btn-primary w-full gap-2 shadow-lg shadow-primary/30 hover:shadow-primary/50 transition-all duration-300 hover:scale-[1.02] active:scale-95"
            @click="joinChat"
            :disabled="!username.trim()"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
            开始聊天
          </button>

          <!-- 底部提示 -->
          <p class="text-xs text-base-content/40">
            按 Enter 快速加入 · 最多 20 个字符
          </p>
        </div>
      </div>
    </div>
  </div>

  <!-- 聊天界面 -->
  <div v-else class="h-screen flex overflow-hidden">
    <!-- 侧边栏 -->
    <Sidebar
      :channels="channelList"
      :activeChannel="currentChannel"
      :username="username"
      @select-channel="switchChannel"
      @refresh-channels="fetchChannels"
    />

    <!-- 主聊天区域 -->
    <div class="flex-1 flex flex-col bg-base-100 min-w-0">
      <!-- 顶部栏 -->
      <header class="flex items-center gap-3 px-5 py-3 border-b border-base-300 bg-base-100/80 backdrop-blur-sm shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-lg font-bold text-base-content/90">#</span>
          <span class="text-lg font-bold text-base-content">{{ currentChannel }}</span>
        </div>
        <div class="badge badge-ghost badge-sm gap-1 ml-2">
          <span class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></span>
          {{ messages.length }} 条消息
        </div>
        <div class="flex-1"></div>
        <div class="tooltip tooltip-bottom" data-tip="你的身份">
          <div class="badge badge-outline gap-1.5 px-3 py-1">
            <span class="w-2 h-2 rounded-full" :class="avatarColor(username)"></span>
            {{ username }}
          </div>
        </div>
      </header>

      <!-- 消息区域 -->
      <div ref="messageContainer" class="flex-1 overflow-y-auto scroll-smooth">
        <!-- 空状态 -->
        <div v-if="messages.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/30 gap-3">
          <svg class="w-16 h-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <p class="text-sm">暂无消息，发送第一条吧 ✨</p>
        </div>

        <!-- 消息列表 -->
        <div class="px-4 py-3 space-y-1">
          <template v-for="(msg, index) in messages" :key="msg.id || index">
            <!-- 日期分隔线 -->
            <div
              v-if="shouldShowDate(index, msg)"
              class="flex items-center gap-3 py-2"
            >
              <div class="flex-1 h-px bg-base-300"></div>
              <span class="text-xs text-base-content/40 font-medium whitespace-nowrap">{{ formatDate(msg.created_at) }}</span>
              <div class="flex-1 h-px bg-base-300"></div>
            </div>

            <!-- 系统消息 -->
            <div v-if="msg.username === '系统'" class="flex justify-center py-1.5">
              <span class="text-xs text-base-content/50 bg-base-200 px-3 py-1 rounded-full">{{ msg.content }}</span>
            </div>

            <!-- 普通消息气泡 -->
            <div v-else class="flex items-start gap-3 py-0.5 group hover:bg-base-200/50 rounded-lg px-2 -mx-2 transition-colors duration-150"
              :class="msg.username === username ? 'flex-row-reverse' : ''">
              <!-- 头像 -->
              <div class="avatar placeholder shrink-0 mt-1">
                <div class="w-9 h-9 rounded-full !rounded-btn text-sm font-bold"
                  :class="avatarColor(msg.username)">
                  <span>{{ msg.username.charAt(0).toUpperCase() }}</span>
                </div>
              </div>

              <!-- 消息内容 -->
              <div class="min-w-0" :class="msg.username === username ? 'items-end' : 'items-start'">
                <div class="flex items-center gap-2 mb-0.5"
                  :class="msg.username === username ? 'flex-row-reverse' : ''">
                  <span class="text-sm font-semibold text-base-content/80">{{ msg.username }}</span>
                  <span class="text-[11px] text-base-content/30 opacity-0 group-hover:opacity-100 transition-opacity">
                    {{ formatTime(msg.created_at) }}
                  </span>
                </div>
                <div class="chat-bubble text-sm leading-relaxed max-w-lg break-words"
                  :class="msg.username === username
                    ? 'chat-bubble-primary'
                    : ''">
                  {{ msg.content }}
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="p-4 bg-base-200/50 border-t border-base-300 shrink-0">
        <div class="flex gap-3 items-end max-w-4xl mx-auto">
          <!-- Emoji 按钮占位 -->
          <button class="btn btn-ghost btn-circle btn-sm shrink-0 opacity-50 hover:opacity-100 transition-opacity" title="表情">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </button>

          <!-- 输入框 -->
          <div class="flex-1 relative">
            <input
              v-model="newMessage"
              class="input input-bordered w-full pr-12 bg-base-100 focus:input-primary transition-all duration-200"
              placeholder="输入消息... (Enter 发送)"
              maxlength="500"
              @keyup.enter="sendMessage"
            />
            <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-base-content/30 pointer-events-none">
              {{ newMessage.length }}/500
            </span>
          </div>

          <!-- 发送按钮 -->
          <button
            class="btn btn-primary btn-circle shadow-lg shadow-primary/30 hover:shadow-primary/50 transition-all duration-200 hover:scale-105 active:scale-95"
            :disabled="!newMessage.trim()"
            @click="sendMessage"
            title="发送消息"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, watch } from 'vue'
import Sidebar from './components/Sidebar.vue'

const username = ref('')
const newMessage = ref('')
const isJoined = ref(false)
const messages = ref([])
const currentChannel = ref('general')
const channelList = ref([])
let socket = null          // WebSocket 连接实例
const messageContainer = ref(null)

// ============================================================
// 🔗 频道切换
// 关闭当前 WebSocket → 清空消息 → 更新频道 → 重新连接
// 对应后端: handler.rs → ws_handler() → handle_socket()
// ============================================================
const switchChannel = (newName) => {
  if (newName === currentChannel.value) return
  if (socket) socket.close()       // 断开旧的 WebSocket
  messages.value = []               // 清空消息列表
  currentChannel.value = newName    // 更新当前频道
  joinChat()                        // 重新建立 WebSocket 连接
}

// ============================================================
// 🔗 建立 WebSocket 连接（核心）
// 对应后端: GET /ws/{channel} → handler.rs → ws_handler() → handle_socket()
//
// 数据流:
//   发送: socket.send(JSON.stringify({channel, username, content}))
//         → 后端 handler.rs 线路 A: receiver.next() → 存 DB → tx.send(broadcast)
//   接收: 后端 handler.rs 线路 B: rx.recv() → sender.send(JSON)
//         → socket.onmessage → JSON.parse → messages.push → 渲染
// ============================================================
const joinChat = () => {
  if (!username.value.trim()) return
  isJoined.value = true

  // 根据当前协议自动选择 ws:// 或 wss://
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  socket = new WebSocket(`${protocol}//${window.location.host}/ws/${currentChannel.value}`)

  socket.onopen = () => {
    console.log(`已连接到 #${currentChannel.value}`)
  }

  // 接收消息：后端推送的 JSON → 解析 → 加入消息列表 → 自动滚动
  socket.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data)
      messages.value.push(message)
      scrollToBottom()
    } catch (e) {
      console.error('消息解析失败:', e)
    }
  }

  socket.onclose = () => {
    console.log(`已断开 #${currentChannel.value}`)
  }

  socket.onerror = (err) => {
    console.error('WebSocket 错误:', err)
  }
}

// ============================================================
// 🔗 发送消息
// 通过 WebSocket 将 JSON 数据发送到后端
// 对应后端: handler.rs → handle_socket() 中的线路 A (receiver.next())
// 消息格式: {"channel": "general", "username": "mikey", "content": "Hello"}
// ============================================================
const sendMessage = () => {
  if (!newMessage.value.trim() || !socket) return
  const messageData = {
    channel: currentChannel.value,
    username: username.value,
    content: newMessage.value.trim()
  }
  socket.send(JSON.stringify(messageData))
  newMessage.value = ''
}

// ============================================================
// 🔗 获取频道列表（REST API）
// 对应后端: handler.rs → get_channels() → GET /api/channels
// 在组件挂载时调用一次，之后通过 Sidebar 的 refresh-channels 事件重新拉取
// ============================================================
const fetchChannels = async () => {
  try {
    const response = await fetch('/api/channels')
    channelList.value = await response.json()
  } catch (err) {
    console.error('无法加载频道:', err)
  }
}

onMounted(fetchChannels)

// 格式化时间（短格式）
const formatTime = (isoString) => {
  if (!isoString) return ''
  const date = new Date(isoString)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })
}

// 格式化日期（用于分隔线）
const formatDate = (isoString) => {
  if (!isoString) return ''
  const date = new Date(isoString)
  const now = new Date()
  const diff = now - date
  const dayMs = 86400000

  if (diff < dayMs && date.getDate() === now.getDate()) return '今天'
  if (diff < dayMs * 2 && date.getDate() === now.getDate() - 1) return '昨天'

  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'short'
  })
}

// 是否显示日期分隔线
let lastDate = ''
const shouldShowDate = (index, msg) => {
  if (index === 0) {
    lastDate = msg.created_at ? new Date(msg.created_at).toDateString() : ''
    return true
  }
  const currentDate = msg.created_at ? new Date(msg.created_at).toDateString() : ''
  if (currentDate !== lastDate) {
    lastDate = currentDate
    return true
  }
  return false
}

// 头像颜色生成器
const avatarColor = (name) => {
  const colors = ['bg-primary', 'bg-secondary', 'bg-accent', 'bg-info', 'bg-success', 'bg-warning']
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  hash = Math.abs(hash)
  return colors[hash % colors.length]
}

// 自动滚动到底部（仅当用户在底部附近时）
const scrollToBottom = async () => {
  await nextTick()
  if (messageContainer.value) {
    const el = messageContainer.value
    const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 200
    if (isNearBottom || messages.value.length <= 2) {
      el.scrollTop = el.scrollHeight
    }
  }
}
</script>

<style scoped>
.chat-bubble {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}
</style>
