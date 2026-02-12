<template>
  <div class="upload-page">
    <header class="flex justify-between items-center mb-6">
      <div>
        <h1 class="text-3xl font-bold text-slate-900">检查数据上传与存档</h1>
        <p class="text-slate-500 mt-1">上传体检报告图片，通过 OCR 智能识别 + AI 深度分析，管理您的每次检查数据。</p>
      </div>
      <el-button type="primary" @click="showCreateDialog = true">
        <span class="material-symbols-outlined text-sm mr-1">add_circle</span>
        新建检查记录
      </el-button>
    </header>

    <!-- 检查记录列表 -->
    <div v-if="records.length === 0" class="text-center py-20">
      <span class="material-symbols-outlined text-6xl text-slate-300 block mb-4">description</span>
      <p class="text-slate-400 text-lg mb-4">暂无检查记录</p>
      <el-button type="primary" @click="showCreateDialog = true">创建第一条检查记录</el-button>
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="record in records"
        :key="record.id"
        class="bg-white rounded-xl border border-slate-200 overflow-hidden transition-shadow hover:shadow-md"
      >
        <!-- 记录卡片头 -->
        <div class="flex items-center justify-between px-6 py-4 cursor-pointer"
          @click="toggleExpand(record.id)">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl flex items-center justify-center"
              :class="statusBgClass(record.status)">
              <span class="material-symbols-outlined text-2xl" :class="statusIconClass(record.status)">
                {{ statusIcon(record.status) }}
              </span>
            </div>
            <div>
              <div class="font-bold text-lg text-slate-900">{{ record.checkup_date }}</div>
              <div class="flex items-center gap-3 mt-1">
                <el-tag size="small" :type="statusTagType(record.status)">{{ statusLabel(record.status) }}</el-tag>
                <span class="text-xs text-slate-400">{{ record.file_count || 0 }} 个文件</span>
                <template v-if="record.project_names?.length">
                  <el-tag v-for="pn in record.project_names" :key="pn" size="small" type="info" class="!rounded">{{ pn }}</el-tag>
                </template>
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <el-button link type="danger" size="small" @click.stop="handleDeleteRecord(record)">
              <span class="material-symbols-outlined">delete</span>
            </el-button>
            <span class="material-symbols-outlined text-slate-400 transition-transform"
              :class="{ 'rotate-180': expandedId === record.id }">
              expand_more
            </span>
          </div>
        </div>

        <!-- 展开的详情区域 -->
        <div v-if="expandedId === record.id" class="border-t border-slate-100 px-6 py-5 bg-slate-50/50">
          <!-- 文件上传区域 -->
          <div class="mb-6">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-sm font-bold text-slate-700">
                <span class="material-symbols-outlined text-base align-middle mr-1">upload_file</span>
                上传检查报告图片
              </h3>
            </div>

            <!-- 项目选择 + 文件上传 -->
            <div class="flex gap-4 mb-4">
              <el-select v-model="selectedProjectId" placeholder="选择检查项目" class="w-48" size="default">
                <el-option v-for="p in activeProjects" :key="p.id" :label="p.name" :value="p.id" />
              </el-select>
              <el-upload
                ref="uploadRef"
                :auto-upload="false"
                :on-change="handleFileChange"
                multiple
                accept="image/*"
                :show-file-list="false"
                class="flex-1"
              >
                <template #trigger>
                  <el-button>
                    <span class="material-symbols-outlined text-sm mr-1">attach_file</span>
                    选择文件
                  </el-button>
                </template>
              </el-upload>
              <el-button type="primary" :disabled="!selectedProjectId || pendingFiles.length === 0"
                :loading="uploading" @click="doUpload(record)">
                <span class="material-symbols-outlined text-sm mr-1">cloud_upload</span>
                上传 ({{ pendingFiles.length }})
              </el-button>
            </div>

            <!-- 待上传文件预览 -->
            <div v-if="pendingFiles.length > 0" class="flex flex-wrap gap-2 mb-4">
              <div v-for="(f, idx) in pendingFiles" :key="idx"
                class="relative w-20 h-20 rounded-lg overflow-hidden border border-slate-200 group">
                <img :src="f.preview" class="w-full h-full object-cover" />
                <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition flex items-center justify-center">
                  <el-button link size="small" @click="removePending(idx)" class="!text-white">
                    <span class="material-symbols-outlined">close</span>
                  </el-button>
                </div>
                <div class="absolute bottom-0 left-0 right-0 bg-black/50 text-white text-[10px] px-1 truncate">
                  {{ f.name }}
                </div>
              </div>
            </div>
          </div>

          <!-- 已上传文件列表 -->
          <div v-if="currentFiles.length > 0">
            <h3 class="text-sm font-bold text-slate-700 mb-3">
              <span class="material-symbols-outlined text-base align-middle mr-1">folder_open</span>
              已上传文件 ({{ currentFiles.length }})
            </h3>

            <!-- 按项目分组展示 -->
            <div v-for="group in fileGroups" :key="group.projectName" class="mb-4">
              <div class="text-xs text-slate-500 font-medium mb-2">📁 {{ group.projectName }}</div>
              <div class="flex flex-wrap gap-3">
                <div v-for="file in group.files" :key="file.id"
                  class="relative w-28 h-28 rounded-lg overflow-hidden border border-slate-200 cursor-pointer group shadow-sm hover:shadow-md transition"
                  @click="previewFile(file)">
                  <img v-if="file._thumbnail" :src="file._thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full flex items-center justify-center bg-slate-100">
                    <span class="material-symbols-outlined text-3xl text-slate-300">image</span>
                  </div>
                  <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition flex items-center justify-center gap-2">
                    <el-button link size="small" class="!text-white" @click.stop="previewFile(file)">
                      <span class="material-symbols-outlined">visibility</span>
                    </el-button>
                    <el-button link size="small" class="!text-red-300" @click.stop="handleDeleteFile(file)">
                      <span class="material-symbols-outlined">delete</span>
                    </el-button>
                  </div>
                  <div class="absolute bottom-0 left-0 right-0 bg-black/60 text-white text-[10px] px-1 py-0.5 truncate">
                    {{ file.original_filename }}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 操作按钮区 -->
          <div class="flex gap-3 mt-4 pt-4 border-t border-slate-200">
            <el-button
              type="warning"
              :disabled="currentFiles.length === 0 || record.status === 'ocr_processing'"
              :loading="ocrLoading"
              @click="startOcr(record)">
              <span class="material-symbols-outlined text-sm mr-1">document_scanner</span>
              发起 OCR 识别
            </el-button>
            <el-button
              type="success"
              :disabled="record.status !== 'ocr_done' && record.status !== 'ai_done'"
              :loading="aiLoading"
              @click="startAiAnalysis(record)">
              <span class="material-symbols-outlined text-sm mr-1">psychology</span>
              发起 AI 分析
            </el-button>

            <div class="flex-1"></div>

            <!-- OCR 结果 / AI 分析结果查看 -->
            <el-button v-if="record.status === 'ocr_done' || record.status === 'ai_done'"
              link type="primary" @click="viewOcrResults(record)">
              查看 OCR 结果
            </el-button>
            <el-button v-if="record.status === 'ai_done'"
              link type="primary" @click="viewAiResult(record)">
              查看 AI 分析
            </el-button>
          </div>

          <!-- OCR 进度条 -->
          <div v-if="ocrLoading && ocrProgress.record_id === record.id" class="mt-4 p-4 bg-amber-50 rounded-lg">
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm font-medium text-amber-700">
                <span class="material-symbols-outlined text-sm align-middle mr-1">hourglass_top</span>
                OCR 识别中...
              </span>
              <span class="text-xs text-amber-600">{{ ocrProgress.completed }} / {{ ocrProgress.total }}</span>
            </div>
            <el-progress :percentage="Math.round((ocrProgress.completed / ocrProgress.total) * 100)" :stroke-width="8" />
            <div class="text-xs text-amber-500 mt-1">正在处理: {{ ocrProgress.current_file }}</div>
          </div>

          <!-- AI 流式输出区域 -->
          <div v-if="(aiLoading || aiStreamContent) && aiStreamRecordId === record.id" class="mt-4">
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-bold text-slate-700">
                <span class="material-symbols-outlined text-base align-middle mr-1">psychology</span>
                AI 分析结果
              </h3>
              <el-tag v-if="aiLoading" type="primary" size="small" effect="light">
                <span class="material-symbols-outlined text-xs align-middle animate-spin mr-1">progress_activity</span>
                分析中...
              </el-tag>
              <el-tag v-else type="success" size="small">分析完成</el-tag>
            </div>
            <div class="p-4 bg-white rounded-lg border border-slate-200 max-h-[500px] overflow-y-auto prose prose-sm prose-slate max-w-none"
              v-html="renderedAiContent">
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新建检查记录弹窗 -->
    <el-dialog v-model="showCreateDialog" title="新建检查记录" width="400px" :close-on-click-modal="false">
      <el-form label-position="top">
        <el-form-item label="检查日期" required>
          <el-date-picker v-model="newRecordDate" type="date" placeholder="选择检查日期"
            value-format="YYYY-MM-DD" class="!w-full" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="newRecordNotes" type="textarea" :rows="3" placeholder="可选，如：常规体检、专项复查..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createRecord" :loading="creating">创建</el-button>
      </template>
    </el-dialog>

    <!-- 图片预览弹窗 -->
    <el-dialog v-model="showPreview" title="图片预览" width="80%" top="5vh" :close-on-click-modal="true">
      <div class="flex items-center justify-center min-h-[60vh]">
        <img v-if="previewSrc" :src="previewSrc" class="max-w-full max-h-[75vh] object-contain rounded-lg" />
        <div v-else class="text-slate-400">
          <el-icon class="is-loading" :size="32"><Loading /></el-icon>
          <span class="ml-2">加载中...</span>
        </div>
      </div>
      <template #footer>
        <span class="text-sm text-slate-500">{{ previewFilename }}</span>
      </template>
    </el-dialog>

    <!-- OCR 结果弹窗 -->
    <el-dialog v-model="showOcrDialog" title="OCR 识别结果" width="70%" top="5vh">
      <div v-if="ocrResults.length === 0" class="text-center py-10 text-slate-400">
        暂无 OCR 结果
      </div>
      <div v-else class="space-y-4">
        <div v-for="ocr in ocrResults" :key="ocr.id" class="border border-slate-200 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <el-tag :type="ocr.status === 'success' ? 'success' : 'danger'" size="small">
              {{ ocr.status === 'success' ? '识别成功' : '识别失败' }}
            </el-tag>
            <span class="text-xs text-slate-400">{{ ocr.created_at }}</span>
          </div>
          <div v-if="ocr.status === 'success'">
            <el-table :data="parseOcrItems(ocr.parsed_items)" stripe size="small" max-height="300">
              <el-table-column prop="name" label="指标名称" min-width="150" />
              <el-table-column prop="value" label="数值" width="100" />
              <el-table-column prop="unit" label="单位" width="100" />
              <el-table-column prop="reference_range" label="参考范围" width="120" />
              <el-table-column label="状态" width="80" align="center">
                <template #default="{ row }">
                  <el-tag v-if="row.is_abnormal" type="danger" size="small">异常</el-tag>
                  <el-tag v-else type="success" size="small">正常</el-tag>
                </template>
              </el-table-column>
            </el-table>
          </div>
          <div v-else class="text-red-500 text-sm">
            {{ ocr.error_message }}
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- AI 分析结果弹窗 -->
    <el-dialog v-model="showAiDialog" title="AI 分析结果" width="70%" top="5vh">
      <div v-if="aiResults.length === 0" class="text-center py-10 text-slate-400">
        暂无 AI 分析结果
      </div>
      <div v-else>
        <div v-for="analysis in aiResults" :key="analysis.id" class="mb-4">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <el-tag :type="analysis.status === 'success' ? 'success' : 'danger'" size="small">
                {{ analysis.status === 'success' ? '分析成功' : (analysis.status === 'processing' ? '分析中' : '分析失败') }}
              </el-tag>
              <span class="text-xs text-slate-500">模型: {{ analysis.model_used }}</span>
            </div>
            <span class="text-xs text-slate-400">{{ analysis.created_at }}</span>
          </div>
          <div v-if="analysis.status === 'success'"
            class="p-4 bg-slate-50 rounded-lg prose prose-sm prose-slate max-w-none"
            v-html="renderMarkdown(analysis.response_content)">
          </div>
          <div v-else-if="analysis.status === 'failed'" class="text-red-500 text-sm">
            {{ analysis.error_message }}
          </div>
          <div v-else class="text-slate-400 text-sm flex items-center gap-2">
            <el-icon class="is-loading"><Loading /></el-icon>
            分析处理中，请稍候...
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'

