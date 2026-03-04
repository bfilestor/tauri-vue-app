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
      
      <div class="text-center space-y-3 w-full px-4">
        <h3 class="font-bold text-slate-700">请使用手机相机或微信扫码</h3>
        
        <div class="bg-slate-50 py-3 px-4 rounded-lg flex flex-col items-center gap-2">
            <span class="text-xs text-slate-500">💡 手机需与电脑连接同一网络</span>
            <el-select v-model="selectedIp" size="small" @change="onIpChange" class="w-48" placeholder="选择网络 IP" :disabled="loading">
                <el-option v-for="ip in availableIps" :key="ip" :label="ip" :value="ip" />
            </el-select>
        </div>
        
        <div v-if="url" class="mt-4 pt-4 border-t border-slate-100 w-full px-4">
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

const availableIps = ref([])
const selectedIp = ref('')

watch(() => props.modelValue, async (val) => {
  visible.value = val
  if (val) {
    if (availableIps.value.length === 0) {
      await initIps();
    }
    if (!url.value) startServer()
  } else {
    stopServer()
  }
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

const initIps = async () => {
    try {
        const ips = await invoke('get_local_ips');
        availableIps.value = ips;
        const configIp = await invoke('get_config', { key: 'mobile_upload_ip' });
        if (configIp && ips.includes(configIp)) {
            selectedIp.value = configIp;
        } else if (ips.length > 0) {
            selectedIp.value = ips[0];
        }
    } catch (e) {
        console.error('Failed to init IPs:', e);
    }
}

const onIpChange = async (newIp) => {
    try {
        await invoke('save_config', { key: 'mobile_upload_ip', value: newIp });
        await startServer();
    } catch (e) {
        console.error('Failed to update IP config:', e);
    }
}

const startServer = async () => {
  loading.value = true
  // Stop existing server first if any
  await stopServer();
  
  try {
    const res = await invoke('start_mobile_server', { selectedIp: selectedIp.value })
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
