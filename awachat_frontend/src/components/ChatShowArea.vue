<script setup lang="ts">
import { SocketService } from '@/services/socketService';
import { ref } from 'vue';

const messageList = ref<string[]>([])

SocketService.addMessageListener(handleSocketMessage)

function handleSocketMessage(e: MessageEvent<string>) {
  messageList.value.push(e.data)
  console.log(e.data)
}
</script>

<template>
  <div class="chat-area">
    <span v-for="item, index in messageList" :key="index">{{ item }}</span>
  </div>
</template>

<style lang="css" scoped>
.chat-area {
  display: flex;
  flex-direction: column;
}
</style>
