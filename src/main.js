import { createApp } from 'vue'
import './assets/main.css'
import './assets/fonts/fonts.css'
import App from './App.vue'
import router from './router/index'
import plugins from './plugins/index'
import tray_init from "./tray.js"
import { bootstrapClientSecurity } from './modules/security/index.js'

tray_init()
const useDevProxy = import.meta.env.DEV && import.meta.env.VITE_USE_DEV_PROXY !== 'false'
void bootstrapClientSecurity({
  // In development, prefer Vite proxy to avoid browser CORS preflight failures.
  baseUrl: useDevProxy ? '' : (import.meta.env.VITE_API_BASE_URL || ''),
  appVersion: import.meta.env.VITE_APP_VERSION || '1.0.2',
  activationSecretProof: import.meta.env.VITE_DEVICE_SECRET_PROOF || 'init-secret-proof',
})
createApp(App).use(router).use(plugins).mount('#app')