// ===== 检查记录 =====
const records = ref([])
const expandedId = ref(null)
const showCreateDialog = ref(false)
const newRecordDate = ref('')
const newRecordNotes = ref('')
const creating = ref(false)

const loadRecords = async () => {
  try {
    records.value = await invoke('list_records')
  } catch (e) {
    ElMessage.error('加载检查记录失败: ' + e)
  }
}

const createRecord = async () => {
  if (!newRecordDate.value) {
    ElMessage.warning('请选择检查日期')
    return
  }
  creating.value = true
  try {
    await invoke('create_record', {
      input: {
        checkup_date: newRecordDate.value,
        notes: newRecordNotes.value || null,
      }
    })
    ElMessage.success('检查记录创建成功')
    showCreateDialog.value = false
    newRecordDate.value = ''
    newRecordNotes.value = ''
    await loadRecords()
    // 自动展开最新记录
    if (records.value.length > 0) {
      expandedId.value = records.value[0].id
    }
  } catch (e) {
    ElMessage.error('' + e)
  } finally {
    creating.value = false
  }
}

const handleDeleteRecord = async (record) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除 ${record.checkup_date} 的检查记录吗？\n该操作将同时删除所有关联文件、OCR 结果和 AI 分析。`,
      '确认删除',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' }
    )
    await invoke('delete_record', { id: record.id })
    ElMessage.success('记录已删除')
    if (expandedId.value === record.id) expandedId.value = null
    await loadRecords()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('' + e)
  }
}

const toggleExpand = (id) => {
  if (expandedId.value === id) {
    expandedId.value = null
  } else {
    expandedId.value = id
    loadFiles(id)
  }
}

// ===== 项目列表 =====
const activeProjects = ref([])
const selectedProjectId = ref('')

const loadProjects = async () => {
  try {
    const all = await invoke('list_projects')
    activeProjects.value = all.filter(p => p.is_active)
    if (activeProjects.value.length > 0 && !selectedProjectId.value) {
      selectedProjectId.value = activeProjects.value[0].id
    }
  } catch (e) {
    console.error('加载项目失败:', e)
  }
}

// ===== 文件上传 =====
const pendingFiles = ref([])
const uploading = ref(false)
const uploadRef = ref(null)

const handleFileChange = (file) => {
  // 转为 base64 预览
  const reader = new FileReader()
  reader.onload = (e) => {
    pendingFiles.value.push({
      name: file.name,
      raw: file.raw,
      preview: e.target.result,
      base64: e.target.result.split(',')[1], // 去掉 data:xxx;base64, 前缀
    })
  }
  reader.readAsDataURL(file.raw)
}

const removePending = (index) => {
  pendingFiles.value.splice(index, 1)
}

const doUpload = async (record) => {
  if (!selectedProjectId.value) {
    ElMessage.warning('请先选择检查项目')
    return
  }
  uploading.value = true
  try {
    const files = pendingFiles.value.map(f => ({
      record_id: record.id,
      project_id: selectedProjectId.value,
      checkup_date: record.checkup_date,
      file_data: f.base64,
      filename: f.name,
    }))
    await invoke('upload_files', { files })
    ElMessage.success(`${files.length} 个文件上传成功`)
    pendingFiles.value = []
    await loadFiles(record.id)
    await loadRecords()
  } catch (e) {
    ElMessage.error('上传失败: ' + e)
  } finally {
    uploading.value = false
  }
}

// ===== 已上传文件 =====
const currentFiles = ref([])

const loadFiles = async (recordId) => {
  try {
    const files = await invoke('list_files', { recordId })
    // 生成缩略图
    for (const f of files) {
      try {
        f._thumbnail = await invoke('read_file_base64', { fileId: f.id })
      } catch {
        f._thumbnail = null
      }
    }
    currentFiles.value = files
  } catch (e) {
    console.error('加载文件失败:', e)
    currentFiles.value = []
  }
}

// 按项目分组
const fileGroups = computed(() => {
  const groups = {}
  for (const f of currentFiles.value) {
    const name = f.project_name || '未知项目'
    if (!groups[name]) groups[name] = { projectName: name, files: [] }
    groups[name].files.push(f)
  }
  return Object.values(groups)
})

const handleDeleteFile = async (file) => {
  try {
    await ElMessageBox.confirm(`确定要删除文件「${file.original_filename}」吗？`, '确认删除', {
      type: 'warning'
    })
    await invoke('delete_file', { fileId: file.id })
    ElMessage.success('文件已删除')
    if (expandedId.value) await loadFiles(expandedId.value)
    await loadRecords()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('' + e)
  }
}

// ===== 图片预览 =====
const showPreview = ref(false)
const previewSrc = ref('')
const previewFilename = ref('')

const previewFile = async (file) => {
  previewFilename.value = file.original_filename
  previewSrc.value = ''
  showPreview.value = true
  try {
    previewSrc.value = file._thumbnail || await invoke('read_file_base64', { fileId: file.id })
  } catch (e) {
    ElMessage.error('图片加载失败: ' + e)
  }
}

// ===== OCR 功能 =====
const ocrLoading = ref(false)
const ocrProgress = reactive({
  record_id: '',
  total: 0,
  completed: 0,
  current_file: '',
})

const startOcr = async (record) => {
  ocrLoading.value = true
  ocrProgress.record_id = record.id
  ocrProgress.total = record.file_count || 0
  ocrProgress.completed = 0
  ocrProgress.current_file = ''

  try {
    await invoke('start_ocr', { recordId: record.id })
    ElMessage.info('OCR 识别已启动，请稍候...')
  } catch (e) {
    ocrLoading.value = false
    ElMessage.error('启动OCR失败: ' + e)
  }
}

// ===== AI 分析功能 =====
const aiLoading = ref(false)
const aiStreamContent = ref('')
const aiStreamRecordId = ref('')

const startAiAnalysis = async (record) => {
  aiLoading.value = true
  aiStreamContent.value = ''
  aiStreamRecordId.value = record.id

  try {
    await invoke('start_ai_analysis', { recordId: record.id })
    ElMessage.info('AI 分析已启动，请稍候...')
  } catch (e) {
    aiLoading.value = false
    ElMessage.error('启动AI分析失败: ' + e)
  }
}

// 简单的 Markdown 渲染（不依赖额外库）
const renderMarkdown = (text) => {
  if (!text) return ''
  return text
    // 标题
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    // 加粗
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    // 斜体
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // 列表
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/^(\d+)\. (.+)$/gm, '<li>$2</li>')
    // 代码块
    .replace(/```[\s\S]*?```/g, (m) => `<pre><code>${m.slice(3, -3)}</code></pre>`)
    // 行内代码
    .replace(/`(.+?)`/g, '<code>$1</code>')
    // 换行
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br/>')
}

const renderedAiContent = computed(() => renderMarkdown(aiStreamContent.value))

// ===== OCR 结果查看 =====
const showOcrDialog = ref(false)
const ocrResults = ref([])

const viewOcrResults = async (record) => {
  try {
    ocrResults.value = await invoke('get_ocr_results', { recordId: record.id })
    showOcrDialog.value = true
  } catch (e) {
    ElMessage.error('加载OCR结果失败: ' + e)
  }
}

const parseOcrItems = (jsonStr) => {
  try {
    return JSON.parse(jsonStr)
  } catch {
    return []
  }
}

// ===== AI 分析结果查看 =====
const showAiDialog = ref(false)
const aiResults = ref([])

const viewAiResult = async (record) => {
  try {
    aiResults.value = await invoke('get_ai_analysis', { recordId: record.id })
    showAiDialog.value = true
  } catch (e) {
    ElMessage.error('加载AI分析结果失败: ' + e)
  }
}

// ===== Tauri Event 监听 =====
let unlistenProgress = null
let unlistenComplete = null
let unlistenError = null
let unlistenAiChunk = null
let unlistenAiDone = null
let unlistenAiError = null

const setupEventListeners = async () => {
  // OCR 进度
  unlistenProgress = await listen('ocr_progress', (event) => {
    const data = event.payload
    ocrProgress.record_id = data.record_id
    ocrProgress.total = data.total
    ocrProgress.completed = data.completed
    ocrProgress.current_file = data.current_file
  })

  // OCR 完成
  unlistenComplete = await listen('ocr_complete', (event) => {
    const data = event.payload
    ocrLoading.value = false

    if (data.success > 0) {
      ElNotification({
        title: 'OCR 识别完成',
        message: `成功识别 ${data.success}/${data.total} 个文件`,
        type: 'success',
        duration: 5000,
      })
    } else {
      ElNotification({
        title: 'OCR 识别失败',
        message: '所有文件识别失败，请检查AI配置',
        type: 'error',
        duration: 5000,
      })
    }

    loadRecords()
  })

  // OCR 错误
  unlistenError = await listen('ocr_error', (event) => {
    const data = event.payload
    ocrLoading.value = false
    ElMessage.error('OCR 错误: ' + data.error)
    loadRecords()
  })

  // AI 流式 chunk
  unlistenAiChunk = await listen('ai_stream_chunk', (event) => {
    const data = event.payload
    aiStreamContent.value += data.content
  })

  // AI 完成
  unlistenAiDone = await listen('ai_stream_done', (event) => {
    aiLoading.value = false
    ElNotification({
      title: 'AI 分析完成',
      message: '健康分析报告已生成',
      type: 'success',
      duration: 5000,
    })
    loadRecords()
  })

  // AI 错误
  unlistenAiError = await listen('ai_stream_error', (event) => {
    const data = event.payload
    aiLoading.value = false
    ElMessage.error('AI 分析失败: ' + data.error)
    loadRecords()
  })
}

// ===== 状态辅助函数 =====
const statusLabel = (status) => {
  const map = {
    pending_upload: '待上传',
    pending_ocr: '待识别',
    ocr_processing: 'OCR 中',
    ocr_done: '已识别',
    ai_processing: 'AI 分析中',
    ai_done: '已分析',
  }
  return map[status] || status
}

const statusTagType = (status) => {
  const map = {
    pending_upload: 'info',
    pending_ocr: 'warning',
    ocr_processing: 'warning',
    ocr_done: '',
    ai_processing: '',
    ai_done: 'success',
  }
  return map[status] || 'info'
}

const statusIcon = (status) => {
  const map = {
    pending_upload: 'cloud_upload',
    pending_ocr: 'document_scanner',
    ocr_processing: 'hourglass_top',
    ocr_done: 'check_circle',
    ai_processing: 'psychology',
    ai_done: 'verified',
  }
  return map[status] || 'description'
}

const statusBgClass = (status) => {
  const map = {
    pending_upload: 'bg-slate-100',
    pending_ocr: 'bg-amber-50',
    ocr_processing: 'bg-amber-50',
    ocr_done: 'bg-blue-50',
    ai_processing: 'bg-blue-50',
    ai_done: 'bg-emerald-50',
  }
  return map[status] || 'bg-slate-100'
}

const statusIconClass = (status) => {
  const map = {
    pending_upload: 'text-slate-400',
    pending_ocr: 'text-amber-500',
    ocr_processing: 'text-amber-500',
    ocr_done: 'text-blue-500',
    ai_processing: 'text-blue-500',
    ai_done: 'text-emerald-500',
  }
  return map[status] || 'text-slate-400'
}

// ===== 初始化 =====
onMounted(() => {
  loadRecords()
  loadProjects()
  setupEventListeners()
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenComplete?.()
  unlistenError?.()
  unlistenAiChunk?.()
  unlistenAiDone?.()
  unlistenAiError?.()
})
</script>
