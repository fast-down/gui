import Vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { defineConfig } from 'vite'
import { PrimeVueResolver } from '@primevue/auto-import-resolver'
import PostcssPresetEnv from 'postcss-preset-env'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    Vue(),
    Components({
      resolvers: [PrimeVueResolver()],
      dts: 'types/components.d.ts',
    }),
    AutoImport({
      imports: [
        'vue',
        'pinia',
        '@vueuse/core',
        {
          from: '@tauri-apps/api/core',
          imports: ['invoke'],
        },
      ],
      dirs: ['src/stores', 'src/utils', 'src/binding'],
      dts: 'types/auto-imports.d.ts',
      eslintrc: {
        enabled: true,
        filepath: 'types/.eslintrc-auto-import.json',
      },
    }),
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  css: {
    postcss: {
      plugins: [PostcssPresetEnv()],
    },
  },
})
