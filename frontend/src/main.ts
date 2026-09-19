import { createPinia } from 'pinia'
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { initWalletKit } from './lib/wallet-kit'
import { router } from './router'
import { useWalletStore } from './stores/wallet'

initWalletKit()

const app = createApp(App)
app.use(createPinia())
app.use(router)

// Önceki cüzdan bağlantısını geri yükle (kit adresi localStorage'da tutar).
void useWalletStore().restore()

app.mount('#app')
