import { createApp } from 'vue'
import './assets/main.css'
import './assets/fonts/fonts.css'
import App from './App.vue'
import router from './router/index'
import plugins from './plugins/index'
import tray_init from "./tray.js"

tray_init()
createApp(App).use(router).use(plugins).mount('#app')
