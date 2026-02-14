<template>
  <div class="flex flex-col h-full bg-slate-50 border border-slate-200 rounded-xl overflow-hidden shadow-sm">
    <!-- Top Toolbar -->
    <header class="h-16 flex items-center justify-between px-6 bg-white border-b border-slate-200 select-none">
      <div class="flex items-center gap-4">
        <h2 class="text-xl font-bold text-slate-800 tracking-tight">数据脱敏工作台</h2>
        <div class="h-6 w-px bg-slate-200"></div>
        <button 
          @click="handleOpenFile"
          class="flex items-center gap-2 px-4 py-2 bg-blue-50 text-blue-600 rounded-lg hover:bg-blue-100 transition-colors font-medium border border-blue-100"
        >
          <span class="material-symbols-outlined text-xl">folder_open</span>
          打开图片
        </button>
        <button 
          @click="handleSaveFile"
          :disabled="!hasImage"
          class="flex items-center gap-2 px-4 py-2 bg-emerald-50 text-emerald-600 rounded-lg hover:bg-emerald-100 transition-colors font-medium border border-emerald-100 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span class="material-symbols-outlined text-xl">save</span>
          另存为
        </button>
      <button 
          @click="showMobileDialog = true"
          class="flex items-center gap-2 px-4 py-2 bg-indigo-50 text-indigo-600 rounded-lg hover:bg-indigo-100 transition-colors font-medium border border-indigo-100 ml-2"
          title="手机扫码上传"
        >
          <span class="material-symbols-outlined text-xl">qr_code_scanner</span>
          手机上传
        </button>
      </div>

      
      <!-- Zoom Controls or Status -->
      <div v-if="hasImage" class="flex items-center gap-2 text-sm text-slate-500 bg-slate-50 px-3 py-1 rounded-full border border-slate-200">
        <span>{{ imageInfo }}</span>
      </div>
    </header>

    <!-- Main Workspace -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left Tool Palette -->
      <aside class="w-16 bg-white border-r border-slate-200 flex flex-col items-center py-6 gap-3 select-none z-10 shadow-sm">
        <div 
          v-for="tool in tools" 
          :key="tool.id"
          class="group relative"
        >
          <button 
            @click="currentTool = tool.id"
            :class="[
              'w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-200',
              currentTool === tool.id 
                ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/30' 
                : 'text-slate-500 hover:bg-slate-100 hover:text-slate-800'
            ]"
            :disabled="!hasImage"
          >
            <span class="material-symbols-outlined text-xl">{{ tool.icon }}</span>
          </button>
          <!-- Tooltip -->
          <div class="absolute left-full ml-2 top-1/2 -translate-y-1/2 px-2 py-1 bg-slate-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap z-50">
            {{ tool.name }}
          </div>
        </div>

        <div class="h-px w-8 bg-slate-200 my-2"></div>

        <!-- History Actions -->
        <button 
          @click="undo"
          :disabled="!canUndo"
          class="w-10 h-10 rounded-xl flex items-center justify-center text-slate-500 hover:bg-slate-100 hover:text-slate-800 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="撤销"
        >
          <span class="material-symbols-outlined text-xl">undo</span>
        </button>
        
        <button 
          @click="reset"
          :disabled="!hasImage"
          class="w-10 h-10 rounded-xl flex items-center justify-center text-orange-400 hover:bg-orange-50 hover:text-orange-500 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="重置编辑"
        >
          <span class="material-symbols-outlined text-xl">restore</span>
        </button>

        <div class="h-px w-8 bg-slate-200 my-2"></div>

        <!-- Zoom Controls -->
        <div class="flex flex-col items-center gap-2">
            <button 
                @click="handleZoomIn"
                :disabled="!hasImage"
                class="w-10 h-10 rounded-xl flex items-center justify-center text-slate-600 hover:bg-blue-50 hover:text-blue-600 transition-colors disabled:opacity-30 disabled:cursor-not-allowed bg-white border border-slate-100 shadow-sm"
                title="放大"
            >
                <span class="material-symbols-outlined text-xl">add_circle</span>
            </button>
            <button 
                @click="handleZoomOut"
                :disabled="!hasImage"
                class="w-10 h-10 rounded-xl flex items-center justify-center text-slate-600 hover:bg-blue-50 hover:text-blue-600 transition-colors disabled:opacity-30 disabled:cursor-not-allowed bg-white border border-slate-100 shadow-sm"
                title="缩小"
            >
                <span class="material-symbols-outlined text-xl">remove_circle</span>
            </button>
        </div>

        <div class="h-px w-8 bg-slate-200 my-2"></div>

        <button 
          @click="handleClear"
          :disabled="!hasImage"
          class="w-10 h-10 rounded-xl flex items-center justify-center text-red-500 hover:bg-red-50 hover:text-red-600 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="清空画布"
        >
          <span class="material-symbols-outlined text-xl">delete_forever</span>
        </button>
      </aside>

      <!-- Canvas Area -->
      <main class="flex-1 bg-slate-100 relative overscroll-none overflow-hidden block">
        <!-- Checkerboard Background Pattern -->
        <div class="absolute inset-0 opacity-5 pointer-events-none z-0" style="background-image: radial-gradient(#64748b 1px, transparent 1px); background-size: 20px 20px;"></div>

        <!-- Empty State -->
        <div v-if="!hasImage" class="absolute inset-0 flex flex-col items-center justify-center text-slate-400 gap-4 pointer-events-none select-none z-10">
          <div class="w-24 h-24 bg-slate-200 rounded-full flex items-center justify-center mb-2 animate-pulse">
            <span class="material-symbols-outlined text-5xl">image</span>
          </div>
          <p class="text-lg font-medium">请从左上角打开一张图片开始编辑</p>
          <p class="text-sm">支持 .jpg, .png, .bmp 格式</p>
        </div>

        <!-- Editor Component -->
        <div v-else class="absolute inset-0 z-10">
            <ImageEditor 
            ref="editorRef"
            :src="imageSrc"
            :tool="currentTool"
            @update:canUndo="canUndo = $event"
            @update:imageInfo="imageInfo = $event"
            />
        </div>
      </main>
    </div>
  </div>
    <MobileUploadDialog v-model="showMobileDialog" />
