import { createVuetify } from 'vuetify'
import { VIconBtn } from 'vuetify/labs/VIconBtn'
import { en, zhHans, zhHant } from 'vuetify/locale'
import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
  components: {
    VIconBtn,
  },
  theme: {
    defaultTheme: 'system',
  },
  locale: {
    locale: 'zhHans',
    fallback: 'en',
    messages: { zhHans, zhHant, en },
  },
})
