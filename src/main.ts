import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config'
import { createPinia } from 'pinia'
import PiniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Aura from '@primeuix/themes/aura'
import './styles/index.css'
import 'primeicons/primeicons.css'
import ToastService from 'primevue/toastservice'
import { attachConsole } from '@tauri-apps/plugin-log'

attachConsole()

const pinia = createPinia()
pinia.use(PiniaPluginPersistedstate)

const app = createApp(App)
app.config.throwUnhandledErrorInProduction = true
app.use(PrimeVue, {
  theme: {
    preset: Aura,
  },
  ripple: true,
})
app.use(pinia)
app.use(ToastService)
app.mount('#app')
