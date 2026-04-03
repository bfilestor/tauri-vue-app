export const CUSTOM_MODE_GUIDE_STEPS = Object.freeze([
  '在对应平台创建开发者账号并完成实名认证。',
  '创建 API Key 并妥善保存，不要写入公开仓库或聊天截图。',
  '回到“模型服务 > 自定义模式”填写 API Key、API 地址并添加模型。',
])

export const CUSTOM_MODE_PROVIDER_GUIDES = Object.freeze([
  {
    key: 'zhipu',
    name: '智谱 AI',
    description: '支持 GLM 系列模型，适合中文场景。',
    signupUrl: '',
  },
  {
    key: 'wenxin',
    name: '文心一言',
    description: '百度智能云文心模型与应用服务。',
    signupUrl: '',
  },
  {
    key: 'doubao',
    name: '豆包',
    description: '火山引擎大模型服务入口。',
    signupUrl: '',
  },
])

export function resolveCustomModeProviderGuides(guides = CUSTOM_MODE_PROVIDER_GUIDES) {
  return guides.map((item) => {
    const signupUrl = typeof item.signupUrl === 'string'
      ? item.signupUrl.trim()
      : ''

    return {
      ...item,
      signupUrl,
      linkConfigured: /^https?:\/\//i.test(signupUrl),
    }
  })
}
