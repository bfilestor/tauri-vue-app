<template>
  <div class="flex gap-8 items-start relative min-h-[calc(100vh-4rem)]">
    <!-- 主内容区 -->
    <div class="flex-1 w-full min-w-0">
      <header class="flex justify-between items-center mb-8">
        <div>
          <h1 class="text-3xl font-bold text-slate-900">检查数据上传与存档</h1>
          <p class="text-slate-500 mt-2">支持多文件上传，通过 AI 进行深度解析与存档管理。</p>
        </div>
        <div class="flex items-center gap-4">
          <!-- 这里可以放日期显示或其他状态 -->
          <el-tag size="large" effect="plain" class="!text-sm !font-bold">{{ todayDate }}</el-tag>
        </div>
      </header>

      <!-- 项目选择与上传区域 -->
      <div class="flex flex-col md:flex-row gap-6 mb-8">
        <!-- 左侧：项目选择 -->
        <div class="w-full md:w-64 shrink-0">
          <label class="block text-sm font-bold text-slate-700 mb-3">选择检查项目</label>
          <div class="space-y-2 max-h-[400px] overflow-y-auto pr-2">
            <template v-for="p in projects" :key="p.id">
              <button
                @click="currentProjectId = p.id"
                class="w-full flex items-center justify-between px-4 py-3 rounded-xl transition-all border text-left"
                :class="currentProjectId === p.id 
                  ? 'bg-blue-500 text-white border-blue-600 shadow-md shadow-blue-500/20' 
                  : 'bg-white border-slate-200 text-slate-600 hover:border-blue-400 hover:text-blue-500'"
              >
                <div class="flex items-center gap-3">
                  <span class="material-symbols-outlined text-xl">{{ getProjectIcon(p.name) }}</span>
                  <span class="font-medium">{{ p.name }}</span>
                </div>
                <span v-if="getProjectFileCount(p.id) > 0" 
                  class="text-xs px-2 py-0.5 rounded-full"
                  :class="currentProjectId === p.id ? 'bg-white/20 text-white' : 'bg-slate-100 text-slate-500'">
                  {{ getProjectFileCount(p.id) }}
                </span>
              </button>
            </template>
            <!-- 占位符项目（如果为空） -->
            <div v-if="projects.length === 0" class="text-center text-slate-400 py-4 text-sm">
              暂无项目，请去设置页添加
            </div>
          </div>
        </div>

        <!-- 右侧：上传区域 -->
        <div class="flex-1 min-w-0 flex flex-col">
          <!-- 拖拽上传框 -->
          <div
            v-if="!record || !currentProjectId"
            class="bg-slate-50 rounded-2xl border-2 border-dashed border-slate-300 p-10 text-center flex flex-col items-center justify-center h-64 text-slate-400"
          >
            <p v-if="!record">正在初始化记录...</p>
            <p v-else>请选择左侧项目以上传图片</p>
          </div>
          
          <div v-else
            class="bg-white rounded-2xl border-2 border-dashed border-blue-400/30 p-8 text-center group hover:bg-blue-50/10 transition-all flex-1 flex flex-col"
            @dragover.prevent
            @drop.prevent="handleDrop"
          >
            <!-- 待上传列表（预览） -->
            <div v-if="pendingFiles.length > 0" class="flex-1 mb-6">
               <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-4">
                  <div v-for="(file, idx) in pendingFiles" :key="idx" class="relative group/preview aspect-square rounded-lg overflow-hidden border border-slate-200">
                    <img :src="file.preview" class="w-full h-full object-cover" />
                    <button @click="removePending(idx)" class="absolute top-1 right-1 bg-red-500 text-white rounded-full p-1 opacity-0 group-hover/preview:opacity-100 transition-opacity">
                      <span class="material-symbols-outlined text-xs block">close</span>
                    </button>
                  </div>
                  <!-- 继续添加按钮 -->
                  <div class="aspect-square rounded-lg border-2 border-dashed border-slate-200 flex items-center justify-center cursor-pointer hover:border-blue-400 hover:text-blue-500 transition-colors"
                    @click="triggerFileInput">
                     <span class="material-symbols-outlined text-3xl text-slate-300">add</span>
                  </div>
               </div>
            </div>

            <!-- 空状态 / 拖拽提示 -->
            <div v-else-if="currentProjectFiles.length === 0" class="flex-1 flex flex-col items-center justify-center min-h-[160px]">
              <div class="w-16 h-16 bg-blue-100/50 rounded-full flex items-center justify-center mb-4 text-blue-500">
                <span class="material-symbols-outlined text-3xl">upload_file</span>
              </div>
              <h3 class="text-lg font-bold text-slate-700 mb-1">拖拽文件到这里</h3>
              <p class="text-sm text-slate-400 mb-6">支持 JPG, PNG 格式，一次可上传多张</p>
              <button @click="triggerFileInput" class="px-8 py-2.5 bg-blue-500 text-white font-bold rounded-xl shadow-lg shadow-blue-500/20 hover:scale-[1.02] active:scale-95 transition-all">
                浏览文件
              </button>
            </div>

            <!-- 已上传文件展示 (当只是查看时) -->
            <div v-else class="flex-1 mb-6">
               <h4 class="text-left text-sm font-bold text-slate-700 mb-3 flex items-center justify-between">
                 <span>{{ currentProjectName }} - 已传文件</span>
                 <button @click="triggerFileInput" class="text-xs text-blue-500 hover:underline flex items-center">
                   <span class="material-symbols-outlined text-sm mr-1">add_circle</span> 继续上传
                 </button>
               </h4>
               <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-4">
                  <div v-for="file in currentProjectFiles" :key="file.id" 
                    class="relative aspect-square rounded-lg overflow-hidden border border-slate-200 group/item cursor-pointer"
                    @click="previewFile(file)"
                  >
                    <img v-if="file._thumbnail" :src="file._thumbnail" class="w-full h-full object-cover" />
                    <div v-else class="w-full h-full bg-slate-100 flex items-center justify-center">
                      <span class="material-symbols-outlined text-slate-300">image</span>
                    </div>
                    <!-- 状态标识 -->
                    <div class="absolute bottom-0 left-0 right-0 bg-black/60 text-white text-[10px] px-1 truncate py-0.5">
                      {{ file.original_filename }}
                    </div>
                    <!-- 删除按钮 -->
                    <div class="absolute inset-0 bg-black/40 opacity-0 group-hover/item:opacity-100 transition-opacity flex items-center justify-center">
                       <button @click.stop="deleteFile(file)" class="bg-red-500 text-white p-1 rounded-full hover:bg-red-600">
                          <span class="material-symbols-outlined text-sm block">delete</span>
                       </button>
                    </div>
                  </div>
               </div>
            </div>

            <!-- 底部：上传确认条 -->
            <div v-if="pendingFiles.length > 0" class="mt-4 pt-4 border-t border-slate-100 flex justify-between items-center animate-fade-in-up">
              <span class="text-sm text-slate-500">待上传: <b class="text-slate-800">{{ pendingFiles.length }}</b> 个文件</span>
              <div class="flex gap-3">
                 <button @click="pendingFiles = []" class="px-4 py-2 text-sm text-slate-500 hover:text-slate-700">取消</button>
                 <button @click="uploadFiles" :disabled="uploading" class="px-6 py-2 bg-blue-500 text-white font-bold rounded-lg shadow-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2">
                    <span v-if="uploading" class="material-symbols-outlined text-sm animate-spin">progress_activity</span>
                    <span v-else class="material-symbols-outlined text-sm">cloud_upload</span>
                    确认上传
                 </button>
              </div>
            </div>
            
            <input type="file" ref="fileInput" multiple accept="image/*" class="hidden" @change="handleFileSelect" />
          </div>

        </div>
      </div>

      <!-- 底部操作按钮区域 -->
      <div class="flex items-center justify-end gap-4 mb-8 pr-2">
        <button
          @click="startOcr"
          :disabled="loadingOcr || allFiles.length === 0"
          class="flex items-center gap-2 px-6 py-3 bg-emerald-500 text-white font-bold rounded-xl hover:bg-emerald-600 shadow-lg shadow-emerald-500/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span v-if="loadingOcr" class="material-symbols-outlined text-xl animate-spin">refresh</span>
          <span v-else class="material-symbols-outlined text-xl">spellcheck</span>
          {{ loadingOcr ? '识别中...' : 'OCR识别' }}
        </button>
        <button
          @click="startAi"
          :disabled="loadingAi || allFiles.length === 0"
          class="flex items-center gap-2 px-6 py-3 bg-blue-500 text-white font-bold rounded-xl hover:bg-blue-600 shadow-lg shadow-blue-500/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span v-if="loadingAi" class="material-symbols-outlined text-xl animate-spin">psychology</span>
          <span v-else class="material-symbols-outlined text-xl">psychology</span>
          {{ loadingAi ? '分析中...' : 'AI深度分析' }}
        </button>
        
        <!-- 结果按钮 -->
          <button v-if="hasOcrResult" @click="showOcrResult = true" class="px-4 py-3 bg-white border border-slate-200 text-slate-600 font-bold rounded-xl hover:bg-slate-50">
            识别结果
          </button>
          <button v-if="hasAiResult" @click="showAiResult = true" class="px-4 py-3 bg-white border border-slate-200 text-slate-600 font-bold rounded-xl hover:bg-slate-50">
            分析报告
          </button>
      </div>

      <!-- 历史记录区域 (简化版) -->
      <section v-if="historyRecords.length > 0">
        <div class="flex justify-between items-center mb-6">
          <h2 class="text-xl font-bold flex items-center gap-2 text-slate-800">
            <span class="material-symbols-outlined text-blue-500">folder_managed</span>
            历史存档记录
          </h2>
          <!-- <button class="text-sm text-blue-500 hover:underline">查看全部</button> -->
        </div>
        <div class="space-y-4">
           <div v-for="rec in historyRecords.slice(0, 3)" :key="rec.id" class="bg-white p-5 rounded-2xl border border-slate-100 hover:shadow-md transition-all flex items-center justify-between">
              <div class="flex items-center gap-4">
                 <div class="w-12 h-12 bg-blue-50 rounded-xl flex items-center justify-center text-blue-500">
                   <span class="material-symbols-outlined">description</span>
                 </div>
                 <div>
                    <h4 class="font-bold text-slate-800">{{ rec.checkup_date }} 检查</h4>
                    <div class="text-xs text-slate-400 mt-1 flex gap-2">
                       <span>{{ rec.file_count || 0 }} 个文件</span>
                       <span v-if="rec.notes" class="truncate max-w-[200px]">{{ rec.notes }}</span>
                    </div>
                 </div>
              </div>
              <div class="flex gap-2">
                 <el-tag :type="rec.status === 'ai_done' ? 'success' : 'info'" size="small">{{ statusLabel(rec.status) }}</el-tag>
              </div>
           </div>
        </div>
      </section>
    </div>

    <!-- 右侧：AI 状态助手栏 -->
    <aside class="w-80 bg-white border-l border-slate-200 p-6 hidden xl:block rounded-xl sticky top-0 h-fit">
      <div class="bg-blue-50/50 rounded-2xl p-6 border border-blue-100">
        <h3 class="font-bold text-slate-900 mb-4 flex items-center gap-2">
          <span class="material-symbols-outlined text-blue-500">auto_awesome</span>
          AI 状态助手
        </h3>
        
        <div class="space-y-6">
           <!-- OCR 状态 -->
           <div class="flex items-start gap-3" :class="{'opacity-50': !ocrProgress.total && !hasOcrResult}">
             <div class="w-6 h-6 rounded-full flex items-center justify-center shrink-0 mt-0.5"
               :class="hasOcrResult ? 'bg-emerald-500 text-white' : 'bg-slate-200 text-slate-400'">
               <span class="material-symbols-outlined text-xs">{{ hasOcrResult ? 'check' : 'document_scanner' }}</span>
             </div>
             <div>
                <p class="text-sm font-bold text-slate-800 mb-1">OCR 智能识别</p>
                <p v-if="loadingOcr" class="text-xs text-blue-600 font-medium">
                   正在识别: ({{ ocrProgress.completed }}/{{ ocrProgress.total }})<br/>
                   <span class="text-[10px] text-slate-400 truncate w-40 block">{{ ocrProgress.current_file }}</span>
                </p>
                <p v-else-if="hasOcrResult" class="text-xs text-slate-500">
                   已完成识别，提取指标数据。
                </p>
                <p v-else class="text-xs text-slate-400">待开始</p>
             </div>
           </div>

           <!-- AI 状态 -->
           <div class="flex items-start gap-3" :class="{'opacity-50': !loadingAi && !hasAiResult}">
             <div class="w-6 h-6 rounded-full flex items-center justify-center shrink-0 mt-0.5"
                :class="hasAiResult ? 'bg-emerald-500 text-white' : 'bg-slate-200 text-slate-400'">
                <span class="material-symbols-outlined text-xs">{{ hasAiResult ? 'check' : 'psychology' }}</span>
             </div>
             <div>
                <p class="text-sm font-bold text-slate-800 mb-1">AI深度分析</p>
                <p v-if="loadingAi" class="text-xs text-blue-600 font-medium animate-pulse">
                   正在分析健康趋势...
                </p>
                <p v-else-if="hasAiResult" class="text-xs text-slate-500">
                   分析报告已生成。
                </p>
                <p v-else class="text-xs text-slate-400">待开始（需先完成 OCR）</p>
             </div>
           </div>
        </div>
      </div>

      <div class="mt-8">
        <h4 class="text-sm font-bold text-slate-900 mb-4 px-2">上传小贴士</h4>
        <div class="space-y-3">
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-[11px] text-slate-500 leading-relaxed">
            拍摄时请确保化验单平整，光线均匀，避免文字模糊或反光。
          </div>
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-[11px] text-slate-500 leading-relaxed">
            支持点击左侧不同项目，分别上传对应的检查单，系统会自动归档。
          </div>
        </div>
      </div>
    </aside>

    <!-- 弹窗组件 -->
    <el-dialog v-model="showOcrResult" title="OCR 识别结果" width="800px">
       <div v-if="ocrResults.length === 0" class="text-center py-10 text-slate-400">暂无结果</div>
       <div v-else class="max-h-[500px] overflow-y-auto space-y-4">
          <div v-for="res in ocrResults" :key="res.id" class="border p-4 rounded-lg">
             <div class="flex justify-between mb-2">
                <span class="font-bold text-sm">{{ res.original_filename }}</span>
                <el-tag :type="res.status==='success'?'success':'danger'" size="small">{{ res.status }}</el-tag>
             </div>
             <div v-if="res.status === 'success'" class="text-xs">
                <el-table :data="parseOcrItems(res.parsed_items)" size="small" border stripe>
                   <el-table-column prop="name" label="名称" />
                   <el-table-column prop="value" label="值" width="80" />
                   <el-table-column prop="unit" label="单位" width="80" />
                   <el-table-column prop="reference_range" label="参考" width="100" />
                   <el-table-column label="状态" width="60" align="center">
                      <template #default="{row}">
                         <span v-if="row.is_abnormal" class="text-red-500 font-bold">异常</span>
                      </template>
                   </el-table-column>
                </el-table>
             </div>
             <div v-else class="text-red-500 text-xs">{{ res.error_message }}</div>
          </div>
       </div>
    </el-dialog>

    <el-dialog v-model="showAiResult" title="AI 分析报告" width="800px">
       <div v-if="aiResult" class="prose prose-sm max-w-none p-4 bg-slate-50 rounded-lg" v-html="renderMarkdown(aiResult.response_content)"></div>
       <div v-else class="text-center py-10 text-slate-400">暂无分析结果</div>
    </el-dialog>

    <el-dialog v-model="showPreview" title="预览" width="80%" top="5vh">
      <img :src="previewSrc" class="max-w-full max-h-[70vh] mx-auto object-contain rounded" />
    </el-dialog>

  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElNotification } from 'element-plus'

