import '@fontsource/ibm-plex-sans/latin-400.css'
import '@fontsource/ibm-plex-sans/latin-500.css'
import '@fontsource/ibm-plex-sans/latin-600.css'
import '@fontsource/ibm-plex-sans/latin-700.css'
// Turkish characters (ğ, ş, İ, ı) are in the latin-ext subset.
import '@fontsource/ibm-plex-sans/latin-ext-400.css'
import '@fontsource/ibm-plex-sans/latin-ext-500.css'
import '@fontsource/ibm-plex-sans/latin-ext-600.css'
import '@fontsource/ibm-plex-sans/latin-ext-700.css'
import '@fontsource-variable/bricolage-grotesque/index.css'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { vReveal, vTilt } from './lib/directives'
import { mainnetShowcase } from './lib/stellar'
import { router } from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.directive('reveal', vReveal)
app.directive('tilt', vTilt)

async function start() {
  if (!mainnetShowcase) {
    const [{ initWalletKit }, { useWalletStore }] = await Promise.all([
      import('./lib/wallet-kit'),
      import('./stores/wallet'),
    ])
    initWalletKit()
    // Restore the previous wallet connection (the kit keeps the address in localStorage).
    void useWalletStore().restore()
  }
  app.mount('#app')
}

void start()
