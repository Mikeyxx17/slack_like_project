<template>
  <aside class="sidebar h-full flex flex-col">
    <div class="sidebar-header flex justify-between items-center mb-4 px-2">
      <span class="text-sm font-semibold opacity-70 uppercase tracking-wider">
        频道列表
      </span>
      <button @click="showModal = true" class="btn btn-xs btn-circle btn-ghost text-white">＋</button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div 
        v-for="ch in channels" 
        :key="ch.id" 
        class="channel-item"
        :class="{ active: activeChannel === ch.name }"
        @click="$emit('select-channel', ch.name)"
      >
        # {{ ch.name }}
      </div>
    </div>

    <div :class="{ 'modal-open': showModal }" class="modal text-base-content">
      <div class="modal-box bg-base-100">
        <h3 class="font-bold text-lg text-black">新建频道</h3>
        <p class="py-4 text-gray-600">请输入新频道的名称：</p>
        
        <input 
          v-model="newChannelName" 
          type="text" 
          class="input input-bordered w-full" 
          placeholder="例如：rust-学习群" 
          @keyup.enter="createChannel"
        />

        <div class="modal-action">
          <button @click="showModal = false" class="btn">取消</button>
          <button @click="createChannel" class="btn btn-primary">创建</button>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { ref } from 'vue'

// 接收父组件的数据
defineProps(['channels', 'activeChannel'])

// 定义发射给父组件的信号
const emit = defineEmits(['select-channel', 'refresh-channels'])

// 💡 响应式状态
const showModal = ref(false)
const newChannelName = ref('')

// 🚀 执行创建逻辑
const createChannel = async () => {
  if (!newChannelName.value.trim()) {
    alert('请输入频道名称')
    return
  }

  try {
    const response = await fetch('http://119.29.224.113:3000/api/channels', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: newChannelName.value.trim() })
    })

    if (response.ok) {
      showModal.value = false      // 关闭弹窗
      newChannelName.value = ''    // 清空输入框
      emit('refresh-channels')     // 📢 重点：通知父组件刷新列表
    }
  } catch (err) {
    console.error('创建频道失败:', err)
  }
}
</script>

<style scoped>
.sidebar { width: 260px; background: #3f0e40; color: white; padding: 15px; }
.sidebar-header { border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 10px; }
.channel-item { padding: 10px 12px; cursor: pointer; border-radius: 6px; margin-bottom: 2px; }
.channel-item:hover { background: #522653; }
.channel-item.active { background: #1164a3; font-weight: bold; }
</style>