// --- 状态 ---
const todayDate = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
const record = ref(null)
const projects = ref([])
const currentProjectId = ref('')
const allFiles = ref([]) // 当前记录的所有文件
const historyRecords = ref([])

// 上传相关
const fileInput = ref(null)
const pendingFiles = ref([]) // { name, base64, preview }
const uploading = ref(false)

// 任务状态
const loadingOcr = ref(false)
const loadingAi = ref(false)
const ocrProgress = reactive({ total: 0, completed: 0, current_file: '' })
const ocrResults = ref([])
const aiResult = ref(null)

// 弹窗
const showOcrResult = ref(false)
const showAiResult = ref(false)
const showPreview = ref(false)
const previewSrc = ref('')

// --- 计算属性 ---
const currentProject = computed(() => projects.value.find(p => p.id === currentProjectId.value))
const currentProjectName = computed(() => currentProject.value?.name || '')
const currentProjectFiles = computed(() => {
  if (!currentProjectId.value) return []
  return allFiles.value.filter(f => f.project_id === currentProjectId.value)
})

const hasOcrResult = computed(() => ocrResults.value.some(r => r.status === 'success'))
const hasAiResult = computed(() => !!aiResult.value)

// --- 初始化 ---
const init = async () => {
  try {
    // 1. 获取/创建今日记录
    record.value = await invoke('get_or_create_today_record')
    
    // 2. 获取项目列表
    const allProjs = await invoke('list_projects')
    projects.value = allProjs.filter(p => p.is_active).sort((a,b) => a.sort_order - b.sort_order)
    if (projects.value.length > 0) {
      currentProjectId.value = projects.value[0].id
    }

    // 3. 加载当前记录的文件
    await refreshFiles()

    // 4. 加载历史记录 (排除今天)
    const allRecs = await invoke('list_records')
    historyRecords.value = allRecs.filter(r => r.id !== record.value.id)

    // 5. 检查当前的 OCR 和 AI 状态
    await refreshStatus()

  } catch (e) {
    ElMessage.error('初始化失败: ' + e)
  }
}

