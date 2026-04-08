<template>
  <div class="settings-page">
    <header class="mb-8">
      <h1 class="text-3xl font-bold text-slate-900">系统参数设置</h1>
      <p class="text-slate-500 mt-2">配置 AI 分析接口及医疗检查项目，优化您的健康数据分析体验。</p>
    </header>

    <el-tabs v-model="activeTab" class="w-full">
      <!-- ========== 模型服务 Tab (ISS-060~066) ========== -->
      <el-tab-pane label="模型服务" name="ai">
        <el-card shadow="never" class="!rounded-xl !border-slate-200 mb-6">
          <template #header>
            <div class="flex items-center gap-2 font-bold text-slate-700 text-sm">
              <span class="material-symbols-outlined text-blue-500 text-[18px]">switch_access_shortcut</span>
              模式切换
            </div>
          </template>
          <div class="space-y-4">
            <el-radio-group v-model="activeAiMode" size="large">
              <el-radio-button :label="AI_MODES.general">通用模式</el-radio-button>
              <el-radio-button :label="AI_MODES.custom">自定义模式</el-radio-button>
            </el-radio-group>
            <p v-if="isGeneralMode" class="text-sm text-slate-500">
              使用平台内置模型服务，按调用次数计费。可在本页直接购买套餐并查看剩余次数。
            </p>
            <p v-else class="text-sm text-slate-500">
              使用你自己的 Provider 与模型配置。现有提供商、模型和网络设置能力保持不变。
            </p>
          </div>
        </el-card>

        <template v-if="isGeneralMode">
          <el-card shadow="never" class="!rounded-xl !border-slate-200 mb-6">
            <template #header>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2 font-bold text-slate-700 text-sm">
                  <span class="material-symbols-outlined text-blue-500 text-[18px]">credit_score</span>
                  通用模式次数概览
                </div>
                <el-button size="small" plain :loading="refreshingAccountContext" @click="handleRefreshAccountContext">
                  刷新余额
                </el-button>
              </div>
            </template>
            <div class="space-y-4">
              <div class="rounded-xl bg-slate-50 border border-slate-100 p-4">
                <div class="flex items-center justify-between text-sm text-slate-600">
                  <span>当前剩余次数</span>
                  <span class="font-bold text-blue-600">{{ accountUsage.label }}</span>
                </div>
                <div class="mt-2 h-2 w-full rounded-full bg-slate-200 overflow-hidden">
                  <div class="h-full bg-gradient-to-r from-blue-500 to-cyan-400 rounded-full" :style="{ width: `${accountUsage.percent}%` }"></div>
                </div>
                <p v-if="accountUsage.stale" class="mt-2 text-xs text-amber-600">余额接口异常，当前显示缓存数据</p>
              </div>
              <el-alert
                v-if="accountContextState.memberBlocked"
                type="warning"
                :closable="false"
                :title="accountContextState.memberBlockedReason || '未找到默认成员，请先在账户中心设置默认成员'"
              />
              <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                <div
                  v-for="card in accountPackageCards"
                  :key="card.targetCalls"
                  class="rounded-xl border p-3"
                  :class="card.missing ? 'border-slate-200 bg-slate-50' : 'border-blue-200 bg-blue-50/40'"
                >
                  <p class="text-sm font-bold text-slate-800">{{ card.title }}</p>
                  <p class="text-xs mt-1" :class="card.missing ? 'text-slate-400' : 'text-slate-600'">
                    {{ card.missing ? '该档套餐暂不可用' : `SKU: ${card.product?.skuId || '-'}` }}
                  </p>
                  <p class="text-xs mt-1" :class="card.missing ? 'text-slate-400' : 'text-blue-600'">
                    {{ card.missing ? '请稍后刷新商品列表' : `价格: ¥${card.product?.price ?? '-'}` }}
                  </p>
                  <el-button
                    size="small"
                    class="!mt-2 !w-full"
                    :disabled="card.missing || !card.purchasable"
                    @click="handleOpenPurchaseDialog(card.targetCalls)"
                  >
                    购买此套餐
                  </el-button>
                </div>
              </div>
            </div>
          </el-card>
        </template>

        <template v-else>
          <el-alert
            class="mb-4"
            type="info"
            :closable="false"
            :title="accountContextState.isGuest ? '当前为访客态，建议登录后再配置自定义模型服务。' : '当前为自定义模式，所有模型调用将复用你现有的 Provider 与模型配置。'"
          />
          <div class="mb-4 rounded-xl border border-slate-200 bg-slate-50/70 p-4 flex flex-col md:flex-row md:items-center md:justify-between gap-3">
            <div>
              <p class="text-sm font-bold text-slate-700">还没有 API Key？先看接入说明</p>
              <p class="text-xs text-slate-500 mt-1">内含平台类型、获取 API Key 步骤和注册链接占位（未配置会明确提示）。</p>
            </div>
            <el-button plain size="small" @click="openCustomModeGuide">
              <span class="material-symbols-outlined text-sm mr-1">help</span>查看接入说明
            </el-button>
          </div>
          <div class="flex h-[620px] border border-slate-200 rounded-xl overflow-hidden bg-white shadow-sm">
            <!-- 左侧: 提供商列表 -->
            <div class="w-60 border-r border-slate-100 bg-slate-50/80 flex flex-col shrink-0">
              <div class="p-3 border-b border-slate-100 bg-white">
                <el-input v-model="searchProvider" placeholder="搜索提供商..." clearable size="small">
                  <template #prefix><span class="material-symbols-outlined text-sm text-slate-400">search</span></template>
                </el-input>
              </div>
              <div class="flex-1 overflow-y-auto p-2 space-y-1">
                <div v-for="p in filteredProviders" :key="p.id"
                     class="flex items-center justify-between px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150"
                     :class="activeProviderId === p.id
                       ? 'bg-white shadow-sm border border-blue-200 ring-1 ring-blue-100'
                       : 'hover:bg-white/70 border border-transparent'"
                     @click="selectProvider(p.id)">
                  <div class="flex items-center gap-2.5 overflow-hidden min-w-0">
                    <div class="w-7 h-7 shrink-0 rounded-full flex items-center justify-center text-xs font-bold text-white shadow-inner"
                         :style="{ background: getProviderColor(p.type) }">
                      {{ p.name.charAt(0).toUpperCase() }}
                    </div>
                    <span class="text-sm font-medium text-slate-700 truncate">{{ p.name }}</span>
                  </div>
                  <el-switch v-model="p.enabled" size="small" @click.stop @change="toggleProviderEnabled(p)" class="shrink-0 ml-1" />
                </div>
                <div v-if="filteredProviders.length === 0" class="text-center text-slate-400 mt-10 text-xs">暂无匹配</div>
              </div>
              <div class="p-3 border-t border-slate-100 bg-white">
                <el-button class="w-full" plain size="small" @click="openAddProviderDialog">
                  <span class="material-symbols-outlined text-sm mr-1">add</span>添加提供商
                </el-button>
              </div>
            </div>

            <!-- 右侧: 提供商配置 + 模型列表 -->
            <div v-if="activeProvider" class="flex-1 flex flex-col min-w-0 overflow-y-auto">
              <div class="px-6 py-4 border-b border-slate-100 flex justify-between items-center sticky top-0 bg-white/95 backdrop-blur z-10">
                <h2 class="text-lg font-bold text-slate-800 flex items-center gap-2">
                  <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold text-white" :style="{ background: getProviderColor(activeProvider.type) }">
                    {{ activeProvider.name.charAt(0).toUpperCase() }}
                  </div>
                  {{ activeProvider.name }}
                  <el-tag size="small" type="info" effect="plain" class="!ml-2">{{ activeProvider.type }}</el-tag>
                </h2>
                <div class="flex items-center gap-2">
                  <el-button link size="small" @click="editProvider(activeProvider)">
                    <span class="material-symbols-outlined text-slate-400 text-[18px] hover:text-blue-500 transition-colors">settings</span>
                  </el-button>
                  <el-button link type="danger" size="small" @click="delProvider(activeProvider.id)">
                    <span class="material-symbols-outlined text-[18px]">delete</span>
                  </el-button>
                </div>
              </div>
              <div class="p-6 space-y-6 flex-1">
                <div class="bg-slate-50/60 p-5 rounded-xl border border-slate-100 space-y-4">
                  <div>
                    <div class="flex justify-between items-center mb-1.5">
                      <label class="text-sm font-bold text-slate-700">API 密钥</label>
                      <el-button type="primary" plain size="small" class="!px-3 !h-7" @click="testProviderConnection(activeProvider)" :loading="testingProviderId === activeProvider.id">
                        <span class="material-symbols-outlined text-xs mr-1">wifi_tethering</span>检测
                      </el-button>
                    </div>
                    <el-input v-model="activeProvider.api_key" type="password" show-password placeholder="sk-..." @change="saveProviderField(activeProvider)" />
                  </div>
                  <div>
                    <label class="text-sm font-bold text-slate-700 mb-1.5 block">API 地址</label>
                    <el-input v-model="activeProvider.api_url" placeholder="https://api.openai.com/v1/chat/completions" @change="saveProviderField(activeProvider)" />
                  </div>
                </div>
                <div>
                  <div class="flex justify-between items-center mb-3">
                    <h3 class="font-bold text-slate-800 flex items-center gap-2">模型 <el-tag size="small" type="primary" round effect="plain">{{ providerModels.length }}</el-tag></h3>
                    <el-button size="small" type="primary" plain @click="openAddModelDialog"><span class="material-symbols-outlined text-sm mr-1">add</span>添加</el-button>
                  </div>
                  <div v-if="providerModels.length === 0" class="text-center py-8 bg-slate-50 rounded-xl border border-dashed border-slate-200">
                    <span class="material-symbols-outlined text-3xl text-slate-300 mb-2">widgets</span>
                    <p class="text-sm text-slate-400">尚未添加模型</p>
                  </div>
                  <div v-else class="space-y-3">
                    <div v-for="(models, groupName) in groupedModels" :key="groupName" class="border border-slate-200 rounded-xl overflow-hidden">
                      <div class="bg-slate-100/80 px-4 py-2 text-xs font-bold text-slate-500 border-b border-slate-200 flex items-center gap-1.5">
                        <span class="material-symbols-outlined text-sm text-slate-400">folder_open</span>{{ groupName || '未分组' }}
                      </div>
                      <div class="divide-y divide-slate-50">
                        <div v-for="m in models" :key="m.id" class="px-4 py-2.5 flex justify-between items-center hover:bg-blue-50/30 group transition-colors">
                          <div class="flex items-center gap-3 min-w-0">
                            <div class="w-7 h-7 rounded-lg bg-white border border-slate-200 flex items-center justify-center shadow-sm shrink-0">
                              <span class="material-symbols-outlined text-slate-500 text-[16px]">smart_toy</span>
                            </div>
                            <div class="flex flex-col min-w-0">
                              <div class="flex items-center gap-2">
                                <span class="text-sm font-medium text-slate-800 truncate">{{ m.model_name || m.model_id }}</span>
                                <el-tag v-if="m.is_default" size="small" type="success" effect="dark" round class="!text-[10px] !h-4 !leading-4">默认</el-tag>
                              </div>
                              <span v-if="m.model_name && m.model_name !== m.model_id" class="text-xs text-slate-400 font-mono truncate">{{ m.model_id }}</span>
                            </div>
                          </div>
                          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                            <el-button v-if="!m.is_default" size="small" round class="!px-2 !h-6 !text-xs" @click="handleSetDefault(m)">设默认</el-button>
                            <el-button link size="small" @click="editModel(m)"><span class="material-symbols-outlined text-[16px] text-slate-400 hover:text-blue-500">edit</span></el-button>
                            <el-button link type="danger" size="small" @click="handleDeleteModel(m)"><span class="material-symbols-outlined text-[16px]">remove</span></el-button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="flex-1 flex flex-col items-center justify-center text-slate-400 bg-slate-50/50">
              <div class="w-20 h-20 mb-4 rounded-full bg-slate-100 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-slate-300">hub</span>
              </div>
              <p class="text-sm text-slate-500 font-medium">请在左侧选择或添加提供商</p>
            </div>
          </div>
          <!-- 全局网络设置 -->
          <el-card shadow="never" class="!rounded-xl !border-slate-200 mt-6">
            <template #header>
              <div class="flex items-center gap-2 font-bold text-slate-700 text-sm">
                <span class="material-symbols-outlined text-blue-500 text-[18px]">router</span>全局网络设置
              </div>
            </template>
            <el-form :model="networkConfig" label-position="top" size="small" class="grid grid-cols-1 md:grid-cols-2 gap-x-6">
              <div class="col-span-full flex items-center justify-between mb-3 bg-slate-50 p-2.5 rounded-lg border border-slate-100">
                <span class="text-sm font-medium text-slate-700">启用 SOCKS 代理</span>
                <el-switch v-model="networkConfig.proxyEnabled" @change="saveNetworkConfig" />
              </div>
              <template v-if="networkConfig.proxyEnabled">
                <el-form-item label="代理地址"><el-input v-model="networkConfig.proxyUrl" placeholder="127.0.0.1:7890" @change="saveNetworkConfig" /></el-form-item>
                <el-form-item label="请求超时 (秒)"><el-input-number v-model="networkConfig.timeout" :min="10" :max="600" controls-position="right" class="!w-full" @change="saveNetworkConfig" /></el-form-item>
                <el-form-item label="代理账号"><el-input v-model="networkConfig.proxyUsername" @change="saveNetworkConfig" /></el-form-item>
                <el-form-item label="代理密码"><el-input v-model="networkConfig.proxyPassword" type="password" @change="saveNetworkConfig" /></el-form-item>
              </template>
              <template v-else>
                <el-form-item label="请求超时 (秒)"><el-input-number v-model="networkConfig.timeout" :min="10" :max="600" controls-position="right" class="!w-full" @change="saveNetworkConfig" /></el-form-item>
              </template>
            </el-form>
          </el-card>
        </template>
      </el-tab-pane>

      <el-tab-pane label="家庭成员" name="members">
        <el-card shadow="never" class="!rounded-xl !border-slate-200 mb-6">
          <template #header>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2 font-bold text-slate-700 text-sm">
                <span class="material-symbols-outlined text-blue-500 text-[18px]">groups</span>
                家庭成员管理
              </div>
              <el-button
                type="primary"
                size="small"
                :disabled="accountContextState.isGuest || !accountContextState.profile"
                @click="openCreateMemberDialog"
              >
                <span class="material-symbols-outlined text-sm mr-1">person_add</span>
                新增成员
              </el-button>
            </div>
          </template>

          <div v-if="accountContextState.isGuest || !accountContextState.profile" class="space-y-3">
            <el-alert
              type="warning"
              :closable="false"
              title="当前为访客态，登录后才可管理家庭成员。"
            />
          </div>

          <div v-else class="space-y-4">
            <div class="rounded-xl border border-slate-100 bg-slate-50 p-4">
              <div class="flex flex-wrap items-center gap-3 text-sm text-slate-600">
                <span>当前成员：<strong class="text-slate-800">{{ accountContextState.currentMember?.memberName || '未选择' }}</strong></span>
                <span>默认成员：<strong class="text-slate-800">{{ accountContextState.defaultMember?.memberName || '未设置' }}</strong></span>
              </div>
            </div>

            <el-table :data="accountContextState.members" border stripe>
              <el-table-column prop="memberName" label="成员姓名" min-width="140" />
              <el-table-column prop="relationCode" label="关系" width="120" />
              <el-table-column prop="gender" label="性别" width="90" />
              <el-table-column prop="birthday" label="生日" width="140" />
              <el-table-column label="状态" width="220">
                <template #default="{ row }">
                  <div class="flex items-center gap-2">
                    <el-tag v-if="String(accountContextState.currentMember?.memberId) === String(row.memberId)" size="small" type="success">当前</el-tag>
                    <el-tag v-if="row.isDefault" size="small" type="primary">默认</el-tag>
                    <el-tag v-if="row.status" size="small" effect="plain">{{ row.status }}</el-tag>
                  </div>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="280" fixed="right">
                <template #default="{ row }">
                  <div class="flex items-center gap-2">
                    <el-button size="small" plain @click="handleSwitchMember(row.memberId)">切换</el-button>
                    <el-button size="small" plain @click="openEditMemberDialog(row)">编辑</el-button>
                    <el-button size="small" plain :disabled="row.isDefault" @click="handleSetDefaultMember(row.memberId)">设默认</el-button>
                    <el-button size="small" type="danger" plain @click="handleDeleteMember(row)">删除</el-button>
                  </div>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </el-card>
      </el-tab-pane>

      <!-- ========== Prompt 设置 Tab ========== -->
      <el-tab-pane label="患者信息" name="prompt">
        <el-card shadow="never" class="!rounded-xl !border-slate-200">
          <template #header>
            <!-- 标题栏：连续点击7次进入开发者模式 -->
            <div
              class="flex items-center justify-between cursor-pointer select-none transition-all duration-300"
              @click="handleTitleClick"
              :class="isDeveloperMode ? 'text-amber-700' : ''"
              title=""
            >
              <div class="flex items-center gap-3">
                <span
                  class="material-symbols-outlined"
                  :class="isDeveloperMode ? 'text-amber-500' : 'text-[#2b8cee]'"
                >{{ isDeveloperMode ? 'code' : 'edit_note' }}</span>
                <span class="font-bold text-lg">患者信息设置</span>
                <el-tag
                  v-if="isDeveloperMode"
                  type="warning"
                  size="small"
                  effect="dark"
                  class="!ml-1 animate-pulse"
                >开发者模式</el-tag>
              </div>
              <div class="flex items-center gap-2">
                <!-- 点击进度提示（点击3次以上开始显示） -->
                <span
                  v-if="titleClickCount >= 3 && titleClickCount < 7 && !isDeveloperMode"
                  class="text-xs text-slate-400"
                >还需点击 {{ 7 - titleClickCount }} 次</span>
                <!-- 关闭开发者模式按钮 -->
                <el-button
                  v-if="isDeveloperMode"
                  type="warning"
                  size="small"
                  plain
                  @click.stop="exitDeveloperMode"
                >
                  <span class="material-symbols-outlined text-sm mr-1">lock</span>
                  关闭开发者模式
                </el-button>
              </div>
            </div>
          </template>

          <!-- 开发者模式警告横幅 -->
          <div
            v-if="isDeveloperMode"
            class="mb-6 px-4 py-3 bg-amber-50 border border-amber-200 rounded-lg flex items-start gap-3"
          >
            <span class="material-symbols-outlined text-amber-500 text-xl shrink-0">warning</span>
            <div>
              <p class="text-sm font-bold text-amber-800">当前处于开发者模式</p>
              <p class="text-xs text-amber-600 mt-0.5">OCR 识别 Prompt 和 AI 问答 Prompt 均已显示并可编辑，请谨慎操作以免影响系统功能。刷新页面后将自动退出开发者模式。</p>
            </div>
          </div>

          <el-form label-position="top" class="max-w-3xl space-y-2">
            <!-- ===== 仅开发者模式显示：OCR 识别 Prompt ===== -->
            <div v-if="isDeveloperMode" class="p-4 bg-amber-50/50 border border-amber-100 rounded-xl">
              <el-form-item>
                <template #label>
                  <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-sm text-amber-500">developer_mode</span>
                    <span class="font-bold text-sm text-amber-800">OCR 识别 Prompt 模板</span>
                    <el-tag size="small" type="warning" effect="plain">开发者专用</el-tag>
                  </div>
                </template>
                <el-input v-model="ocrPrompt" type="textarea" :rows="6" placeholder="请识别图片中的医疗检查报告..." />
              </el-form-item>
            </div>

            <!-- ===== 仅开发者模式显示：AI 问答 Prompt ===== -->
            <div v-if="isDeveloperMode" class="p-4 bg-amber-50/50 border border-amber-100 rounded-xl">
              <el-form-item>
                <template #label>
                  <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-sm text-amber-500">developer_mode</span>
                    <span class="font-bold text-sm text-amber-800">AI 问答 Prompt 模板</span>
                    <el-tag size="small" type="warning" effect="plain">开发者专用</el-tag>
                  </div>
                </template>
                <el-input v-model="aiPrompt" type="textarea" :rows="6" placeholder="你是一位专业的医疗健康分析师。请根据以下检查数据..." />
              </el-form-item>
            </div>

            <!-- ===== 始终显示：患者情况说明（用户自定义 Prompt）===== -->
            <div class="p-4 bg-slate-50 border border-slate-200 rounded-xl">
              <el-form-item>
                <template #label>
                  <div class="flex items-center gap-2 mb-1">
                    <span class="material-symbols-outlined text-sm text-[#2b8cee]">person</span>
                    <span class="font-bold text-sm text-slate-800">患者情况说明</span>
                  </div>
                  <p class="text-xs text-slate-500 leading-relaxed">
                    请在此填写患者的基本信息（年龄、性别、身高、体重、病史等）。此内容将作为背景信息随每次 AI 问答一起发送，无需每次重复填写。
                  </p>
                </template>
                <el-input
                  v-model="userCustomPrompt"
                  type="textarea"
                  :rows="10"
                  placeholder="请输入患者情况说明，例如：&#10;患者基本信息：&#10;- 年龄：60岁&#10;- 性别：男&#10;- 身高：172cm  体重：75kg&#10;- 主要病史：高血压、慢性肾病3期&#10;- 当前用药：降压药&#10;- 其他说明：需定期检查肾功能和血常规"
                />
              </el-form-item>
            </div>
          </el-form>

          <div class="flex justify-end mt-4 gap-2">
            <el-button @click="loadPrompts" :disabled="savingPrompt" plain>重置</el-button>
            <el-button type="primary" @click="savePrompts" :loading="savingPrompt">保存 Prompt 模板</el-button>
          </div>
        </el-card>
      </el-tab-pane>


      <el-tab-pane label="检查项目" name="project">
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
                <el-switch v-model="row.is_active" size="small" @change="toggleProject(row)" />
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
      </el-tab-pane>

      <el-tab-pane label="数据管理" name="data">
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
            
            <el-divider />
            
            <div class="px-2">
                <h4 class="font-bold text-slate-800 mb-2 flex items-center gap-2">
                    <span class="material-symbols-outlined text-blue-500">cloud_sync</span>
                    数据备份与还原
                </h4>
                 <p class="text-sm text-slate-500 mb-4 leading-relaxed">
                    定期备份数据是个好习惯。备份文件为 ZIP 压缩包，包含完整的数据库记录和检查报告图片。
                </p>
                <div class="flex gap-4">
                     <el-button type="primary" plain @click="handleBackupData">
                        <span class="material-symbols-outlined text-sm mr-2">upload</span>
                        导出备份
                    </el-button>
                    <el-button type="warning" plain @click="handleRestoreData">
                        <span class="material-symbols-outlined text-sm mr-2">download</span>
                        导入还原
                    </el-button>
                </div>
            </div>
        </el-card>
      </el-tab-pane>
    </el-tabs>

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

    <!-- 自定义模式说明抽屉 -->
    <el-drawer v-model="showCustomModeGuideDrawer" title="自定义模式接入说明" size="540px">
      <div class="space-y-6">
        <div class="rounded-xl border border-slate-200 bg-slate-50/70 p-4">
          <p class="text-sm font-bold text-slate-700">准备步骤</p>
          <ol class="mt-2 space-y-2">
            <li
              v-for="(step, index) in CUSTOM_MODE_GUIDE_STEPS"
              :key="step"
              class="text-sm text-slate-600 flex items-start gap-2"
            >
              <span class="w-5 h-5 rounded-full bg-blue-100 text-blue-600 text-xs font-bold flex items-center justify-center mt-0.5">{{ index + 1 }}</span>
              <span>{{ step }}</span>
            </li>
          </ol>
        </div>
        <div>
          <p class="text-sm font-bold text-slate-700 mb-3">可接入平台（注册链接占位）</p>
          <div class="space-y-3">
            <div
              v-for="guide in customModeProviderGuides"
              :key="guide.key"
              class="rounded-xl border border-slate-200 p-4 bg-white"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-bold text-slate-800">{{ guide.name }}</p>
                  <p class="text-xs text-slate-500 mt-1">{{ guide.description }}</p>
                  <p v-if="!guide.linkConfigured" class="text-xs text-amber-600 mt-2">注册链接暂未配置</p>
                  <p v-else class="text-xs text-slate-500 mt-2 break-all">{{ guide.signupUrl }}</p>
                </div>
                <el-button
                  size="small"
                  :disabled="!guide.linkConfigured"
                  @click="handleOpenProviderSignup(guide)"
                >
                  前往注册
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-drawer>

    <!-- 新增/编辑提供商弹窗 -->
    <el-dialog v-model="showProviderDialog" :title="editingProviderObj ? '编辑提供商' : '添加提供商'" width="380px" :close-on-click-modal="false">
      <div class="flex justify-center mb-4">
        <div class="w-14 h-14 rounded-full flex items-center justify-center text-2xl font-bold text-white"
             :style="{ background: getProviderColor(providerForm.type) }">
          {{ (providerForm.name || 'P').charAt(0).toUpperCase() }}
        </div>
      </div>
      <el-form :model="providerForm" label-position="top" size="default">
        <el-form-item label="提供商名称" required>
          <el-input v-model="providerForm.name" placeholder="例如 OpenAI" maxlength="32" />
        </el-form-item>
        <el-form-item label="提供商类型" required>
          <el-select v-model="providerForm.type" class="w-full">
            <el-option label="OpenAI" value="openai" />
            <el-option label="Gemini" value="gemini" />
            <el-option label="Anthropic" value="anthropic" />
            <el-option label="Azure OpenAI" value="azure-openai" />
            <el-option label="Ollama" value="ollama" />
            <el-option label="自定义 (Custom)" value="custom" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showProviderDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmSaveProvider">{{ editingProviderObj ? '保存' : '添加' }}</el-button>
      </template>
    </el-dialog>

    <!-- 新增/编辑模型弹窗 -->
    <el-dialog v-model="showModelDialog" :title="editingModelObj ? '编辑模型' : '添加模型'" width="400px" :close-on-click-modal="false">
      <el-form :model="modelForm" label-position="top">
        <el-form-item label="模型 ID" required>
          <el-input v-model="modelForm.modelId" placeholder="例如 gpt-3.5-turbo" />
          <span class="text-xs text-slate-400 mt-1">调用 API 时使用的真实模型 ID</span>
        </el-form-item>
        <el-form-item label="模型名称 (可选)">
          <el-input v-model="modelForm.modelName" placeholder="例如 GPT-4o" />
        </el-form-item>
        <el-form-item label="分组名称 (可选)">
          <el-input v-model="modelForm.groupName" placeholder="例如 ChatGPT" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showModelDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmSaveModel">{{ editingModelObj ? '保存' : '添加模型' }}</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="showMemberDialog" :title="editingMemberId ? '编辑家庭成员' : '新增家庭成员'" width="480px">
      <el-form :model="memberForm" label-position="top">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <el-form-item label="成员姓名" required>
            <el-input v-model="memberForm.memberName" maxlength="32" />
          </el-form-item>
          <el-form-item label="关系" required>
            <el-select v-model="memberForm.relationCode" class="w-full">
              <el-option label="本人" value="SELF" />
              <el-option label="父亲" value="FATHER" />
              <el-option label="母亲" value="MOTHER" />
              <el-option label="配偶" value="SPOUSE" />
              <el-option label="儿子" value="SON" />
              <el-option label="女儿" value="DAUGHTER" />
              <el-option label="其他" value="OTHER" />
            </el-select>
          </el-form-item>
          <el-form-item label="性别">
            <el-select v-model="memberForm.gender" class="w-full" clearable>
              <el-option label="男" value="MALE" />
              <el-option label="女" value="FEMALE" />
            </el-select>
          </el-form-item>
          <el-form-item label="生日">
            <el-date-picker v-model="memberForm.birthday" type="date" value-format="YYYY-MM-DD" class="!w-full" />
          </el-form-item>
        </div>
        <el-form-item label="手机号">
          <el-input v-model="memberForm.mobile" maxlength="20" />
        </el-form-item>
        <el-form-item label="健康备注">
          <el-input v-model="memberForm.healthNote" type="textarea" :rows="3" maxlength="200" show-word-limit />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showMemberDialog = false">取消</el-button>
        <el-button type="primary" :loading="savingMember" @click="submitMemberForm">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onActivated, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox, ElLoading } from 'element-plus'