</template>


<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'

import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import MobileUploadDialog from '@/components/MobileUploadDialog.vue'



// Placeholder import - in real implementation this points to the new component
import ImageEditor from '@/components/ImageEditor.vue' 

const tools = [
  { id: 'move', name: '移动/查看', icon: 'pan_tool' },
  { id: 'crop', name: '裁剪', icon: 'crop' },
  { id: 'blur', name: '区域模糊', icon: 'blur_on' },
]

const currentTool = ref('move')
const imageSrc = ref(null)
const hasImage = computed(() => !!imageSrc.value)
const canUndo = ref(false)
const imageInfo = ref('')
const editorRef = ref(null)

const handleOpenFile = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp']
            }]
        })

        if (selected) {
            // Read file as binary to avoid asset protocol issues on some Windows setups
            const content = await readFile(selected);
            const blob = new Blob([content]);
            const url = URL.createObjectURL(blob);
            
            console.log('Opened image blob:', url);
            
            // Force component unmount/mount to ensure clean state
            if (imageSrc.value) {
                // Revoke old URL if it was a blob URL
                if (imageSrc.value.startsWith('blob:')) {
                    URL.revokeObjectURL(imageSrc.value);
                }
                imageSrc.value = null;
                await nextTick();
            }
            
            imageSrc.value = url;
            currentTool.value = 'move';
        }
    } catch (err) {
        console.error('Failed to open file:', err)
        ElMessage.error('打开文件失败: ' + err.message)
    }
}

const handleSaveFile = async () => {
    if (!editorRef.value) return;
    
    try {
        const dataUrl = await editorRef.value.exportImage();
        if (!dataUrl) {
            ElMessage.warning('无法导出图片');
            return;
        }

        // Convert base64 to binary
        // data:image/png;base64,...
        const base64Data = dataUrl.split(',')[1];
        const binaryData = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));

        const filePath = await save({
            filters: [{
                name: 'Image',
                extensions: ['png']
            }]
        });

        if (filePath) {
            await writeFile(filePath, binaryData);
            ElMessage.success('保存成功');
        }
    } catch (err) {
        console.error(err);
        ElMessage.error('保存失败: ' + err.message);
    }
}

const undo = () => {
  editorRef.value?.undo()
}

const reset = () => {
  editorRef.value?.reset()
}

const handleZoomIn = () => {
    editorRef.value?.zoomIn()
}

const handleZoomOut = () => {
    editorRef.value?.zoomOut()
}