const refreshFiles = async () => {
  if (!record.value) return
  try {
     const files = await invoke('list_files', { recordId: record.value.id })
     // 异步加载缩略图
     files.forEach(async f => {
        try {
           f._thumbnail = await invoke('read_file_base64', { fileId: f.id })
        } catch {}
     })
     allFiles.value = files
  } catch (e) { console.error(e) }
}

const refreshStatus = async () => {
   if (!record.value) return
   try {
     ocrResults.value = await invoke('get_ocr_results', { recordId: record.value.id })
     const aiList = await invoke('get_ai_analysis', { recordId: record.value.id })
     if (aiList && aiList.length > 0) {
        aiResult.value = aiList[aiList.length - 1] // 取最新的
     }
   } catch {}
}

// --- 文件操作 ---
const triggerFileInput = () => fileInput.value?.click()

const handleFileSelect = (e) => processFiles(e.target.files)
const handleDrop = (e) => processFiles(e.dataTransfer.files)

const processFiles = (fileList) => {
  if (!fileList || fileList.length === 0) return
  Array.from(fileList).forEach(file => {
     if (!file.type.startsWith('image/')) return
     const reader = new FileReader()
     reader.onload = (e) => {
        pendingFiles.value.push({
           name: file.name,
           raw: file,
           preview: e.target.result,
           base64: e.target.result.split(',')[1]
        })
     }
     reader.readAsDataURL(file)
  })
  // Reset input
  if (fileInput.value) fileInput.value.value = ''
}

