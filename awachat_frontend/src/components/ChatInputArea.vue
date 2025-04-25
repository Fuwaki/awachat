<script setup lang="ts">
import { SocketService } from '@/services/socketService';
import { ref } from 'vue';

const inputText = ref("")

function send() {
  SocketService.sendMessage(inputText.value)
}
</script>

<template>
  <div class="chat-input-area border-t-sm">
    <v-responsive>
      <v-text-field hide-details="auto" placeholder="输入内容..." v-model="inputText"></v-text-field>
    </v-responsive>
    <v-btn variant="flat" @click="send">发送！</v-btn>
  </div>
</template>

<style lang="css" scoped>
.chat-input-area {
  display: flex;
  height: min-content;
  align-items: center;
}
</style>
