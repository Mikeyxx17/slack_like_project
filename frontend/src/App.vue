<template>
  <LoginView v-if="!isJoined" />
  <ChatLayout v-else />
</template>

<script setup>
import { useAppState } from './composables/useAppState'
import { useChannels } from './composables/useChannels'
import LoginView from './components/LoginView.vue'
import ChatLayout from './components/ChatLayout.vue'

const { isJoined } = useAppState()

// 应用启动时预加载频道列表和主题
const { fetchChannels } = useChannels()
fetchChannels()
</script>
