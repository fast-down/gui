import { fileURLToPath, URL } from 'node:url';
import vue from '@vitejs/plugin-vue';
import autoImport from 'unplugin-auto-import/vite';
import fonts from 'unplugin-fonts/vite';
import components from 'unplugin-vue-components/vite';
import { VueRouterAutoImports } from 'unplugin-vue-router';
import vueRouter from 'unplugin-vue-router/vite';
import { defineConfig } from 'vite';
import {PrimeVueResolver} from '@primevue/auto-import-resolver';
import layouts from 'vite-plugin-vue-layouts-next';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vueRouter({
      dts: 'src/typed-router.d.ts',
    }),
    layouts(),
    autoImport({
      imports: [
        'vue',
        VueRouterAutoImports,
        {
          pinia: ['defineStore', 'storeToRefs'],
        },
      ],
      dts: 'src/auto-imports.d.ts',
      eslintrc: {
        enabled: true,
      },
      vueTemplate: true,
    }),
    components({
      resolvers: [PrimeVueResolver()],
      dts: 'src/components.d.ts',
    }),
    vue(),
    fonts({
      fontsource: {
        families: [
          {
            name: 'Roboto',
            weights: [100, 300, 400, 500, 700, 900],
            styles: ['normal', 'italic'],
          },
        ],
      },
    }),
  ],
  define: { 'process.env': {} },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url)),
    },
    extensions: ['.js', '.json', '.jsx', '.mjs', '.ts', '.tsx', '.vue'],
  },
  server: {
    port: 3000,
  },
  css: {
    preprocessorOptions: {
      sass: {
        api: 'modern-compiler',
      },
      scss: {
        api: 'modern-compiler',
      },
    },
  },
});