import { save, open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  AI_MODES,
  CUSTOM_MODE_GUIDE_STEPS,
  resolveCustomModeProviderGuides,
  resolveUsageFeedback,
  useAccountContext,
  useAiMode,
  usePurchaseDialog,
} from '@/modules/security/index.js'

// ===== 多提供商管理 (ISS-060~066) =====
const activeTab = ref('ai')
const {
  state: accountContextState,
  refresh: refreshAccountContext,
  selectMember,
  createMember,
  updateMember,
  deleteMember,
  setDefaultMember,
} = useAccountContext()
const { state: aiModeState, setMode: setAiMode } = useAiMode()
const { openPurchaseDialog } = usePurchaseDialog()
const refreshingAccountContext = ref(false)
const searchProvider = ref('')
const activeProviderId = ref('')
const testingProviderId = ref('')
const savingPrompt = ref(false)

const providers = ref([])
const providerModels = ref([])

// 网络配置（全局）
const networkConfig = reactive({
  proxyEnabled: false,
  proxyUrl: '',
  proxyUsername: '',
  proxyPassword: '',
  timeout: 120,
})

const ocrPrompt = ref('')
const aiPrompt = ref('')
const userCustomPrompt = ref('')

// ===== 开发者模式（Prompt 保护）=====
const isDeveloperMode = ref(false)
const titleClickCount = ref(0)
let titleClickTimer = null

