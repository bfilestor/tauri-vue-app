<template>
  <div class="settings-page">
    <header class="mb-8">
      <h1 class="text-3xl font-bold text-slate-900">系统参数设置</h1>
      <p class="text-slate-500 mt-2">配置 AI 分析接口及医疗检查项目，优化您的健康数据分析体验。</p>
    </header>

    <div class="grid grid-cols-1 xl:grid-cols-2 gap-8">
      <!-- 左列：AI 接口设置 -->
      <section class="space-y-6">
        <el-card shadow="never" class="!rounded-xl !border-slate-200">
          <template #header>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <span class="material-symbols-outlined text-[#2b8cee]">auto_awesome</span>
                <span class="font-bold text-lg">AI 接口设置</span>
              </div>
              <el-tag :type="connectionStatus === 'success' ? 'success' : 'info'" size="small" round>
                {{ connectionStatus === 'success' ? '已连接' : '未测试' }}
              </el-tag>
            </div>
          </template>

          <el-form :model="aiConfig" label-position="top" class="space-y-1">
            <el-form-item label="API 接口地址 (URL)">
              <el-input v-model="aiConfig.apiUrl" placeholder="https://api.openai.com/v1/chat/completions" />
            </el-form-item>

            <el-form-item label="API Key">
              <el-input v-model="aiConfig.apiKey" :type="showApiKey ? 'text' : 'password'" placeholder="sk-...">
                <template #suffix>
                  <el-button link @click="showApiKey = !showApiKey">
                    <span class="material-symbols-outlined text-sm">{{ showApiKey ? 'visibility_off' : 'visibility' }}</span>
                  </el-button>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item label="可用模型列表">
              <div class="w-full">
                <div class="flex gap-2 mb-2 flex-wrap">
                  <el-tag v-for="(model, index) in aiConfig.models" :key="index" closable
                    @close="removeModel(index)" type="primary" class="!rounded-lg">
                    {{ model }}
                  </el-tag>
                </div>
                <div class="flex gap-2">
                  <el-input v-model="newModel" placeholder="输入模型名称" size="small" @keyup.enter="addModel" />
                  <el-button size="small" type="primary" @click="addModel">添加</el-button>
                </div>
              </div>
            </el-form-item>

            <el-form-item label="默认模型">
              <el-select v-model="aiConfig.defaultModel" placeholder="选择默认模型" class="w-full">
                <el-option v-for="model in aiConfig.models" :key="model" :label="model" :value="model" />
              </el-select>
            </el-form-item>

            <el-divider />

            <div class="flex items-center justify-between mb-4">
              <div>
                <h4 class="text-sm font-bold">SOCKS 代理设置</h4>
                <p class="text-xs text-slate-500">如需访问特定网络，请开启此项</p>
              </div>
              <el-switch v-model="aiConfig.proxyEnabled" />
            </div>

            <template v-if="aiConfig.proxyEnabled">
              <el-form-item label="代理地址">
                <el-input v-model="aiConfig.proxyUrl" placeholder="127.0.0.1:7890" />
              </el-form-item>
              <div class="grid grid-cols-2 gap-4">
                <el-form-item label="代理账号">
                  <el-input v-model="aiConfig.proxyUsername" placeholder="可选" />
                </el-form-item>
                <el-form-item label="代理密码">
                  <el-input v-model="aiConfig.proxyPassword" type="password" placeholder="可选" />
                </el-form-item>
              </div>
            </template>
          </el-form>

          <div class="flex justify-end gap-3 mt-4 pt-4 border-t border-slate-100">
            <el-button @click="testConnection" :loading="testing">测试连接</el-button>
            <el-button type="primary" @click="saveAiConfig" :loading="savingAi">保存 AI 配置</el-button>
          </div>
        </el-card>

        <!-- Prompt 模板设置 -->
        <el-card shadow="never" class="!rounded-xl !border-slate-200">
          <template #header>
            <div class="flex items-center gap-3">
              <span class="material-symbols-outlined text-[#2b8cee]">edit_note</span>
              <span class="font-bold text-lg">Prompt 模板设置</span>
            </div>
          </template>
          <el-form label-position="top">
            <el-form-item label="OCR 识别 Prompt 模板">
              <el-input v-model="ocrPrompt" type="textarea" :rows="4"
                placeholder="请识别图片中的医疗检查报告，提取所有检查指标..." />
            </el-form-item>
            <el-form-item label="AI 分析 Prompt 模板">
              <el-input v-model="aiPrompt" type="textarea" :rows="4"
                placeholder="请根据以下检查数据，分析患者的健康状况..." />
            </el-form-item>
          </el-form>
          <div class="flex justify-end mt-2">
            <el-button type="primary" @click="savePrompts" :loading="savingPrompt">保存 Prompt 模板</el-button>
          </div>
        </el-card>
      </section>

      <!-- 右列：检查项目管理 -->
      <section class="space-y-6">
        <el-card shadow="never" class="!rounded-xl !border-slate-200">
          <template #header>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <span class="material-symbols-outlined text-[#2b8cee]">medical_services</span>
                <span class="font-bold text-lg">检查项目设置</span>
              </div>
              <el-button type="primary" size="small" @click="showProjectDialog = true">
                <span class="material-symbols-outlined text-sm mr-1">add</span>新增项目
              </el-button>
            </div>
          </template>

          <el-table :data="projects" stripe style="width: 100%" empty-text="暂无检查项目，请点击新增">
            <el-table-column prop="name" label="类别名称" min-width="120">
              <template #default="{ row }">
                <div class="flex items-center gap-2">
                  <div class="w-8 h-8 rounded bg-blue-100 flex items-center justify-center">
                    <span class="material-symbols-outlined text-sm text-[#2b8cee]">biotech</span>
                  </div>
                  <span class="font-medium text-sm">{{ row.name }}</span>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="指标数量" width="100" align="center">
              <template #default="{ row }">
                <el-tag size="small" type="info">{{ row.indicatorCount || 0 }} 项</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="80" align="center">
              <template #default="{ row }">
                <el-switch v-model="row.is_active" size="small"
                  @change="toggleProject(row)" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="140" align="right">
              <template #default="{ row }">
                <el-button link type="primary" size="small" @click="openIndicators(row)">
                  <span class="material-symbols-outlined text-base">tune</span>
                </el-button>
                <el-button link type="primary" size="small" @click="editProject(row)">
                  <span class="material-symbols-outlined text-base">edit</span>
                </el-button>
                <el-button link type="danger" size="small" @click="handleDeleteProject(row)">
                  <span class="material-symbols-outlined text-base">delete</span>
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </section>
    </div>

    <!-- 数据重置与维护 -->
    <section class="mt-8">
      <el-card shadow="never" class="!rounded-xl !border-slate-200 border-red-50">
          <template #header>
            <div class="flex items-center gap-3 text-red-600">
              <span class="material-symbols-outlined">delete_forever</span>
              <span class="font-bold text-lg">数据重置与维护</span>
            </div>
          </template>
          
          <div class="grid md:grid-cols-2 gap-12 px-2">
            <div>
                <h4 class="font-bold text-slate-800 mb-2 flex items-center gap-2">
                  <span class="material-symbols-outlined text-amber-500">history</span>
                  重置检查数据
                </h4>
                <p class="text-sm text-slate-500 mb-6 leading-relaxed">
                  将删除所有历史上传的检查报告图片（仅限程序目录下的 pictures 文件夹）、OCR 识别记录、AI 分析报告以及趋势数据。
                  <br><span class="text-xs text-red-500 font-bold bg-red-50 px-2 py-0.5 rounded mt-1 inline-block">警告：此操作不可恢复！</span>
                </p>
                <el-button type="danger" plain @click="handleResetCheckupData">
                  重置检查数据
                </el-button>
            </div>

            <div>
                <h4 class="font-bold text-slate-800 mb-2 flex items-center gap-2">
                  <span class="material-symbols-outlined text-red-600">restart_alt</span>
                  重置全部数据 (恢复出厂)
                </h4>
                <p class="text-sm text-slate-500 mb-6 leading-relaxed">
                  除了清除所有检查数据外，还会删除自定义的检查项目和指标设置，将系统恢复到初始状态。
                  <br><span class="text-xs text-red-600 font-bold bg-red-100 px-2 py-0.5 rounded mt-1 inline-block">严重警告：所有数据将永久丢失！</span>
                </p>
                <el-button type="danger" @click="handleResetAllData">
                  重置全部数据 (慎用)
                </el-button>
            </div>
          </div>
      </el-card>
    </section>

    <!-- 新增/编辑项目弹窗 -->
    <el-dialog v-model="showProjectDialog" :title="editingProject ? '编辑检查项目' : '新增检查项目'" width="420px" :close-on-click-modal="false">
      <el-form :model="projectForm" label-position="top">
        <el-form-item label="项目名称" required>
          <el-input v-model="projectForm.name" placeholder="如：血常规、肝功能" />
        </el-form-item>
        <el-form-item label="项目描述">
          <el-input v-model="projectForm.description" type="textarea" :rows="3" placeholder="可选，简要描述该检查项目" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showProjectDialog = false">取消</el-button>
        <el-button type="primary" @click="saveProject" :loading="savingProject">保存</el-button>
      </template>
    </el-dialog>

    <!-- 指标管理抽屉 -->
    <el-drawer v-model="showIndicatorDrawer" :title="currentProject?.name + ' - 指标管理'" size="480px">
      <div class="mb-4 flex justify-between items-center">
        <span class="text-sm text-slate-500">管理 {{ currentProject?.name }} 下的检查指标</span>
        <el-button type="primary" size="small" @click="showIndicatorDialog = true">
          <span class="material-symbols-outlined text-sm mr-1">add</span>新增指标
        </el-button>
      </div>

      <el-table :data="indicators" stripe size="small" empty-text="暂无指标，请添加">
        <el-table-column prop="name" label="指标名称" min-width="100" />
        <el-table-column prop="unit" label="单位" width="70" />
        <el-table-column prop="reference_range" label="参考范围" width="100" />
        <el-table-column label="核心" width="60" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.is_core" type="warning" size="small">核心</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100" align="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="editIndicator(row)">编辑</el-button>
            <el-button link type="danger" size="small" @click="handleDeleteIndicator(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 新增/编辑指标弹窗 -->
      <el-dialog v-model="showIndicatorDialog" :title="editingIndicator ? '编辑指标' : '新增指标'" width="380px" append-to-body>
        <el-form :model="indicatorForm" label-position="top" size="small">
          <el-form-item label="指标名称" required>
            <el-input v-model="indicatorForm.name" placeholder="如：白细胞计数 (WBC)" />
          </el-form-item>
          <el-form-item label="单位">
            <el-input v-model="indicatorForm.unit" placeholder="如：×10⁹/L" />
          </el-form-item>
          <el-form-item label="参考范围">
            <el-input v-model="indicatorForm.reference_range" placeholder="如：3.5-9.5" />
          </el-form-item>
          <el-form-item label="标记为核心指标">
            <el-switch v-model="indicatorForm.is_core" />
            <span class="text-xs text-slate-400 ml-2">核心指标会在趋势图中默认显示</span>
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="showIndicatorDialog = false" size="small">取消</el-button>
          <el-button type="primary" @click="saveIndicator" size="small" :loading="savingIndicator">保存</el-button>
        </template>
      </el-dialog>
    </el-drawer>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