const removePending = (index) => pendingFiles.value.splice(index, 1)

const uploadFiles = async () => {
  if (pendingFiles.value.length === 0) return
  uploading.value = true
  try {
     const files = pendingFiles.value.map(f => ({
        record_id: record.value.id,
        project_id: currentProjectId.value,
        checkup_date: record.value.checkup_date,
        file_data: f.base64,
        filename: f.name
     }))
     
     await invoke('upload_files', { files })
     ElMessage.success('上传成功')
     pendingFiles.value = []
     await refreshFiles()
  } catch (e) {
     ElMessage.error('上传失败: ' + e)
  } finally {
     uploading.value = false
  }
}

const deleteFile = async (file) => {
  try {
     await invoke('delete_file', { fileId: file.id })
     await refreshFiles()
  } catch (e) { ElMessage.error('' + e) }
}

const previewFile = async (file) => {
   previewSrc.value = file._thumbnail || await invoke('read_file_base64', { fileId: file.id })
   showPreview.value = true
}

// --- 业务操作 ---
const startOcr = async () => {
   if (!record.value) return
   loadingOcr.value = true
   ocrProgress.total = 0 // reset
   try {
      await invoke('start_ocr', { recordId: record.value.id })
      ElMessage.info('OCR 任务已提交')
   } catch (e) {
      loadingOcr.value = false
      ElMessage.error(e)
   }
}

