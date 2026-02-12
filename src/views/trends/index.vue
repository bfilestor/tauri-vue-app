<template>
  <div class="trends-page">
    <header class="mb-6">
      <h1 class="text-3xl font-bold text-slate-900">趋势分析</h1>
      <p class="text-slate-500 mt-1">通过可视化图表追踪您的健康指标变化趋势，及时发现潜在风险。</p>
    </header>

    <!-- 项目选择 -->
    <div class="flex items-center gap-4 mb-6">
      <el-select v-model="selectedProjectId" placeholder="选择检查项目" clearable @change="loadTrends" class="w-56">
        <el-option v-for="p in projects" :key="p.id" :label="p.name" :value="p.id" />
      </el-select>
      <el-button type="primary" @click="loadAllTrends" :loading="loading">
        <span class="material-symbols-outlined text-sm mr-1">show_chart</span>
        查看全部项目
      </el-button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="text-center py-16">
      <el-icon class="is-loading text-4xl text-blue-400"><Loading /></el-icon>
      <p class="text-slate-400 mt-4">加载趋势数据...</p>
    </div>

    <!-- 空状态 -->
    <div v-else-if="trendData.length === 0" class="text-center py-20">
      <span class="material-symbols-outlined text-6xl text-slate-300 block mb-4">monitoring</span>
      <p class="text-slate-400 text-lg mb-2">暂无趋势数据</p>
      <p class="text-slate-400 text-sm">请先上传检查报告并完成 OCR 识别，系统将自动生成趋势图表</p>
    </div>

    <!-- 趋势图表区域 -->
    <div v-else class="space-y-8">
      <div v-for="project in trendData" :key="project.project_id" class="bg-white rounded-xl border border-slate-200 overflow-hidden">
        <!-- 项目标题 -->
        <div class="px-6 py-4 border-b border-slate-100 bg-slate-50/50">
          <h2 class="text-lg font-bold text-slate-800">
            <span class="material-symbols-outlined text-base align-middle mr-2 text-blue-500">science</span>
            {{ project.project_name }}
          </h2>
          <p class="text-xs text-slate-400 mt-1">共 {{ project.indicators.length }} 个指标</p>
        </div>

        <!-- 指标卡片网格 -->
        <div class="p-6">
          <!-- 无数据时 -->
          <div v-if="project.indicators.every(i => i.data_points.length === 0)" class="text-center py-8 text-slate-400 text-sm">
            该项目暂无历史检测数据
          </div>

          <!-- 指标网格 -->
          <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div v-for="indicator in project.indicators.filter(i => i.data_points.length > 0)"
              :key="indicator.indicator_id"
              class="border border-slate-200 rounded-xl p-5 hover:shadow-md transition-shadow">

              <!-- 指标头 -->
              <div class="flex items-center justify-between mb-4">
                <div>
                  <h3 class="font-semibold text-slate-800">{{ indicator.indicator_name }}</h3>
                  <span v-if="indicator.unit" class="text-xs text-slate-400">单位: {{ indicator.unit }}</span>
                  <span v-if="indicator.reference_range" class="text-xs text-slate-400 ml-3">
                    参考: {{ indicator.reference_range }}
                  </span>
                </div>
                <div v-if="indicator.data_points.length > 0" class="text-right">
                  <div class="text-xl font-bold" :class="latestPointClass(indicator)">
                    {{ latestValue(indicator) }}
                  </div>
                  <div class="text-xs text-slate-400">最新值</div>
                </div>
              </div>

              <!-- ECharts 图表 -->
              <div :ref="el => setChartRef(indicator.indicator_id, el)" style="height: 200px" class="w-full"></div>

              <!-- 数据点列表 -->
              <div class="mt-3 pt-3 border-t border-slate-100">
                <div class="flex flex-wrap gap-2">
                  <div v-for="dp in indicator.data_points" :key="dp.checkup_date"
                    class="text-xs px-2 py-1 rounded"
                    :class="dp.is_abnormal ? 'bg-red-50 text-red-600' : 'bg-slate-100 text-slate-600'">
                    {{ dp.checkup_date }}: {{ dp.value_text || dp.value }}
                    <span v-if="dp.is_abnormal" class="ml-1">⚠️</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
  LegendComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([LineChart, GridComponent, TooltipComponent, MarkLineComponent, LegendComponent, CanvasRenderer])

// 项目列表
const projects = ref([])
const selectedProjectId = ref('')
const loading = ref(false)
const trendData = ref([])

// 图表实例管理
const chartRefs = {}
const chartInstances = {}

const setChartRef = (id, el) => {
  if (el) {
    chartRefs[id] = el
  }
}

const loadProjects = async () => {
  try {
    const all = await invoke('list_projects')
    projects.value = all.filter(p => p.is_active)
  } catch (e) {
    console.error('加载项目失败:', e)
  }
}

const loadTrends = async () => {
  if (!selectedProjectId.value) return
  loading.value = true
  try {
    const data = await invoke('get_project_trends', { projectId: selectedProjectId.value })
    trendData.value = [data]
    await nextTick()
    renderCharts()
  } catch (e) {
    ElMessage.error('加载趋势数据失败: ' + e)
  } finally {
    loading.value = false
  }
}

