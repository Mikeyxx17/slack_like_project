<!--
  Sidebar.vue — 频道侧边栏组件

  与后端交互点：
    1. 创建频道 → fetch('/api/channels', {method: 'POST', body: {name}})
       对应后端 handler.rs → create_channel() → POST /api/channels
    2. 刷新频道列表 → 通过 emit('refresh-channels') 通知父组件 App.vue
       调用 fetch('/api/channels')，对应后端 handler.rs → get_channels() → GET /api/channels
    3. 切换频道 → emit('select-channel', ch.name)
       通知父组件关闭旧 WebSocket，建立新 WebSocket 连接到 /ws/{新频道}
-->
<template>
  <aside class="w-[260px] h-full flex flex-col bg-base-300/60 border-r border-base-300 shrink-0 select-none">
    
    <!-- 品牌区域 -->
    <div class="px-4 py-4 border-b border-base-300/50">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-md">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        </div>
        <div>
          <h1 class="text-base font-bold text-base-content">SlackChat</h1>
          <p class="text-[10px] text-base-content/40 -mt-0.5">团队即时通讯</p>
        </div>
      </div>
    </div>

    <!-- 频道标题行 -->
    <div class="flex items-center justify-between px-4 pt-4 pb-2">
      <span class="text-[11px] font-semibold uppercase tracking-widest text-base-content/50">频道</span>
      <button
        class="btn btn-ghost btn-xs btn-square hover:bg-primary/20 hover:text-primary transition-colors"
        @click="showModal = true"
        title="创建频道"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>

    <!-- 频道列表 -->
    <div class="flex-1 overflow-y-auto px-3 pb-2">
      <div
        v-for="ch in channels"
        :key="ch.id"
        class="flex items-center gap-2.5 px-3 py-2 rounded-lg cursor-pointer transition-all duration-150 mb-0.5 group"
        :class="activeChannel === ch.name
          ? 'bg-primary/15 text-primary font-semibold shadow-sm'
          : 'text-base-content/70 hover:bg-base-200/80 hover:text-base-content'"
        @click="$emit('select-channel', ch.name)"
      >
        <span class="text-lg font-medium shrink-0"
          :class="activeChannel === ch.name ? '' : 'text-base-content/30 group-hover:text-base-content/60'">#</span>
        <span class="truncate text-sm">{{ ch.name }}</span>
        <span 
          v-if="activeChannel === ch.name"
          class="ml-auto w-1.5 h-1.5 rounded-full bg-primary animate-pulse shrink-0"
        ></span>
      </div>

      <!-- 空状态 -->
      <div v-if="channels.length === 0" class="text-center py-8 text-base-content/30">
        <svg class="w-8 h-8 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M5.586 5.586a2 2 0 012.828 0L12 9.172l3.586-3.586a2 2 0 112.828 2.828L14.828 12l3.586 3.586a2 2 0 11-2.828 2.828L12 14.828l-3.586 3.586a2 2 0 11-2.828-2.828L9.172 12 5.586 8.414a2 2 0 010-2.828z" />
        </svg>
        <p class="text-xs">暂无频道</p>
      </div>
    </div>

    <!-- 用户信息栏 -->
    <div class="px-4 py-3 border-t border-base-300/50 bg-base-200/50">
      <div class="flex items-center gap-3">
        <!-- 头像 -->
        <div class="avatar placeholder shrink-0">
          <div class="w-8 h-8 rounded-full !rounded-btn text-xs font-bold bg-primary text-primary-content">
            {{ username ? username.charAt(0).toUpperCase() : '?' }}
          </div>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium text-base-content truncate">{{ username || '未登录' }}</p>
          <p class="text-[10px] text-success/80 flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-success"></span>
            在线
          </p>
        </div>
      </div>
    </div>

    <!-- 创建频道弹窗 -->
    <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      @click.self="showModal = false">
      <div class="card bg-base-100 shadow-2xl w-96 animate-in">
        <div class="card-body gap-4 p-6">
          <h3 class="card-title text-lg flex items-center gap-2">
            <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            新建频道
          </h3>
          <p class="text-sm text-base-content/60">创建一个新的聊天频道，邀请团队成员加入讨论。</p>

          <div class="form-control">
            <label class="label pb-1">
              <span class="label-text text-xs font-medium">频道名称</span>
            </label>
            <div class="relative">
              <span class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-base-content/40 font-bold">#</span>
              <input
                v-model="newChannelName"
                type="text"
                class="input input-bordered w-full pl-9 focus:input-primary"
                placeholder="例如：rust-学习群"
                maxlength="30"
                @keyup.enter="createChannel"
                ref="modalInput"
              />
            </div>
          </div>

          <div class="card-actions justify-end mt-2">
            <button class="btn btn-ghost btn-sm" @click="showModal = false">取消</button>
            <button
              class="btn btn-primary btn-sm gap-2"
              @click="createChannel"
              :disabled="!newChannelName.trim()"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              创建
            </button>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { ref, nextTick } from 'vue'

const props = defineProps({
  channels: Array,
  activeChannel: String,
  username: String
})

const emit = defineEmits(['select-channel', 'refresh-channels'])

const showModal = ref(false)
const newChannelName = ref('')
const modalInput = ref(null)

// 打开弹窗时自动聚焦输入框
const toggleModal = (val) => {
  showModal.value = val
  if (val) {
    nextTick(() => {
      modalInput.value?.focus()
    })
  }
}

// ============================================================
// 🔗 与后端交互: POST /api/channels 创建新频道
// 对应后端: handler.rs → create_channel()
// 请求: fetch('/api/channels', {method: 'POST', body: JSON.stringify({name})})
// 响应: 201 (创建成功) 或 500 (失败)
// ============================================================
const createChannel = async () => {
  const name = newChannelName.value.trim()
  if (!name) return

  try {
    const response = await fetch('/api/channels', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name })
    })

    if (response.ok) {
      showModal.value = false
      newChannelName.value = ''
      // 通知父组件重新拉取频道列表
      emit('refresh-channels')
    } else {
      alert('创建失败，频道可能已存在')
    }
  } catch (err) {
    console.error('创建频道失败:', err)
    alert('网络错误，请稍后重试')
  }
}
</script>

<style scoped>
/* 自定义滚动条 */
aside ::-webkit-scrollbar {
  width: 4px;
}
aside ::-webkit-scrollbar-track {
  background: transparent;
}
aside ::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}
aside ::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

/* 弹窗动画 */
.animate-in {
  animation: modalIn 0.2s ease-out;
}
@keyframes modalIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