const handleTitleClick = () => {
  titleClickCount.value++
  // 重置计时器：3秒内未再次点击则清零
  clearTimeout(titleClickTimer)
  titleClickTimer = setTimeout(() => {
    if (!isDeveloperMode.value) {
      titleClickCount.value = 0
    }
  }, 3000)

  if (titleClickCount.value >= 7 && !isDeveloperMode.value) {
    isDeveloperMode.value = true
    titleClickCount.value = 0
    clearTimeout(titleClickTimer)
    ElMessage.success({ message: '已进入开发者模式，可编辑核心 Prompt 模板', type: 'warning', duration: 3000 })
  }
}

const exitDeveloperMode = () => {
  isDeveloperMode.value = false
  titleClickCount.value = 0
  ElMessage.info('已退出开发者模式')
}

// Provider / Model 弹窗
const showProviderDialog = ref(false)
const showModelDialog = ref(false)
const showMemberDialog = ref(false)
const showCustomModeGuideDrawer = ref(false)
const editingProviderObj = ref(null)
const editingModelObj = ref(null)
const editingMemberId = ref('')
const providerForm = reactive({ name: '', type: 'openai' })
const modelForm = reactive({ modelId: '', modelName: '', groupName: '' })
const savingMember = ref(false)
const memberForm = reactive({
  memberName: '',
  relationCode: 'OTHER',
  gender: '',
  birthday: '',
  mobile: '',
  healthNote: '',
})

