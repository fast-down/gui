import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import './styles/index.css'
import 'primeicons/primeicons.css'
import ToastService from 'primevue/toastservice'
import Preset from './preset.ts'

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

const app = createApp(App)
app.use(PrimeVue, {
  theme: {
    preset: Preset,
  },
  ripple: true,
})
app.use(pinia)
app.use(ToastService)
app.mount('#app')
