<template>
  <div class="app-container">
    <div v-if="!isJoined" class="join-modal">
      <h2>欢迎来到类 Slack 聊天室</h2>
      <input v-model="username" placeholder="请输入你的昵称..." @keyup.enter="joinChat" />
      <button @click="joinChat">开始聊天</button>
    </div>

    <div v-else class="chat-room">
      <aside class="sidebar">
        <div class="sidebar-header">频道列表</div>
        <div 
          v-for="ch in channelList" 
          :key="ch.id" 
          class="channel-item"
          :class="{ active: currentChannel === ch.name }"
          @click="switchChannel(ch.name)"
        >
          # {{ ch.name }}
        </div>
      </aside>

      <main class="chat-main">
        <header>当前频道：#{{ currentChannel }} | 用户：{{ username }}</header>
        
        <div class="message-list">
          <div v-for="msg in messages" :key="msg.id" class="message-box">
            <strong class="user-name"> {{ msg.username }}: </strong>
            <span class="message-text">{{ msg.content }}</span>
            <span class="message-time">[{{ formatTime(msg.created_at) }}]</span>
          </div>
        </div>

        <div class="input-area">
          <input v-model="newMessage" placeholder="想说点什么..." @keyup.enter="sendMessage" />
          <button @click="sendMessage">发送</button>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const username = ref('')
const newMessage = ref('')
const isJoined = ref(false)
const messages = ref([])
const currentChannel = ref('general') // 默认进入 general
const channelList = ref([])
let socket = null

// 🔄 切换频道的“四步走”逻辑
const switchChannel = (newName) => {
  if (newName === currentChannel.value) return // 拦截重复点击
  if (socket) socket.close()                 // 1. 断开旧连接
  messages.value = []                        // 2. 清空屏幕
  currentChannel.value = newName             // 3. 更新当前频道名
  joinChat()                                 // 4. 重新连上新频道
}

const joinChat = () => {
  if (!username.value.trim()) return
  isJoined.value = true
  // 🔗 动态拨号：URL 后面带着当前选中的频道名
  socket = new WebSocket(`ws://119.29.224.113:3000/ws/${currentChannel.value}`)
  
  socket.onmessage = (event) => {
    const message = JSON.parse(event.data)
    messages.value.push(message)
  }
}

// 📦 发送消息逻辑
const sendMessage = () => {
  if (!newMessage.value.trim() || !socket) return
  const messageData = {
    channel: currentChannel.value, // 打上当前频道的标签
    username: username.value,
    content: newMessage.value
  }
  socket.send(JSON.stringify(messageData))
  newMessage.value = ''
}

// 🚚 初始化：一进网页就去后端取频道清单
const fetchChannels = async () => {
  try {
    const response = await fetch('http://119.29.224.113:3000/api/channels')
    channelList.value = await response.json()
  } catch (err) { console.error('❌ 无法加载频道:', err) }
}

onMounted(fetchChannels)

// 🕒 时间格式化函数：将后端发来的 ISO 时间字符串转成中文易读格式
const formatTime = (isoString) => {
  if (!isoString) return '刚刚'
  const date = new Date(isoString)
  
  // 使用 zh-CN习惯进行格式化
  return date.toLocaleString('zh-CN', {
    year: 'numeric',   // 2026年
    month: '2-digit',  // 04月
    day: '2-digit',    // 21日
    hour: '2-digit',   // 19时
    minute: '2-digit', // 18分
    hour12: false      // 使用 24 小时制
  })
}

</script>

<style>
/* 核心布局 CSS */
.chat-room {
  display: flex; /* 开启左右并排模式 ↔️ */
  height: 90vh;
}
.sidebar {
  width: 200px;
  background: #3f0e40; /* Slack 经典紫 🟣 */
  color: white;
  padding: 10px;
}
.channel-item {
  padding: 8px;
  cursor: pointer;
  border-radius: 4px;
}
.channel-item:hover { background: #522653; }
.channel-item.active { background: #1164a3; } /* 你刚才选中的蓝色亮起！🟦 */
.chat-main { flex: 1; display: flex; flex-direction: column; }
.message-list { flex: 1; overflow-y: auto; padding: 20px; }
/* 其他样式省略... */
</style>