// ===== 计算属性 =====
const filteredProviders = computed(() => {
  if (!searchProvider.value) return providers.value
  return providers.value.filter(p => p.name.toLowerCase().includes(searchProvider.value.toLowerCase()))
})

const activeProvider = computed(() => providers.value.find(p => p.id === activeProviderId.value))

const groupedModels = computed(() => {
  const groups = {}
  for (const m of providerModels.value) {
    const g = m.group_name || ''
    if (!groups[g]) groups[g] = []
    groups[g].push(m)
  }
  return groups
})
const accountUsage = computed(() => resolveUsageFeedback(accountContextState))
const accountPackageCards = computed(() => accountContextState.packageCards || [])
const activeAiMode = computed({
  get: () => aiModeState.mode,
  set: (nextMode) => {
    setAiMode(nextMode)
  },
})
const isGeneralMode = computed(() => activeAiMode.value === AI_MODES.general)
const customModeProviderGuides = computed(() => resolveCustomModeProviderGuides())

// ===== 颜色映射 =====
const PROVIDER_COLORS = {
  openai: '#10a37f',
  gemini: '#4285f4',
  anthropic: '#d97706',
  'azure-openai': '#0078d4',
  ollama: '#334155',
  custom: '#6366f1',
}
const getProviderColor = (type) => PROVIDER_COLORS[type] || '#6366f1'