const handleClear = async () => {
    if (!hasImage.value) return;
    
    try {
        await ElMessageBox.confirm(
            '确定要清空当前图片吗？未保存的修改将丢失。',
            '清空确认',
            {
                confirmButtonText: '清空',
                cancelButtonText: '取消',
                type: 'warning',
            }
        )
        imageSrc.value = null;
        currentTool.value = 'move';
        imageInfo.value = '';
    } catch (e) {
        // Cancelled
    }
}


const showMobileDialog = ref(false)
const route = useRoute()


onMounted(async () => {
    // 监听手机上传事件
    await listen('mobile_upload_success', async (event) => {
        if (route.path !== '/desensitize') return


        // 如果当前不在脱敏页面（虽然组件未卸载，但为了安全起见也可以加判断，
        // 不过由于是 keepAlive，组件一直挂载。我们通过 `showMobileDialog` 判断用户意图，
        // 或者简单地：如果收到了文件，且用户在当前页面（或者是通过本页面的弹窗发起的服务），就处理。
        // 为了简化，我们假设用户只要看这个页面，收到文件就处理。
        // 但要注意 upload 页面也监听了这个。如果不加区分，两个页面都会处理。
        // 不过 mobile upload dialog 是模态的，通常用户只在一个地方打开。
        // 但是服务是单例的。如果用户在 upload 页打开服务，然后切到脱敏页？
        // 最好判断当前 dialog 是否显示，或者简单地都处理（upload 页加到列表，desensitize 页替换画布）。
        // 如果 desensitize 页 dialog 没显示，突然替换画布会很怪。
        // 所以加一个判断：只有当 showMobileDialog 为 true 时，才自动处理。
        // 或者：总是处理，但是给用户提示。
        
        // 改进策略：仅当本页面的 mobile dialog 显示时，或者当用户主动在页面上时处理。
        // 为了体验，只要收到文件，就尝试加载。如果当前有图片，询问替换。
         
        const { filepath, filename } = event.payload
        console.log('Mobile upload success in Desensitize:', filename)

        // 如果弹窗没开，可能是从 Upload 页面传来的，或者后台传来的，这里我们暂时只处理当前页面发起的（用户意图明确）
        // 或者更宽泛一点：只要收到了，就提示用户是否要打开。
        // 对于本需求 "一次只能上传一个"，我们假设用户就是在等这个文件。
        
        try {
            // 提示用户
            if (hasImage.value) {
                 try {
                    await ElMessageBox.confirm(
                        `收到新文件 "${filename}"，是否替换当前画布内容？`,
                        '新文件到达',
                        { confirmButtonText: '替换', cancelButtonText: '忽略', type: 'info' }
                    )
                 } catch {
                     return // 用户取消
                 }
            } else if (!showMobileDialog.value) {
                // 如果没开弹窗也没图片，可能是在 Upload 页触发的。
                // 我们可以弹个 Notification 提示用户去查看，或者直接 Confirm。
                // 简单起见，如果弹窗开着，直接加载。如果没开，弹 Confirm。
                 try {
                    await ElMessageBox.confirm(
                        `收到新文件 "${filename}"，是否加载到画布？`,
                        '新文件到达',
                        { confirmButtonText: '加载', cancelButtonText: '忽略', type: 'info' }
                    )
                 } catch {
                     return 
                 }
            }

            // 读取文件
            const base64Data = await invoke('read_temp_file', { path: filepath })
            
            // 清理旧资源
             if (imageSrc.value && imageSrc.value.startsWith('blob:')) {
                URL.revokeObjectURL(imageSrc.value);
            }
            imageSrc.value = null;
            await nextTick();
            
            imageSrc.value = base64Data; // data:image/jjj;base64,...
            currentTool.value = 'move';
            
            ElNotification({
                title: '图片已加载',
                message: filename,
                type: 'success',
            })
            
            // 如果弹窗开着，可以考虑关闭它，或者保持开启允许继续传？
            // 需求说 "一次只能上传一个"，可能是指 user mental model。
            // 我们可以自动关闭弹窗
            showMobileDialog.value = false;

        } catch (e) {
            console.error('Failed to handle mobile upload:', e)
        }
    })
})

</script>

<style scoped>
/* Custom Scrollbar for Tools if needed */
aside::-webkit-scrollbar {
  width: 0px;
}
</style>
