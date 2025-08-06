/**
 * main.ts
 *
 * Bootstraps Vuetify and other plugins then mounts the App`
 */

import Aura from '@primeuix/themes/aura';
import PrimeVue from 'primevue/config';
import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router'
import { routes } from 'vue-router/auto-routes'
import App from './App.vue';
import 'unfonts.css';
import './styles.scss'
import pinia from "@/stores"

const app = createApp(App);
app.use(PrimeVue, {
  theme: {
    preset: Aura,
  },
});
app.use(createRouter({
  history: createWebHistory(),
  routes,
}));
app.use(pinia)
app.mount('#app');