const handleRefreshAccountContext = async () => {
  refreshingAccountContext.value = true
  try {
    await refreshAccountContext({ force: true })
  } catch (e) {
    ElMessage.error('账户次数刷新失败: ' + (e?.message || e))
  } finally {
    refreshingAccountContext.value = false
  }
}

const handleOpenPurchaseDialog = (preferredCalls) => {
  void openPurchaseDialog({
    preferredCalls,
    reason: 'settings',
  })
}

const resetMemberForm = () => {
  editingMemberId.value = ''
  memberForm.memberName = ''
  memberForm.relationCode = 'OTHER'
  memberForm.gender = ''
  memberForm.birthday = ''
  memberForm.mobile = ''
  memberForm.healthNote = ''
}

const openCreateMemberDialog = () => {
  resetMemberForm()
  showMemberDialog.value = true
}

const openEditMemberDialog = (member) => {
  editingMemberId.value = String(member.memberId)
  memberForm.memberName = member.memberName || ''
  memberForm.relationCode = member.relationCode || 'OTHER'
  memberForm.gender = member.gender || ''
  memberForm.birthday = member.birthday || ''
  memberForm.mobile = member.mobile || ''
  memberForm.healthNote = member.healthNote || ''
  showMemberDialog.value = true
}

const submitMemberForm = async () => {
  if (!memberForm.memberName.trim()) {
    ElMessage.warning('请输入成员姓名')
    return
  }

  if (!memberForm.relationCode) {
    ElMessage.warning('请选择成员关系')
    return
  }

  const payload = {
    memberName: memberForm.memberName.trim(),
    relationCode: memberForm.relationCode,
    gender: memberForm.gender || undefined,
    birthday: memberForm.birthday || undefined,
    mobile: memberForm.mobile?.trim() || undefined,
    healthNote: memberForm.healthNote?.trim() || undefined,
  }

  savingMember.value = true
  try {
    if (editingMemberId.value) {
      await updateMember(editingMemberId.value, payload)
      ElMessage.success('成员信息已更新')
    } else {
      await createMember(payload)
      ElMessage.success('成员已创建')
    }
    showMemberDialog.value = false
  } catch (e) {
    ElMessage.error(e?.message || '保存成员失败')
  } finally {
    savingMember.value = false
  }
}

