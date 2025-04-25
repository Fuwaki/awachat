import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import 'vuetify/styles'
import App from './App.vue'
import router from './router'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { SocketService } from './services/socketService'

const vuetify = createVuetify({
  components: components,
  directives: directives,
})

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(vuetify)

SocketService.init()

app.mount('#app')