const startAi = async () => {
   if (!record.value) return
   loadingAi.value = true
   try {
      await invoke('start_ai_analysis', { recordId: record.value.id })
      ElMessage.info('AI 分析任务已提交')
   } catch (e) {
      loadingAi.value = false
      ElMessage.error(e)
   }
}

// --- 辅助 ---
const getProjectIcon = (name) => {
  if (name.includes('血')) return 'bloodtype'
  if (name.includes('肝')) return 'monitor_heart'
  if (name.includes('影') || name.includes('CT') || name.includes('X')) return 'radiology'
  if (name.includes('尿')) return 'water_drop'
  return 'description'
}

const getProjectFileCount = (pid) => {
   return allFiles.value.filter(f => f.project_id === pid).length
}

const statusLabel = (s) => {
   const m = { pending_upload: '待上传', pending_ocr: '待识别', ocr_done: 'OCR完成', ai_done: '已完成' }
   return m[s] || s
}

const parseOcrItems = (str) => {
   try { return JSON.parse(str) } catch { return [] }
}

const renderMarkdown = (text) => {
   if (!text) return ''
   return text
     .replace(/^### (.*$)/gim, '<h3>$1</h3>')
     .replace(/^## (.*$)/gim, '<h2>$1</h2>')
     .replace(/^# (.*$)/gim, '<h1>$1</h1>')
     .replace(/\*\*(.*)\*\*/gim, '<b>$1</b>')
     .replace(/\n/gim, '<br>')
}

// --- Events ---
let listeners = []

onMounted(async () => {
   await init()
   
   listeners.push(await listen('ocr_progress', e => {
      loadingOcr.value = true
      Object.assign(ocrProgress, e.payload)
   }))
   listeners.push(await listen('ocr_complete', async () => {
      loadingOcr.value = false
      ElNotification.success('OCR 识别完成')
      await refreshStatus()
   }))
   listeners.push(await listen('ocr_error', e => {
      loadingOcr.value = false
      ElMessage.error(e.payload.error)
   }))
   
   listeners.push(await listen('ai_stream_done', async () => {
      loadingAi.value = false
      ElNotification.success('AI 分析完成')
      await refreshStatus()
   }))
   
   // 这里简单处理流式，实际可以 accumulating
   listeners.push(await listen('ai_stream_error', e => {
      loadingAi.value = false
      ElMessage.error(e.payload.error)
   }))
})

onUnmounted(() => {
   listeners.forEach(fn => fn())
})
</script>

<style scoped>
/* 简单动画 */
@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in-up {
  animation: fadeInUp 0.3s ease-out;
}
</style>