const handleSwitchMember = async (memberId) => {
  try {
    selectMember(memberId)
    ElMessage.success('已切换当前成员')
  } catch (e) {
    ElMessage.error(e?.message || '切换成员失败')
  }
}

const handleSetDefaultMember = async (memberId) => {
  try {
    await setDefaultMember(memberId)
    ElMessage.success('默认成员已更新')
  } catch (e) {
    ElMessage.error(e?.message || '设置默认成员失败')
  }
}

const handleDeleteMember = async (member) => {
  if ((accountContextState.members || []).length <= 1) {
    ElMessage.warning('至少保留 1 个家庭成员')
    return
  }

  try {
    await ElMessageBox.confirm(`确定删除成员「${member.memberName || member.memberId}」吗？`, '确认删除', {
      type: 'warning',
    })
    await deleteMember(member.memberId)
    ElMessage.success('成员已删除')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(e?.message || '删除成员失败')
    }
  }
}

const openCustomModeGuide = () => {
  showCustomModeGuideDrawer.value = true
}

const handleOpenProviderSignup = async (guide) => {
  if (!guide?.linkConfigured || !guide?.signupUrl) {
    ElMessage.info('该平台注册链接暂未配置')
    return
  }

  try {
    await openUrl(guide.signupUrl)
  } catch (e) {
    ElMessage.error('打开注册链接失败: ' + (e?.message || e))
  }
}

