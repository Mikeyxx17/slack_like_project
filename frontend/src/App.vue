 <template>
  <div class="w-screen h-screen bg-base-200">
    
    <div v-if="!isJoined" class="flex w-full h-full items-center justify-center">
      <div class="card bg-base-100 shadow-xl p-10 flex flex-col gap-4 text-center">
        <h2 class="text-2xl font-bold">欢迎来到类 Slack 聊天室</h2>
        <input 
          v-model="username" 
          class="input input-bordered w-full" 
          placeholder="请输入你的昵称..." 
          @keyup.enter="joinChat" 
        />
        <button class="btn btn-primary shadow-lg hover:scale-105" @click="joinChat">
          开始聊天
        </button>
      </div>
    </div>

    <div v-else class="flex w-full h-full">
      <Sidebar 
        :channels="channelList" 
        :activeChannel="currentChannel" 
        @select-channel="switchChannel" 
      />

      <main class="flex-1 flex flex-col bg-base-100">
        <header class="p-4 border-b border-base-300 font-bold shadow-sm">
          当前频道：#{{ currentChannel }} | 用户：{{ username }}
        </header>
        
        <div ref="messageContainer" class="flex-1 overflow-y-auto p-4 space-y-4">
          <div 
            v-for="msg in messages" 
            :key="msg.id" 
            class="chat"
            :class="msg.username === username ? 'chat-end' : 'chat-start'"
          >
            <div class="chat-header mb-1">
              {{ msg.username }}
              <time class="text-xs opacity-50 ml-1">{{ formatTime(msg.created_at) }}</time>
            </div>

            <div 
              class="chat-bubble shadow-md" 
              :class="msg.username === username ? 'chat-bubble-primary' : ''"
            >
              {{ msg.content }}
            </div>

            <div class="chat-footer opacity-50 text-xs mt-1">
              已送达
            </div>
          </div>
        </div>

        <div class="p-4 bg-base-200 flex gap-2">
          <input 
            v-model="newMessage" 
            class="input input-bordered flex-1" 
            placeholder="想说点什么..." 
            @keyup.enter="sendMessage" 
          />
          <button class="btn btn-primary" @click="sendMessage">发送</button>
        </div>
      </main>
    </div>
    
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick} from 'vue'
import Sidebar from './components/Sidebar.vue'
const username = ref('')
const newMessage = ref('')
const isJoined = ref(false)
const messages = ref([])
const currentChannel = ref('general') // 默认进入 general
const channelList = ref([])
let socket = null
const messageContainer = ref(null)

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
    scrollToBottom()
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

const scrollToBottom = async () => {
  await nextTick() 
  if (messageContainer.value) {
    messageContainer.value.scrollTop = messageContainer.value.scrollHeight
  }
}

</script>

