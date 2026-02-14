<template>
  <el-dialog
    v-model="visible"
    title="手机扫码上传"
    width="400px"
    @close="handleClose"
    :close-on-click-modal="false"
    append-to-body
  >
    <div v-loading="loading" class="flex flex-col items-center justify-center py-4">
      <div v-if="qrCodeUrl" class="mb-6 relative">
        <img :src="qrCodeUrl" alt="Scan QR Code" class="w-56 h-56 border border-slate-200 rounded-xl p-2 bg-white shadow-sm" />
        <div class="absolute -bottom-2 -right-2 bg-green-500 text-white rounded-full p-1 shadow-lg">
           <span class="material-symbols-outlined text-sm block">check</span>
        </div>
      </div>
      <div v-else-if="!loading" class="text-slate-400 py-10 flex flex-col items-center">
         <span class="material-symbols-outlined text-4xl mb-2">qr_code_scanner</span>
         <span>生成二维码失败</span>
         <el-button type="primary" link @click="startServer" class="mt-2">重试</el-button>
      </div>
      
      <div class="text-center space-y-3">
        <h3 class="font-bold text-slate-700">请使用手机相机或微信扫码</h3>
        <p class="text-xs text-slate-400 bg-slate-50 py-2 px-4 rounded-full inline-block">
           💡 手机需与电脑连接同一 Wi-Fi
        </p>
        
        <div v-if="url" class="mt-6 pt-4 border-t border-slate-100 w-full px-8">
          <p class="text-xs text-slate-400 mb-1">或者在手机浏览器输入：</p>
          <a :href="url" target="_blank" class="text-blue-500 hover:underline font-mono bg-blue-50 px-2 py-1 rounded text-sm block truncate">
            {{ url }}
          </a>
        </div>
      </div>
    </div>
    <template #footer>
      <div class="flex justify-between items-center w-full">
        <div class="flex items-center text-xs text-slate-400">
           <span class="material-symbols-outlined text-sm mr-1">wifi_tethering</span>
           服务运行中
        </div>
        <el-button @click="visible = false">关闭</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

const props = defineProps({
  modelValue: Boolean
})

const emit = defineEmits(['update:modelValue'])

const visible = ref(false)
const loading = ref(false)
const qrCodeUrl = ref('')
const url = ref('')

watch(() => props.modelValue, (val) => {
  visible.value = val
  if (val) {
    if (!url.value) startServer()
  } else {
    // We don't necessarily stop server on close to allow background upload? 
    // Requirement says "Close dialog stops server" usually, or explicit stop. 
    // Let's stop it for security.
    stopServer()
  }
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

const startServer = async () => {
  loading.value = true
  try {
    const res = await invoke('start_mobile_server')
    url.value = res.url
    qrCodeUrl.value = res.qr_code
  } catch (e) {
    console.error('Failed to start mobile server:', e)
    ElMessage.error('无法启动手机上传服务: ' + e)
  } finally {
    loading.value = false
  }
}

const stopServer = async () => {
  qrCodeUrl.value = ''
  url.value = ''
  try {
    await invoke('stop_mobile_server')
  } catch (e) {
    console.error('Failed to stop server:', e)
  }
}

const handleClose = () => {
  stopServer()
}

onUnmounted(() => {
  stopServer()
})
</script>
