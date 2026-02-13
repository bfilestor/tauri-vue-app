<template>
  <div class="aiqa-page flex h-full p-4 overflow-hidden gap-4">
    <!-- Left Column: History -->
    <section class="w-[35%] flex flex-col bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
        <div class="p-4 border-b border-slate-100 bg-slate-50/50 flex justify-between items-center shrink-0">
            <h3 class="text-base font-bold text-slate-800 flex items-center gap-2">
                <span class="material-symbols-outlined text-primary">history_edu</span>
                历史 AI 分析建议
            </h3>
            <span class="text-xs text-slate-400">共 {{ historyTotal }} 条</span>
        </div>
        
        <div class="flex-1 overflow-y-auto custom-scrollbar p-4 space-y-4" @scroll="handleHistoryScroll">
             <div v-if="historyLoading && historyRecords.length === 0" class="py-10 text-center">
                <el-skeleton :rows="3" />
             </div>

             <div v-if="!historyLoading && historyRecords.length === 0" class="py-20 text-center text-slate-400">
                <span class="material-symbols-outlined text-4xl mb-2 opacity-50">description</span>
                <p class="text-xs">暂无历史分析记录</p>
             </div>

             <div 
                v-for="record in historyRecords" 
                :key="record.id"
                class="border border-slate-100 rounded-lg overflow-hidden transition-all hover:shadow-md"
                :class="{'ring-1 ring-primary/20': record.isExpanded}"
             >
                <!-- Card Header -->
                <div 
                    class="p-3 bg-slate-50 cursor-pointer flex justify-between items-center select-none hover:bg-slate-100 transition-colors"
                    @click="toggleExpand(record)"
                >
                    <div class="flex items-center gap-2 overflow-hidden">
                        <span class="material-symbols-outlined text-slate-400 text-sm transition-transform duration-200" :class="{'rotate-90': record.isExpanded}">chevron_right</span>
                        <div class="truncate">
                            <span class="text-sm font-bold text-slate-700">{{ record.checkup_date }}</span>
                            <span class="text-xs text-slate-400 ml-2">体检分析</span>
                        </div>
                    </div>
                    <span class="text-[10px] text-slate-400 bg-white px-1.5 py-0.5 rounded border border-slate-100">
                        {{ formatTime(record.created_at) }}
                    </span>
                </div>

                <!-- Card Content -->
                <div v-if="record.isExpanded" class="p-3 border-t border-slate-100 bg-white">
                    
                    <!-- View Mode -->
                    <div v-if="!record.isEditing">
                        <div class="prose prose-sm prose-slate max-w-none text-xs leading-relaxed max-h-[400px] overflow-y-auto custom-scrollbar" v-html="renderMarkdown(record.response_content)"></div>
                        
                        <div class="flex justify-end gap-2 mt-3 pt-2 border-t border-slate-50">
                            <button 
                                @click.stop="copyToInput(record.response_content)"
                                class="text-xs flex items-center gap-1 text-primary hover:text-primary/80 px-2 py-1 hover:bg-primary/5 rounded transition-colors"
                                title="引用到右侧输入框"
                            >
                                <span class="material-symbols-outlined text-sm">input</span>
                                引用
                            </button>
                            <button 
                                @click.stop="startEdit(record)"
                                class="text-xs flex items-center gap-1 text-slate-400 hover:text-slate-600 px-2 py-1 hover:bg-slate-50 rounded transition-colors"
                            >
                                <span class="material-symbols-outlined text-sm">edit</span>
                                编辑
                            </button>
                        </div>
                    </div>

                    <!-- Edit Mode -->
                    <div v-else>
                        <textarea 
                            v-model="record.editContent"
                            class="w-full text-xs p-2 border border-blue-200 rounded bg-blue-50/10 focus:outline-none focus:ring-1 focus:ring-primary h-[300px] resize-none custom-scrollbar"
                        ></textarea>
                        <div class="flex justify-end gap-2 mt-2">
                             <button 
                                @click="cancelEdit(record)"
                                class="text-xs px-3 py-1 text-slate-500 hover:bg-slate-100 rounded"
                            >取消</button>
                            <button 
                                @click="saveEdit(record)"
                                class="text-xs px-3 py-1 bg-primary text-white rounded hover:bg-primary/90 flex items-center gap-1"
                            >
                                <span class="material-symbols-outlined text-xs">save</span> 保存
                            </button>
                        </div>
                    </div>

                </div>
             </div>
             
             <div v-if="historyMoreLoading" class="py-2 text-center text-slate-400 text-xs">
                 <span class="animate-pulse">加载更多...</span>
             </div>
             <div v-if="!historyHasMore && historyRecords.length > 0" class="py-4 text-center text-slate-300 text-[10px]">
                 — 已无更多记录 —
             </div>
        </div>
    </section>

    <!-- Right Column: Chat -->
    <section class="flex-1 flex flex-col bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden relative">
        <!-- Header -->
        <div class="p-4 border-b border-slate-100 flex justify-between items-center bg-white z-10 shadow-sm/50">
             <h3 class="text-lg font-bold text-slate-800 flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="material-symbols-outlined text-xl">smart_toy</span>
                </div>
                AI 问答助手
            </h3>
            <div class="flex gap-2">
                <el-tag v-if="isGenerating" type="success" effect="plain" class="animate-pulse">正在思考...</el-tag>
                <button 
                    @click="clearHistory"
                    class="text-slate-400 hover:text-red-500 text-xs flex items-center gap-1 transition-colors px-2 py-1 rounded hover:bg-red-50"
                    title="清空对话记录"
                >
                    <span class="material-symbols-outlined text-sm">delete_sweep</span>
                    清空
                </button>
            </div>
        </div>

        <!-- Chat Area -->
        <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-6 bg-[#f8fafc] custom-scrollbar scroll-smooth">
            <div v-if="chatMessages.length === 0" class="h-full flex flex-col items-center justify-center text-slate-400 opacity-60">
                <span class="material-symbols-outlined text-6xl mb-4 text-slate-300">forum</span>
                <p>有什么可以帮您？</p>
                <div class="flex gap-2 mt-6">
                    <button @click="setInput('如何改善睡眠质量？')" class="px-3 py-1 bg-white border border-slate-200 rounded-full text-xs hover:border-primary hover:text-primary transition-colors">如何改善睡眠质量？</button>
                    <button @click="setInput('解读一下高血压饮食')" class="px-3 py-1 bg-white border border-slate-200 rounded-full text-xs hover:border-primary hover:text-primary transition-colors">解读高血压饮食</button>
                </div>
            </div>

            <div v-for="msg in chatMessages" :key="msg.id" class="flex flex-col gap-1 anim-fade-in">
                <!-- User Message -->
                <div v-if="msg.role === 'user'" class="flex justify-end">
                    <div class="max-w-[85%] flex flex-col items-end gap-1">
                        <div class="bg-blue-100 text-slate-800 p-3 rounded-2xl rounded-tr-sm shadow-sm border border-blue-200 text-sm leading-relaxed group relative transition-all">
                             <div 
                                class="whitespace-pre-wrap transition-all duration-300"
                                :class="{'max-h-[120px] overflow-hidden mask-gradient': msg.isCollapsed}"
                             >{{ msg.content }}</div>
                             
                             <div v-if="isLongContent(msg.content)" class="flex justify-center mt-2 group-hover:opacity-100 opacity-60 transition-opacity">
                                <button 
                                    @click="msg.isCollapsed = !msg.isCollapsed"
                                    class="text-[10px] flex items-center gap-1 text-slate-500 hover:text-primary bg-white/50 px-2 py-0.5 rounded-full border border-blue-200 hover:border-blue-300 transition-colors"
                                >
                                    <span class="material-symbols-outlined text-[10px]">{{ msg.isCollapsed ? 'keyboard_arrow_down' : 'keyboard_arrow_up' }}</span>
                                    {{ msg.isCollapsed ? '展开' : '收起' }}
                                </button>
                             </div>
                        </div>
                        <span class="text-[10px] text-slate-300 mr-1">{{ formatTime(msg.created_at) }}</span>
                    </div>
                </div>

                <!-- AI Message -->
                <div v-else class="flex justify-start gap-3 max-w-[90%]">
                    <div class="w-8 h-8 rounded-full bg-white border border-slate-200 flex items-center justify-center shrink-0 shadow-sm mt-1">
                         <span class="material-symbols-outlined text-primary text-lg">smart_toy</span>
                    </div>
                    <div class="flex flex-col gap-1 flex-1 min-w-0">
                         <div class="bg-white p-4 rounded-2xl rounded-tl-sm border border-slate-200 shadow-sm text-slate-700 text-sm leading-relaxed group relative transition-all">
                            <!-- Typing Indicator for empty content when loading -->
                            <div v-if="!msg.content && isGenerating && msg.id === currentGeneratingId" class="flex gap-1 py-1 px-1">
                                <span class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 0s"></span>
                                <span class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></span>
                                <span class="w-1.5 h-1.5 bg-slate-400 rounded-full animate-bounce" style="animation-delay: 0.4s"></span>
                            </div>
                            
                            <!-- Collapsible Content -->
                            <div v-else>
                                 <div 
                                    class="markdown-body overflow-hidden transition-all duration-300"
                                    :class="{'max-h-[120px] mask-gradient': msg.isCollapsed}"
                                     v-html="renderMarkdown(msg.content)"
                                 ></div>
                                 <!-- Toggle Button -->
                                 <div v-if="isLongContent(msg.content)" class="flex justify-center mt-2 group-hover:opacity-100 opacity-60 transition-opacity">
                                    <button 
                                        @click="msg.isCollapsed = !msg.isCollapsed"
                                        class="text-[10px] flex items-center gap-1 text-slate-400 hover:text-primary bg-slate-50 px-3 py-1 rounded-full border border-slate-100 hover:border-blue-100 transition-colors"
                                    >
                                        <span class="material-symbols-outlined text-xs">{{ msg.isCollapsed ? 'keyboard_arrow_down' : 'keyboard_arrow_up' }}</span>
                                        {{ msg.isCollapsed ? '展开剩余内容' : '收起' }}
                                    </button>
                                 </div>
                            </div>
                            
                            <!-- Actions -->
                            <div class="absolute bottom-[-10px] right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1 z-10">
                                <button 
                                    @click="copyContent(msg.content)"
                                    class="p-1 bg-white border border-slate-200 rounded-lg text-slate-400 hover:text-primary hover:border-primary transition-colors shadow-sm text-xs flex items-center gap-1"
                                >
                                    <span class="material-symbols-outlined text-xs">content_copy</span>
                                </button>
                            </div>
                         </div>
                         <span class="text-[10px] text-slate-300 ml-1">{{ formatTime(msg.created_at) }}</span>
                    </div>
                </div>
            </div>
            
            <!-- Anchor for scroll -->
            <div ref="chatBottomAnchor"></div>
        </div>

        <!-- Input Area -->
        <div class="p-4 bg-white border-t border-slate-100 z-20">
            <div class="relative bg-slate-50 border border-slate-200 rounded-xl focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary transition-all">
                <textarea 
                    v-model="inputContent"
                    @keydown.enter.exact.prevent="sendMessage"
                    ref="inputRef"
                    class="w-full bg-transparent border-none p-3 pl-4 pr-12 text-sm focus:ring-0 resize-none max-h-[150px] custom-scrollbar placeholder:text-slate-400"
                    rows="3"
                    :disabled="isGenerating"
                    placeholder="请输入您的问题... （Shift+Enter 换行）"></textarea>
                
                <div class="absolute right-2 bottom-2 flex items-center">
                    <button 
                        @click="sendMessage"
                        :disabled="!inputContent.trim() || isGenerating"
                        class="p-2 bg-blue-100 text-blue-600 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-200 transition-all shadow-sm flex items-center justify-center w-8 h-8"
                    >
                        <span class="material-symbols-outlined text-sm">send</span>
                    </button>
                </div>
            </div>
             <p class="text-[10px] text-center text-slate-300 mt-2">AI 模型可能产生不准确信息，请注意甄别。</p>
        </div>
    </section>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'

// ===== Left: History =====
const historyRecords = ref([])
const historyLoading = ref(false)
const historyMoreLoading = ref(false)
const historyPage = ref(1)
const historySize = 10
const historyTotal = ref(0) // approximation or distinct count
const historyHasMore = ref(true)

const loadHistory = async (reset = false) => {
    if (reset) {
        historyLoading.value = true
        historyPage.value = 1
        historyRecords.value = []
        historyHasMore.value = true
    } else {
        if (!historyHasMore.value || historyMoreLoading.value) return
        historyMoreLoading.value = true
    }

    try {
        const data = await invoke('get_ai_analyses_history', {
            page: historyPage.value,
            size: historySize
        })

        const mapped = data.map(item => ({
            ...item,
            isExpanded: false,
            isEditing: false,
            editContent: item.response_content
        }))

        if (data.length < historySize) {
            historyHasMore.value = false
        }

        if (reset) {
            historyRecords.value = mapped
        } else {
            historyRecords.value.push(...mapped)
        }
        
        if (data.length > 0) historyPage.value++
        historyTotal.value = historyRecords.value.length // Just showing loaded count for now

    } catch (e) {
        ElMessage.error('加载历史记录失败: ' + e)
    } finally {
        historyLoading.value = false
        historyMoreLoading.value = false
    }
}

const handleHistoryScroll = (e) => {
    const { scrollTop, clientHeight, scrollHeight } = e.target
    if (scrollHeight - scrollTop - clientHeight < 50) {
        loadHistory(false)
    }
}

const toggleExpand = (record) => {
    record.isExpanded = !record.isExpanded
    if (!record.isExpanded) {
        record.isEditing = false // Auto exit edit mode on collapse
    }
}

const startEdit = (record) => {
    record.editContent = record.response_content
    record.isEditing = true
}

const cancelEdit = (record) => {
    record.isEditing = false
    record.editContent = record.response_content
}

const saveEdit = async (record) => {
    try {
        await invoke('update_ai_analysis_content', {
            id: record.id,
            content: record.editContent
        })
        record.response_content = record.editContent
        record.isEditing = false
        ElMessage.success('保存成功')
    } catch (e) {
        ElMessage.error('保存失败: ' + e)
    }
}

const copyToInput = (text) => {
    inputContent.value = text
    if (inputRef.value) inputRef.value.focus()
}

// ===== Right: Chat =====
const chatMessages = ref([])
const inputContent = ref('')
const isGenerating = ref(false)
const currentGeneratingId = ref(null)
const chatBottomAnchor = ref(null)
const inputRef = ref(null)

const scrollToBottom = () => {
    nextTick(() => {
        chatBottomAnchor.value?.scrollIntoView({ behavior: 'smooth' })
    })
}

const loadChatHistory = async () => {
    try {
        const data = await invoke('get_chat_history', { limit: 50, offset: 0 })
        // Mapped with collapsed state, default expanded for history? or collapsed if long?
        chatMessages.value = data.map(m => ({
            ...m,
            isCollapsed: isLongContent(m.content) // default collapse long history
        })).reverse() 
        scrollToBottom()
    } catch (e) {
        console.error(e)
    }
}

const setInput = (text) => {
    inputContent.value = text
    inputRef.value?.focus()
}

const sendMessage = async () => {
    if (!inputContent.value.trim() || isGenerating.value) return
    
    const text = inputContent.value
    inputContent.value = ''
    
    // Optimistic UI for User Message
    const tempUserId = 'temp-user-' + Date.now()
    chatMessages.value.push({
        id: tempUserId,
        role: 'user',
        content: text,
        created_at: new Date().toISOString()
    })
    scrollToBottom()

    isGenerating.value = true

    try {
        const aiMsgId = await invoke('chat_with_ai', { message: text })
        currentGeneratingId.value = aiMsgId
        
        // Optimistic UI for AI Message (Empty initially)
        chatMessages.value.push({
            id: aiMsgId,
            role: 'assistant',
            content: '',
            isCollapsed: false, // Auto expand new message
            created_at: new Date().toISOString()
        })
        scrollToBottom()

    } catch (e) {
        ElMessage.error('发送失败: ' + e)
        isGenerating.value = false
    }
}

const clearHistory = async () => {
    try {
        await ElMessageBox.confirm('确定要清空所有对话记录吗？', '确认操作', {
            type: 'warning'
        })
        await invoke('clear_chat_history')
        chatMessages.value = []
        ElMessage.success('已清空')
    } catch (e) {
        // cancelled
    }
}

const copyContent = async (text) => {
    try {
        await navigator.clipboard.writeText(text)
        ElMessage.success('已复制')
    } catch (e) {
        ElMessage.error('复制失败')
    }
}

// ===== Common & Lifecycle =====
let unlisteners = []

onMounted(async () => {
    await loadHistory(true)
    await loadChatHistory()

    unlisteners.push(await listen('chat_stream_chunk', (event) => {
        const { id, content } = event.payload
        const msg = chatMessages.value.find(m => m.id === id)
        if (msg) {
            msg.content += content
            scrollToBottom()
        }
    }))

    unlisteners.push(await listen('chat_stream_done', (event) => {
        const { id } = event.payload
        if (currentGeneratingId.value === id) {
            isGenerating.value = false
            currentGeneratingId.value = null
        }
    }))

    unlisteners.push(await listen('chat_stream_error', (event) => {
         const { id, error } = event.payload
         if (currentGeneratingId.value === id) {
            isGenerating.value = false
            currentGeneratingId.value = null
            ElMessage.error('AI 响应错误: ' + error)
        }
    }))
})

onUnmounted(() => {
    unlisteners.forEach(fn => fn())
})

const formatTime = (isoString) => {
    if (!isoString) return ''
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

const renderMarkdown = (text) => {
   if (!text) return ''
   // Simple MD parser
   let html = text
     .replace(/^### (.*$)/gim, '<h3 class="font-bold text-sm mt-3 mb-1 text-slate-800">$1</h3>')
     .replace(/^## (.*$)/gim, '<h2 class="font-bold text-base mt-4 mb-2 text-slate-900 border-b pb-1">$1</h2>')
     .replace(/\*\*(.*?)\*\*/gim, '<b class="text-primary">$1</b>')
     .replace(/^- (.*$)/gim, '<li class="ml-4 list-disc">$1</li>')
     .replace(/\n\n/gim, '<p class="mb-2"></p>') // Paragraphs
     .replace(/\n/gim, '<br>')
   return html
}

const isLongContent = (text) => {
    return text && text.length > 200
}
</script>

<style scoped>
.mask-gradient {
    mask-image: linear-gradient(to bottom, black 60%, transparent 100%);
    -webkit-mask-image: linear-gradient(to bottom, black 60%, transparent 100%);
}
.custom-scrollbar::-webkit-scrollbar {
    width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: #e2e8f0;
    border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.selection-white::selection {
    background: rgba(255,255,255,0.3);
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}
.anim-fade-in {
    animation: fadeIn 0.3s ease-out forwards;
}
</style>
