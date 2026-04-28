<template>
  <aside class="sidebar">
    <div class="sidebar-header">频道列表</div>
    <div 
      v-for="ch in channels" 
      :key="ch.id" 
      class="channel-item"
      :class="{ active: activeChannel === ch.name }"
      @click="$emit('select-channel', ch.name)"
    >
      # {{ ch.name }}
    </div>
  </aside>
  <div class="flex items-center justify-between mb-4 px-2">
  <span class="text-sm font-semibold opacity-70 uppercase tracking-wider">
    频道列表
  </span>
  <button @click="showModal = true" class="btn btn-xs btn-circle btn-ghost">＋</button>
</div>
</template>

<script setup>
defineProps(['channels', 'activeChannel'])
defineEmits(['select-channel'])
</script>

<style scoped>
.sidebar { width: 200px; background: #3f0e40; color: white; padding: 10px; }
.channel-item { padding: 8px; cursor: pointer; border-radius: 4px; }
.channel-item:hover { background: #522653; }
.channel-item.active { background: #1164a3; }
</style>