const loadAllTrends = async () => {
  loading.value = true
  selectedProjectId.value = ''
  try {
    const data = await invoke('get_all_trends')
    trendData.value = data
    await nextTick()
    renderCharts()
  } catch (e) {
    ElMessage.error('加载趋势数据失败: ' + e)
  } finally {
    loading.value = false
  }
}

const renderCharts = () => {
  // 销毁旧实例
  Object.values(chartInstances).forEach(inst => inst.dispose())

  for (const project of trendData.value) {
    for (const indicator of project.indicators) {
      if (indicator.data_points.length === 0) continue

      const el = chartRefs[indicator.indicator_id]
      if (!el) continue

      const chart = echarts.init(el)
      chartInstances[indicator.indicator_id] = chart

      const dates = indicator.data_points.map(dp => dp.checkup_date)
      const values = indicator.data_points.map(dp => dp.value)
      const abnormals = indicator.data_points.map(dp => dp.is_abnormal)

      // 解析参考范围
      const refRange = parseReferenceRange(indicator.reference_range)

      const option = {
        tooltip: {
          trigger: 'axis',
          formatter: (params) => {
            const p = params[0]
            const dp = indicator.data_points[p.dataIndex]
            let html = `<div class="text-sm"><b>${p.name}</b><br/>`
            html += `${indicator.indicator_name}: <b>${dp.value_text || dp.value}</b> ${indicator.unit}<br/>`
            if (dp.is_abnormal) html += `<span style="color: #ef4444">⚠️ 异常</span>`
            html += '</div>'
            return html
          }
        },
        grid: {
          left: 50,
          right: 20,
          top: 20,
          bottom: 30,
        },
        xAxis: {
          type: 'category',
          data: dates,
          axisLabel: {
            fontSize: 10,
            color: '#94a3b8',
            rotate: dates.length > 5 ? 30 : 0,
          },
          axisLine: { lineStyle: { color: '#e2e8f0' } },
        },
        yAxis: {
          type: 'value',
          axisLabel: { fontSize: 10, color: '#94a3b8' },
          splitLine: { lineStyle: { color: '#f1f5f9' } },
        },
        series: [{
          type: 'line',
          data: values,
          smooth: true,
          symbol: 'circle',
          symbolSize: 8,
          lineStyle: {
            width: 3,
            color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
              { offset: 0, color: '#3b82f6' },
              { offset: 1, color: '#06b6d4' },
            ]),
          },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(59,130,246,0.15)' },
              { offset: 1, color: 'rgba(59,130,246,0)' },
            ]),
          },
          itemStyle: {
            color: (params) => {
              return abnormals[params.dataIndex] ? '#ef4444' : '#3b82f6'
            },
            borderWidth: 2,
            borderColor: '#fff',
          },
          markLine: refRange ? {
            silent: true,
            symbol: 'none',
            lineStyle: { type: 'dashed', width: 1 },
            data: [
              ...(refRange.min !== null ? [{
                yAxis: refRange.min,
                lineStyle: { color: '#22c55e' },
                label: { formatter: `下限 ${refRange.min}`, fontSize: 10 },
              }] : []),
              ...(refRange.max !== null ? [{
                yAxis: refRange.max,
                lineStyle: { color: '#f59e0b' },
                label: { formatter: `上限 ${refRange.max}`, fontSize: 10 },
              }] : []),
            ]
          } : undefined,
        }],
      }

      chart.setOption(option)
    }
  }
}

// 解析参考范围字符串
const parseReferenceRange = (rangeStr) => {
  if (!rangeStr) return null

  // 匹配 "3.5-5.5" 或 "3.5~5.5" 格式
  const m = rangeStr.match(/([\d.]+)\s*[-~]\s*([\d.]+)/)
  if (m) {
    return { min: parseFloat(m[1]), max: parseFloat(m[2]) }
  }

  // 匹配 "<5.5" 格式
  const lt = rangeStr.match(/[<≤]\s*([\d.]+)/)
  if (lt) return { min: null, max: parseFloat(lt[1]) }

  // 匹配 ">3.5" 格式
  const gt = rangeStr.match(/[>≥]\s*([\d.]+)/)
  if (gt) return { min: parseFloat(gt[1]), max: null }

  return null
}

// 辅助函数
const latestValue = (indicator) => {
  if (indicator.data_points.length === 0) return '-'
  const latest = indicator.data_points[indicator.data_points.length - 1]
  return latest.value_text || latest.value || '-'
}

const latestPointClass = (indicator) => {
  if (indicator.data_points.length === 0) return 'text-slate-400'
  const latest = indicator.data_points[indicator.data_points.length - 1]
  return latest.is_abnormal ? 'text-red-500' : 'text-emerald-500'
}

// 响应窗口大小变化
const handleResize = () => {
  Object.values(chartInstances).forEach(inst => inst.resize())
}

onMounted(() => {
  loadProjects()
  loadAllTrends()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  Object.values(chartInstances).forEach(inst => inst.dispose())
  window.removeEventListener('resize', handleResize)
})
</script>