// ===== 加载数据 =====
const loadProviders = async () => {
  try {
    providers.value = await invoke('list_providers')
    if (providers.value.length > 0 && !activeProviderId.value) {
      activeProviderId.value = providers.value[0].id
    }
    if (activeProviderId.value) {
      await loadModels()
    }
  } catch (e) {
    console.error('加载提供商失败:', e)
  }
}

const loadModels = async () => {
  if (!activeProviderId.value) { providerModels.value = []; return }
  try {
    providerModels.value = await invoke('list_provider_models', { providerId: activeProviderId.value })
  } catch (e) {
    providerModels.value = []
  }
}

const selectProvider = async (id) => {
  activeProviderId.value = id
  await loadModels()
}

const loadNetworkConfig = async () => {
  try {
    networkConfig.proxyEnabled = (await invoke('get_config', { key: 'proxy_enabled' })) === 'true'
    networkConfig.proxyUrl = (await invoke('get_config', { key: 'proxy_url' })) || ''
    networkConfig.proxyUsername = (await invoke('get_config', { key: 'proxy_username' })) || ''
    networkConfig.proxyPassword = (await invoke('get_config', { key: 'proxy_password' })) || ''
    const t = await invoke('get_config', { key: 'ai_timeout' })
    networkConfig.timeout = t ? parseInt(t) : 120
  } catch (e) { console.error(e) }
}

const DEFAULT_OCR_PROMPT = '你是一位专业的医疗影像识别专家。请识别图片中的医疗检查报告，提取所有检查指标的名称、数值、单位和参考范围，请严格按照以下JSON格式返回: [ { "name": "指标名称", "value": "数值", "unit": "单位", "reference_range": "参考范围", "status": "正常/异常" } ] 注意：reference_range 必须是字符串，表示参考范围（如 "3.5-5.5"）。2，status必须是 "正常" 或 "异常"。3，只返回JSON数组，不要包含markdown代码块或其他文字。'
const DEFAULT_AI_PROMPT = '你是一位专业的医疗健康分析师。请根据以下检查数据，综合分析患者的健康状况，指出异常指标，提供治疗建议和生活方式改善方案。'
const DEFAULT_USER_CUSTOM_PROMPT = '患者基本信息：\n- 姓名：（患者姓名）\n- 年龄：（岁）\n- 性别：（男/女）\n- 身高：（cm）\n- 体重：（kg）\n- 主要病史：（现有疾病或既往病史）\n- 当前用药：（正在服用的药物）\n- 其他说明：（其他需要医生了解的情况）'

const loadPrompts = async () => {
  try {
    const ocrTpl = await invoke('get_config', { key: 'ocr_prompt_template' })
    const aiTpl = await invoke('get_config', { key: 'ai_analysis_prompt_template' })
    const userTpl = await invoke('get_config', { key: 'user_custom_prompt_template' })
    ocrPrompt.value = ocrTpl || DEFAULT_OCR_PROMPT
    aiPrompt.value = aiTpl || DEFAULT_AI_PROMPT
    userCustomPrompt.value = userTpl || DEFAULT_USER_CUSTOM_PROMPT
  } catch (e) { console.error(e) }
}

// ===== 保存 =====
const saveNetworkConfig = async () => {
  try {
    await invoke('save_config', { key: 'proxy_enabled', value: String(networkConfig.proxyEnabled) })
    await invoke('save_config', { key: 'proxy_url', value: networkConfig.proxyUrl })
    await invoke('save_config', { key: 'proxy_username', value: networkConfig.proxyUsername })
    await invoke('save_config', { key: 'proxy_password', value: networkConfig.proxyPassword })
    await invoke('save_config', { key: 'ai_timeout', value: String(networkConfig.timeout) })
  } catch (e) { console.error(e) }
}

const savePrompts = async () => {
  savingPrompt.value = true
  try {
    // 始终保存用户自定义 Prompt
    await invoke('save_config', { key: 'user_custom_prompt_template', value: userCustomPrompt.value })
    // 开发者模式下才保存核心 Prompt
    if (isDeveloperMode.value) {
      await invoke('save_config', { key: 'ocr_prompt_template', value: ocrPrompt.value })
      await invoke('save_config', { key: 'ai_analysis_prompt_template', value: aiPrompt.value })
    }
    ElMessage.success('Prompt 模板保存成功')
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  } finally {
    savingPrompt.value = false
  }
}

const saveProviderField = async (p) => {
  try {
    await invoke('update_provider', {
      id: p.id,
      apiKey: p.api_key,
      apiUrl: p.api_url,
    })
  } catch (e) { ElMessage.error('保存失败: ' + e) }
}

const toggleProviderEnabled = async (p) => {
  try {
    await invoke('update_provider', { id: p.id, enabled: p.enabled })
  } catch (e) {
    ElMessage.error('切换失败: ' + e)
    p.enabled = !p.enabled
  }
}

// ===== Provider CRUD =====
const openAddProviderDialog = () => {
  editingProviderObj.value = null
  providerForm.name = ''
  providerForm.type = 'openai'
  showProviderDialog.value = true
}

