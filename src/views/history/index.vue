<template>
  <div class="history-page p-6 max-w-5xl mx-auto">
    <header class="mb-8 flex items-center gap-4">
      <div class="w-12 h-12 bg-blue-100 rounded-xl flex items-center justify-center text-blue-600">
        <span class="material-symbols-outlined text-3xl">history</span>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-slate-800">历史健康档案</h1>
        <p class="text-slate-500 text-sm mt-1">按时间线查看您的体检记录与 AI 健康分析</p>
      </div>
    </header>

    <div v-if="loading" class="py-20 text-center">
      <el-skeleton :rows="5" animated />
    </div>

    <div v-else-if="records.length === 0" class="py-20 text-center text-slate-400 bg-slate-50 rounded-xl">
      <span class="material-symbols-outlined text-4xl mb-2">content_paste_off</span>
      <p>暂无历史记录</p>
    </div>

    <el-timeline v-else class="pl-4">
      <el-timeline-item
        v-for="record in records"
        :key="record.id"
        :timestamp="record.checkup_date"
        placement="top"
        :color="getStatusColor(record)"
        size="large"
      >
        <el-card class="mb-6 rounded-xl border-slate-200 hover:shadow-lg transition-shadow">
          <!-- 头部信息 (可点击展开) -->
          <template #header>
            <div class="flex justify-between items-center cursor-pointer select-none" @click="record.isExpanded = !record.isExpanded">
              <div class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-slate-400 transition-transform duration-300" :class="{'rotate-90': record.isExpanded}">chevron_right</span>
                  <span class="font-bold text-lg text-slate-800">{{ record.checkup_date }} 体检记录</span>
              </div>
              <el-tag :type="getStatusType(record.status)">{{ getStatusLabel(record.status) }}</el-tag>
            </div>
          </template>
          
          <div v-show="record.isExpanded">

          <!-- 1. 异常指标表格 -->
          <div class="mb-6">
            <h4 class="font-bold text-slate-700 mb-3 flex items-center gap-2">
              <span class="material-symbols-outlined text-rose-500">warning</span>
              异常指标清单 
              <span class="text-xs font-normal text-slate-400 bg-slate-100 px-2 py-0.5 rounded-full" v-if="record.abnormal_items.length > 0">
                共 {{ record.abnormal_items.length }} 项
              </span>
            </h4>
            
            <div v-if="record.abnormal_items.length > 0">
              <el-table :data="record.abnormal_items" border stripe size="small" style="width: 100%">
                <el-table-column prop="project_name" label="检查项目" width="120">
                  <template #default="{row}">
                    <el-tag size="small" effect="plain">{{ row.project_name }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="name" label="指标名称" min-width="120" />
                <el-table-column label="检测结果" width="140">
                  <template #default="{row}">
                    <span class="text-rose-600 font-bold">{{ row.value }}</span>
                    <span class="text-xs text-slate-400 ml-1">{{ row.unit }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="reference_range" label="参考范围" width="120">
                   <template #default="{row}">
                      <span class="text-xs text-slate-500">{{ row.reference_range }}</span>
                   </template>
                </el-table-column>
              </el-table>
            </div>
            <div v-else class="bg-emerald-50 text-emerald-700 px-4 py-3 rounded-lg text-sm flex items-center gap-2">
              <span class="material-symbols-outlined text-lg">check_circle</span>
              本次检查各项指标均在正常范围内，未发现异常数据。
            </div>
          </div>

          <!-- 2. AI 分析建议 -->
          <div v-if="record.ai_analysis && record.ai_analysis.length > 0" class="bg-blue-50/50 rounded-xl p-5 border border-blue-100">
            <h4 class="font-bold text-blue-800 mb-3 flex items-center gap-2">
              <span class="material-symbols-outlined">auto_awesome</span>
              AI 健康建议
            </h4>
            
            <div class="text-sm text-slate-700 leading-relaxed relative">
              <!-- 内容区域 -->
              <div 
                ref="contentRefs"
                class="prose prose-sm max-w-none prose-blue"
                :class="{'line-clamp-4 overflow-hidden relative': !record.aiExpanded && isLongContent(record.ai_analysis)}"
                v-html="renderMarkdown(record.ai_analysis)">
              </div>
              
              <!-- 展开/收起按钮 -->
              <div v-if="isLongContent(record.ai_analysis)" class="mt-3 text-center border-t border-blue-100 pt-2">
                <button 
                  @click="record.aiExpanded = !record.aiExpanded"
                  class="text-blue-600 hover:text-blue-800 text-xs font-bold flex items-center justify-center gap-1 mx-auto transition-colors"
                >
                  <span class="material-symbols-outlined text-base">
                    {{ record.aiExpanded ? 'keyboard_arrow_up' : 'keyboard_arrow_down' }}
                  </span>
                  {{ record.aiExpanded ? '收起完整报告' : '展开查看详细分析' }}
                </button>
              </div>
            </div>
          </div>
          
          <div v-else class="mt-4 text-center text-xs text-slate-400 border-t border-slate-100 pt-3">
             暂无 AI 分析报告
          </div>
          </div>

        </el-card>
      </el-timeline-item>
    </el-timeline>

    <div v-if="!loading && hasMore && records.length > 0" class="text-center py-6">
        <el-button :loading="moreLoading" @click="handleLoadMore" type="primary" plain round>
            加载更多记录 <span class="material-symbols-outlined ml-1 text-sm">expand_more</span>
        </el-button>
    </div>
    
    <div v-if="!loading && !hasMore && records.length > 0" class="text-center py-8 text-slate-400 text-sm">
        — 已显示全部记录 —
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onActivated } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

// ===== 历史记录 =====
const records = ref([])
const loading = ref(false)
const moreLoading = ref(false)
const currentPage = ref(1)
const pageSize = 10
const hasMore = ref(true)

const loadData = async (reset = false) => {
  if (reset) {
    loading.value = true
    currentPage.value = 1
    hasMore.value = true
    records.value = []
  } else {
    moreLoading.value = true
  }

  try {
    const offset = (currentPage.value - 1) * pageSize
    const data = await invoke('get_history_timeline', { 
        limit: pageSize, 
        offset: offset 
    })
    
    const mappedData = data.map((item) => ({
      ...item,
      isExpanded: false, 
      aiExpanded: false
    }))
    
    if (data.length < pageSize) {
        hasMore.value = false
    }

    if (reset) {
        if (mappedData.length > 0) mappedData[0].isExpanded = true
        records.value = mappedData
    } else {
        records.value = [...records.value, ...mappedData]
    }
    
    // Only increment page if we got data, or if it's the first empty page (to avoid infinite loop if logic changes)
    // Actually, simply incrementing is fine as offset depends on it. 
    // If we got 0 items, next request will use same offset if we don't increment, 
    // but we set hasMore=false so we won't request again.
    // If we got < pageSize, hasMore=false.
    if (data.length > 0) {
        currentPage.value++
    }

  } catch (error) {
    ElMessage.error('加载历史记录失败: ' + error)
    console.error(error)
  } finally {
    loading.value = false
    moreLoading.value = false
  }
}

const handleLoadMore = () => {
    loadData(false)
}

onMounted(() => {
  loadData(true)
})

onActivated(() => {
  loadData(true)
})

const getStatusColor = (record) => {
  if (record.abnormal_items.length > 0) return '#f43f5e' // red
  if (record.status === 'ai_done') return '#10b981' // green
  return '#94a3b8' // gray
}

const getStatusType = (status) => {
  const map = {
    'pending_upload': 'info',
    'pending_ocr': 'warning',
    'ocr_processing': 'warning',
    'ocr_done': 'primary',
    'ai_processing': 'primary',
    'ai_done': 'success'
  }
  return map[status] || 'info'
}

const getStatusLabel = (status) => {
  const map = {
    'pending_upload': '待上传',
    'pending_ocr': '待识别',
    'ocr_processing': '识别中',
    'ocr_done': 'OCR完成',
    'ai_processing': '分析中',
    'ai_done': '已分析'
  }
  return map[status] || status
}

const isLongContent = (content) => {
  return content && content.length > 100
}

const renderMarkdown = (text) => {
   if (!text) return ''
   // 简单 Markdown 解析
   let html = text
     // Headers
     .replace(/^### (.*$)/gim, '<h3 class="font-bold text-base mt-3 mb-1 text-slate-800">$1</h3>')
     .replace(/^## (.*$)/gim, '<h2 class="font-bold text-lg mt-4 mb-2 text-slate-900 border-b pb-1">$1</h2>')
     // Bold
     .replace(/\*\*(.*?)\*\*/gim, '<b>$1</b>')
     // Lists
     .replace(/^- (.*$)/gim, '<li class="ml-4 list-disc">$1</li>')
     // Line breaks
     .replace(/\n/gim, '<br>')
   
   return html
}
</script>

<style scoped>
.line-clamp-4 {
  display: -webkit-box;
  -webkit-line-clamp: 4;
  line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
