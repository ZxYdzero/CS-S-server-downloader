<script lang="ts">
import { UnlistenFn, listen } from '@tauri-apps/api/event';
import { ref } from "vue";
import { invoke } from '@tauri-apps/api/tauri';
import { defineComponent , onMounted, onUnmounted } from 'vue';

// 禁止右键和检查
//禁止F12
document.onkeydown = function (event: any) {
    var winEvent: any = window.event
    if (winEvent && winEvent.keyCode == 123) {
        event.keyCode = 0
        event.returnValue = false
    }
    if (winEvent && winEvent.keyCode == 13) {
        winEvent.keyCode = 505
    }
}
 
//屏蔽右键菜单
document.oncontextmenu = function (event: any) {
    if (window.event) {
        event = window.event
    }
    try {
        var the = event.srcElement
        if (
            !(
                (the.tagName == 'INPUT' && the.type.toLowerCase() == 'text') ||
                the.tagName == 'TEXTAREA'
            )
        ) {
            return false
        }
        return true
    } catch (e) {
        return false
    }
}

export default defineComponent({
  name: 'App',
  methods: {
    async sdownload() {
        // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
        invoke('sdownload', {}).then(() =>
          console.log('正在下载！!')
      )
    },

  },

  setup() {
    const message = ref<string>('');
    let unlisten: UnlistenFn | null = null;

    onMounted(async () => {
      try {
        // 监听后端发送的事件
        unlisten = await listen('message', (event) => {
          // 更新message状态
          message.value = event.payload as string;
        });
      } catch (error) {
        console.error('Error listening to rs2js event', error);
      }
    });
    

    onUnmounted(() => {
      // 清理: 在组件卸载时停止监听
      if (unlisten) {
        unlisten();
      }
    });

    return {
      message,
    };
  },
})



</script>

<template>
  <form class="row" @submit.prevent="sdownload">
    <button type="submit">下载</button>
  </form>

  <form class="row2">
    <p>{{ message }}</p>
  </form>

</template>
