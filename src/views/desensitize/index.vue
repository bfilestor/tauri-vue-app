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
</template>

<script setup>
import { ref, computed, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile } from '@tauri-apps/plugin-fs'

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
</script>

<style scoped>
/* Custom Scrollbar for Tools if needed */
aside::-webkit-scrollbar {
  width: 0px;
}
</style>