const editProvider = (p) => {
  editingProviderObj.value = p
  providerForm.name = p.name
  providerForm.type = p.type
  showProviderDialog.value = true
}

const confirmSaveProvider = async () => {
  if (!providerForm.name.trim()) { ElMessage.warning('请输入名称'); return }
  try {
    if (editingProviderObj.value) {
      await invoke('update_provider', {
        id: editingProviderObj.value.id,
        name: providerForm.name,
        providerType: providerForm.type,
      })
      ElMessage.success('提供商已更新')
    } else {
      const newP = await invoke('create_provider', {
        name: providerForm.name,
        providerType: providerForm.type,
      })
      activeProviderId.value = newP.id
    }
    showProviderDialog.value = false
    await loadProviders()
  } catch (e) { ElMessage.error('' + e) }
}

const delProvider = async (id) => {
  try {
    await ElMessageBox.confirm('删除提供商将同时删除其下所有模型，确认继续？', '确认', { type: 'warning' })
    await invoke('delete_provider', { id })
    if (activeProviderId.value === id) {
      activeProviderId.value = providers.value.length > 1 ? providers.value.find(p => p.id !== id)?.id || '' : ''
    }
    await loadProviders()
    ElMessage.success('已删除')
  } catch (e) { if (e !== 'cancel') ElMessage.error('' + e) }
}

// ===== Model CRUD =====
const openAddModelDialog = () => {
  editingModelObj.value = null
  modelForm.modelId = ''
  modelForm.modelName = ''
  modelForm.groupName = ''
  showModelDialog.value = true
}

const editModel = (m) => {
  editingModelObj.value = m
  modelForm.modelId = m.model_id
  modelForm.modelName = m.model_name
  modelForm.groupName = m.group_name
  showModelDialog.value = true
}

const confirmSaveModel = async () => {
  if (!modelForm.modelId.trim()) { ElMessage.warning('模型 ID 为必填项'); return }
  try {
    if (editingModelObj.value) {
      await invoke('update_model_info', {
        id: editingModelObj.value.id,
        modelId: modelForm.modelId.trim(),
        modelName: modelForm.modelName.trim() || null,
        groupName: modelForm.groupName.trim() || null,
      })
      ElMessage.success('模型已更新')
    } else {
      await invoke('add_model', {
        providerId: activeProviderId.value,
        modelId: modelForm.modelId.trim(),
        modelName: modelForm.modelName.trim() || null,
        groupName: modelForm.groupName.trim() || null,
      })
    }
    showModelDialog.value = false
    await loadModels()
  } catch (e) { ElMessage.error('' + e) }
}

const handleDeleteModel = async (m) => {
  try {
    await invoke('delete_model', { id: m.id })
    await loadModels()
  } catch (e) { ElMessage.error('' + e) }
}

const handleSetDefault = async (m) => {
  try {
    await invoke('set_default_model', { id: m.id })
    await loadModels()
    ElMessage.success('已设为全局默认模型')
  } catch (e) { ElMessage.error('' + e) }
}

// ===== 连接测试 =====
const testProviderConnection = async (p) => {
  testingProviderId.value = p.id
  try {
    await saveProviderField(p) // 确保最新配置已保存
    const result = await invoke('test_provider_connection', { providerId: p.id })
    ElMessage.success(result)
  } catch (e) {
    ElMessage.error('连接测试失败: ' + e)
  } finally {
    testingProviderId.value = ''
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

const handleBackupData = async () => {
  try {
    // 默认文件名：health_backup_YYYY-MM-DD-HH-mm-ss.zip
    const defaultName = `health_backup_${new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-')}.zip`
    const path = await save({
      filters: [{
        name: 'ZIP Backup',
        extensions: ['zip']
      }],
      defaultPath: defaultName
    });
    
    if (!path) return;
    
    const loading = ElLoading.service({ text: '正在备份数据...', background: 'rgba(255, 255, 255, 0.7)' });
    try {
       await invoke('backup_data', { targetPath: path });
       ElMessage.success('数据备份成功');
    } finally {
       loading.close();
    }
  } catch (e) {
    ElMessage.error('备份失败: ' + e);
  }
}

const handleRestoreData = async () => {
    try {
        const path = await open({
           filters: [{
             name: 'ZIP Backup',
             extensions: ['zip']
           }],
           multiple: false,
           directory: false
        });
        
        if (!path) return;
        
        await ElMessageBox.confirm(
            '还原操作将覆盖当前所有数据（包括图片和数据库），不可撤销！\n建议先备份当前数据。\n\n确定要继续还原吗？',
            '确认还原',
            { 
              type: 'warning', 
              confirmButtonText: '继续还原', 
              cancelButtonText: '取消',
              confirmButtonClass: 'el-button--warning'
            }
        );
        
        const loading = ElLoading.service({ text: '正在还原数据，请勿关闭程序...', background: 'rgba(255, 255, 255, 0.7)' });
        try {
            await invoke('restore_data', { sourcePath: path });
            loading.close(); // 关闭 loading 再弹窗
            
            await ElMessageBox.alert('数据还原成功！即将刷新页面以加载新数据。', '还原成功', {
              confirmButtonText: '确定',
              type: 'success'
            });
            window.location.reload();
        } catch (e) {
            loading.close();
           if (e !== 'cancel') ElMessage.error('还原失败: ' + e);
        }
    } catch (e) {
        if (e !== 'cancel') ElMessage.error('还原操作异常: ' + e);
    }
}


// ===== 初始化 =====
onMounted(() => {
  void handleRefreshAccountContext()
  loadProviders()
  loadNetworkConfig()
  loadPrompts()
  loadProjects()
})

// 路由使用 keep-alive，返回设置页时需要主动刷新项目/指标数据
onActivated(() => {
  if (activeTab.value === 'ai') {
    void handleRefreshAccountContext()
  }
  loadProjects()
  if (showIndicatorDrawer.value && currentProject.value) {
    loadIndicators()
  }
})
</script>
