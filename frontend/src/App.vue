<script setup>
import { ref, onMounted } from 'vue'

// --- 1. 响应式数据定义 ---
const isConnected = ref(false)
const messages = ref([])
// 初始名字随机生成，但用户可以在界面上修改
const username = ref('用户' + Math.floor(Math.random() * 100))
const content = ref('')
const channel = ref('general')

let socket = null

// --- 2. 工具函数：格式化后端传来的时间戳 ---
const formatDate = (dateStr) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  // 只显示 小时:分钟
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

// --- 3. WebSocket 连接逻辑 ---
onMounted(() => {
  socket = new WebSocket("ws://127.0.0.1:3000/ws")

  socket.onopen = () => { isConnected.value = true }
  
  socket.onmessage = (event) => {
    // 接收后端发来的 JSON 字符串并解析
    const data = JSON.parse(event.data)
    messages.value.push(data)
    
    // 自动滚动到聊天框底部
    setTimeout(() => {
      const el = document.getElementById('chat-box')
      if (el) el.scrollTop = el.scrollHeight
    }, 50)
  }
  
  socket.onclose = () => { isConnected.value = false }
})

// --- 4. 发送消息逻辑 ---
const sendMessage = () => {
  if (!content.value || !isConnected.value) return

  // 这里的字段名（channel, username, content）必须和 Rust 的 ChatMessage 结构体完全一致
  const msg = {
    channel: channel.value,
    username: username.value,
    content: content.value
  }

  // 将对象转为 JSON 字符串发给 Rust 后端
  socket.send(JSON.stringify(msg))
  content.value = '' // 发送后清空输入框
}
</script>

<template>
  <div style="padding: 20px; max-width: 600px; margin: 0 auto; font-family: sans-serif;">
    
    <h2 style="color: #4a154b;">{{ username }} 的聊天室</h2>
    
    <div style="margin-bottom: 15px; display: flex; align-items: center; gap: 10px;">
      <span :style="{ color: isConnected ? 'green' : 'red' }">
        ● {{ isConnected ? '在线' : '离线' }}
      </span>
      <div style="flex: 1"></div>
      <label>我的昵称:</label>
      <input v-model="username" style="padding: 4px; width: 100px;" />
    </div>

    <div id="chat-box" style="border: 1px solid #ddd; border-radius: 8px; padding: 15px; height: 400px; overflow-y: auto; background: #fefefe;">
      
      <div v-for="(msg, index) in messages" :key="index" style="margin-bottom: 15px;">
        
        <div v-if="msg.username === '系统'" style="text-align: center; color: #888; font-style: italic; font-size: 0.9em;">
           📢 {{ msg.content }}
        </div>
        
        <div v-else style="display: flex; flex-direction: column;">
          <div style="display: flex; align-items: baseline; gap: 10px;">
            <span style="font-weight: bold; color: #4a154b;">{{ msg.username }}</span>
            <span style="font-size: 0.7em; color: #aaa;">{{ formatDate(msg.created_at) }}</span>
          </div>
          <div style="margin-top: 4px; color: #333;">{{ msg.content }}</div>
        </div>

      </div>
    </div>

    <div style="margin-top: 15px; display: flex; gap: 10px;">
      <input 
        v-model="content" 
        @keyup.enter="sendMessage" 
        placeholder="输入消息..." 
        style="flex: 1; padding: 10px;" 
      />
      <button 
        @click="sendMessage" 
        :disabled="!isConnected"
        style="padding: 10px 20px; background: #4a154b; color: white; border: none; cursor: pointer;"
      >
        发送
      </button>
    </div>

  </div>
</template>