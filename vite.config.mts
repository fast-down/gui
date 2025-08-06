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

const host = process.env.TAURI_DEV_HOST;

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
  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    // make sure this port matches the devUrl port in tauri.conf.json file
    port: 3000,
    // Tauri expects a fixed port, fail if that port is not available
    strictPort: true,
    // if the host Tauri is expecting is set, use it
    host: host || false,
    hmr: host
      ? {
        protocol: 'ws',
        host,
        port: 1421,
      }
      : undefined,

    watch: {
      // tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target:
      process.env.TAURI_ENV_PLATFORM == 'windows'
        ? 'chrome105'
        : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url)),
    },
    extensions: ['.js', '.json', '.jsx', '.mjs', '.ts', '.tsx', '.vue'],
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