// ===== AI 配置 =====
const showApiKey = ref(false)
const connectionStatus = ref('unknown')
const testing = ref(false)
const savingAi = ref(false)
const savingPrompt = ref(false)
const newModel = ref('')

const aiConfig = reactive({
  apiUrl: '',
  apiKey: '',
  models: [],
  defaultModel: '',
  proxyEnabled: false,
  proxyUrl: '',
  proxyUsername: '',
  proxyPassword: '',
})

const ocrPrompt = ref('')
const aiPrompt = ref('')

const addModel = () => {
  const model = newModel.value.trim()
  if (model && !aiConfig.models.includes(model)) {
    aiConfig.models.push(model)
    newModel.value = ''
  }
}

const removeModel = (index) => {
  const removed = aiConfig.models.splice(index, 1)
  if (aiConfig.defaultModel === removed[0]) {
    aiConfig.defaultModel = aiConfig.models[0] || ''
  }
}

const loadAiConfig = async () => {
  try {
    const url = await invoke('get_config', { key: 'ai_api_url' })
    const key = await invoke('get_config', { key: 'ai_api_key' })
    const models = await invoke('get_config', { key: 'ai_models' })
    const defaultModel = await invoke('get_config', { key: 'ai_default_model' })
    const proxyEnabled = await invoke('get_config', { key: 'proxy_enabled' })
    const proxyUrl = await invoke('get_config', { key: 'proxy_url' })
    const proxyUsername = await invoke('get_config', { key: 'proxy_username' })
    const proxyPassword = await invoke('get_config', { key: 'proxy_password' })

    aiConfig.apiUrl = url || ''
    aiConfig.apiKey = key || ''
    aiConfig.models = models ? JSON.parse(models) : []
    aiConfig.defaultModel = defaultModel || ''
    aiConfig.proxyEnabled = proxyEnabled === 'true'
    aiConfig.proxyUrl = proxyUrl || ''
    aiConfig.proxyUsername = proxyUsername || ''
    aiConfig.proxyPassword = proxyPassword || ''

    const ocrTpl = await invoke('get_config', { key: 'ocr_prompt_template' })
    const aiTpl = await invoke('get_config', { key: 'ai_analysis_prompt_template' })
    ocrPrompt.value = ocrTpl || '请识别图片中的医疗检查报告，提取所有检查指标的名称、数值、单位和参考范围，请严格按照以下JSON格式返回: [ { "name": "指标名称", "value": "数值", "unit": "单位", "reference_range": "参考范围", "status": "正常/异常" } ] 注意：reference_range 必须是字符串，表示参考范围（如 "3.5-5.5"）。2，status必须是 "正常" 或 "异常"。3，只返回JSON数组，不要包含markdown代码块或其他文字。'
    aiPrompt.value = aiTpl || '请根据以下检查数据，综合分析患者的健康状况，指出异常指标，提供治疗建议和生活方式改善方案。'
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

const saveAiConfig = async () => {
  savingAi.value = true
  try {
    await invoke('save_config', { key: 'ai_api_url', value: aiConfig.apiUrl })
    await invoke('save_config', { key: 'ai_api_key', value: aiConfig.apiKey })
    await invoke('save_config', { key: 'ai_models', value: JSON.stringify(aiConfig.models) })
    await invoke('save_config', { key: 'ai_default_model', value: aiConfig.defaultModel })
    await invoke('save_config', { key: 'proxy_enabled', value: String(aiConfig.proxyEnabled) })
    await invoke('save_config', { key: 'proxy_url', value: aiConfig.proxyUrl })
    await invoke('save_config', { key: 'proxy_username', value: aiConfig.proxyUsername })
    await invoke('save_config', { key: 'proxy_password', value: aiConfig.proxyPassword })
    ElMessage.success('AI 配置保存成功')
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  } finally {
    savingAi.value = false
  }
}

const savePrompts = async () => {
  savingPrompt.value = true
  try {
    await invoke('save_config', { key: 'ocr_prompt_template', value: ocrPrompt.value })
    await invoke('save_config', { key: 'ai_analysis_prompt_template', value: aiPrompt.value })
    ElMessage.success('Prompt 模板保存成功')
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  } finally {
    savingPrompt.value = false
  }
}

const testConnection = async () => {
  testing.value = true
  connectionStatus.value = 'unknown'
  try {
    if (!aiConfig.apiUrl || !aiConfig.apiKey) {
      ElMessage.warning('请先填写 API 地址和 Key')
      return
    }
    // 先保存配置，确保后端能读取到最新值
    await saveAiConfig()
    // 调用后端真实测试命令
    const result = await invoke('test_ai_connection')
    connectionStatus.value = 'success'
    ElMessage.success(result)
  } catch (e) {
    connectionStatus.value = 'failed'
    ElMessage.error('连接测试失败: ' + e)
  } finally {
    testing.value = false
  }
}

// ===== 检查项目管理 =====
const projects = ref([])
const showProjectDialog = ref(false)
const editingProject = ref(null)
const savingProject = ref(false)

const projectForm = reactive({
  name: '',
  description: '',
})

const loadProjects = async () => {
  try {
    const list = await invoke('list_projects')
    // 加载每个项目的指标数量
    for (const p of list) {
      try {
        const inds = await invoke('list_indicators', { projectId: p.id })
        p.indicatorCount = inds.length
      } catch {
        p.indicatorCount = 0
      }
    }
    projects.value = list
  } catch (e) {
    ElMessage.error('加载项目列表失败: ' + e)
  }
}

const editProject = (row) => {
  editingProject.value = row
  projectForm.name = row.name
  projectForm.description = row.description
  showProjectDialog.value = true
}

const saveProject = async () => {
  if (!projectForm.name.trim()) {
    ElMessage.warning('请输入项目名称')
    return
  }
  savingProject.value = true
  try {
    if (editingProject.value) {
      await invoke('update_project', {
        input: {
          id: editingProject.value.id,
          name: projectForm.name,
          description: projectForm.description,
        }
      })
      ElMessage.success('项目更新成功')
    } else {
      await invoke('create_project', {
        input: {
          name: projectForm.name,
          description: projectForm.description,
        }
      })
      ElMessage.success('项目创建成功')
    }
    showProjectDialog.value = false
    editingProject.value = null
    projectForm.name = ''
    projectForm.description = ''
    await loadProjects()
  } catch (e) {
    ElMessage.error('' + e)
  } finally {
    savingProject.value = false
  }
}

const toggleProject = async (row) => {
  try {
    await invoke('update_project', {
      input: { id: row.id, is_active: row.is_active }
    })
    ElMessage.success(row.is_active ? '已启用' : '已停用')
  } catch (e) {
    ElMessage.error('' + e)
    row.is_active = !row.is_active
  }
}

const handleDeleteProject = async (row) => {
  try {
    await ElMessageBox.confirm(`确定要删除检查项目「${row.name}」吗？`, '确认删除', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
    await invoke('delete_project', { id: row.id })
    ElMessage.success('项目已删除')
    await loadProjects()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('' + e)
  }
}

// ===== 指标管理 =====
const showIndicatorDrawer = ref(false)
const showIndicatorDialog = ref(false)
const currentProject = ref(null)
const indicators = ref([])
const editingIndicator = ref(null)
const savingIndicator = ref(false)

const indicatorForm = reactive({
  name: '',
  unit: '',
  reference_range: '',
  is_core: false,
})

const openIndicators = async (project) => {
  currentProject.value = project
  showIndicatorDrawer.value = true
  await loadIndicators()
}

const loadIndicators = async () => {
  if (!currentProject.value) return
  try {
    indicators.value = await invoke('list_indicators', { projectId: currentProject.value.id })
  } catch (e) {
    ElMessage.error('加载指标失败: ' + e)
  }
}

const editIndicator = (row) => {
  editingIndicator.value = row
  indicatorForm.name = row.name
  indicatorForm.unit = row.unit
  indicatorForm.reference_range = row.reference_range
  indicatorForm.is_core = row.is_core
  showIndicatorDialog.value = true
}

const saveIndicator = async () => {
  if (!indicatorForm.name.trim()) {
    ElMessage.warning('请输入指标名称')
    return
  }
  savingIndicator.value = true
  try {
    if (editingIndicator.value) {
      await invoke('update_indicator', {
        input: {
          id: editingIndicator.value.id,
          name: indicatorForm.name,
          unit: indicatorForm.unit,
          reference_range: indicatorForm.reference_range,
          is_core: indicatorForm.is_core,
        }
      })
      ElMessage.success('指标更新成功')
    } else {
      await invoke('create_indicator', {
        input: {
          project_id: currentProject.value.id,
          name: indicatorForm.name,
          unit: indicatorForm.unit,
          reference_range: indicatorForm.reference_range,
          is_core: indicatorForm.is_core,
        }
      })
      ElMessage.success('指标创建成功')
    }
    showIndicatorDialog.value = false
    editingIndicator.value = null
    Object.assign(indicatorForm, { name: '', unit: '', reference_range: '', is_core: false })
    await loadIndicators()
    await loadProjects()
  } catch (e) {
    ElMessage.error('' + e)
  } finally {
    savingIndicator.value = false
  }
}

const handleDeleteIndicator = async (row) => {
  try {
    await ElMessageBox.confirm(`确定要删除指标「${row.name}」吗？`, '确认删除', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
    await invoke('delete_indicator', { id: row.id })
    ElMessage.success('指标已删除')
    await loadIndicators()
    await loadProjects()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('' + e)
  }
}

// ===== 数据重置 =====
const handleResetCheckupData = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有检查记录、上传的图片和分析数据吗？\n操作将删除 pictures 文件夹下的所有内容，且不可恢复！',
      '确认重置检查数据',
      {
        confirmButtonText: '确定重置',
        cancelButtonText: '取消',
        type: 'warning',
        confirmButtonClass: 'el-button--danger'
      }
    )
    
    await invoke('reset_checkup_data')
    ElMessage.success('检查数据已成功重置')
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('重置失败: ' + e)
  }
}

const handleResetAllData = async () => {
    try {
        const { value } = await ElMessageBox.prompt(
            '此操作将删除所有数据（包括检查记录和项目设置、指标定义），系统将恢复到初始状态。\n\n如果要继续，请输入 "RESET" 以确认操作：',
            '严重警告：重置全部数据',
            {
                confirmButtonText: '确认重置所有',
                cancelButtonText: '取消',
                type: 'error',
                inputPattern: /^RESET$/,
                inputErrorMessage: '输入内容不正确，请输入 RESET',
                confirmButtonClass: 'el-button--danger'
            }
        )
        
        await invoke('reset_all_data')
        ElMessage.success('系统已成功重置为初始状态')
        // 刷新项目列表
        await loadProjects()
    } catch (e) {
        if (e !== 'cancel') ElMessage.warning('操作已取消')
    }
}

// ===== 初始化 =====
onMounted(() => {
  loadAiConfig()
  loadProjects()
})
</script>
