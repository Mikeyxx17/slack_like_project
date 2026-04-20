<template>
  <div class="app-container">
    <div v-if="!isJoined" class="join-modal">
      <h2>欢迎来到类 Slack 聊天室</h2>
      <input 
        v-model="username" 
        placeholder="请输入你的昵称..." 
        @keyup.enter="joinChat"
      />
      <button @click="joinChat">开始聊天</button>
    </div>

    <div v-else class="chat-room">
      <header>当前频道：#general | 用户：{{ username }}</header>
      
      <div v-for="msg in messages" :key="msg.id" class="message-box">
        <strong class="user-name"> {{ msg.username }}: </strong>
        <span class="message-text">{{ msg.content }}</span>
                <span class="message-time" style="color: #888; font-size: 0.8em;">
          [{{ formatTime(msg.created_at) }}]
        </span>
      </div>
      <div class="input-area">
        <input 
          v-model="newMessage" 
          placeholder="想说点什么..." 
          @keyup.enter="sendMessage"
        />
        <button @click="sendMessage">发送</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const username = ref('')
const newMessage = ref('')
const isJoined = ref(false)
const messages = ref([]) // 存储所有收到的消息

const formatTime = (isoString) => {
  if (!isoString) return '刚刚'
  const date = new Date(isoString)
  
  // 使用 zh-CN (中国) 习惯进行格式化
  return date.toLocaleString('zh-CN', {
    year: 'numeric',  // 显示年份
    month: '2-digit', // 两位月份
    day: '2-digit',   // 两位日期
    hour: '2-digit',  // 两位小时
    minute: '2-digit',// 两位分钟
    hour12: false     // 使用 24 小时制
  })
}


const sendMessage = () => {
  // 1. 如果输入框是空的，或者 socket 还没连上，就不发
  if (!newMessage.value.trim() || !socket) return

  // 2. 按照后端的“模具”打包数据
  const messageData = {
    channel: "general",        // 目前先写死为 general
    username: username.value,  // 使用用户刚进入时填写的名字
    content: newMessage.value  // 使用输入框里的内容
  }

  // 3. 通过网线发出去！(记得转成字符串)
  socket.send(JSON.stringify(messageData))

  // 4. 发完之后，清空输入框，方便下次输入
  newMessage.value = ''
}


let socket = null

const joinChat = () => {
  if (username.value.trim()) {
    isJoined.value = true
    
    // socket = new WebSocket('ws://localhost:5173/ws') // 本地开发把这行打开
    socket = new WebSocket('ws://119.29.224.113:3000/ws') // 部署后把这行打开
    socket.onopen = () => {
      console.log('✅ 成功连接到 Rust 后端服务器！')
    }

    // ✅ 核心逻辑：收到消息只负责“存”，显示交给 template
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data)
      // 把新消息塞进数组，Vue 的响应式系统会自动触发 template 里的 v-for 更新
      messages.value.push(message)
      console.log('原始时间数据:', message.created_at)
    }

    socket.onerror = (err) => {
      console.error('❌ 连接错误：', err)
    }
  }
}
</script>