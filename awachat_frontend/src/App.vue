<script setup lang="ts">
import ChatInputArea from './components/ChatInputArea.vue';
import ChatShowArea from './components/ChatShowArea.vue';
</script>

<template>
  <h1 class="big-title">awaChat🥰</h1>
  <div class="chat-area border-md">
    <ChatShowArea />
    <ChatInputArea />
  </div>
</template>

<style lang="css" scoped>
.big-title {
  font-size: 3rem;
}

.chat-area {
  width: 600px;
}
</style>
