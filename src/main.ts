import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import router from './router';
import App from './App.vue';

import { AuraPlus } from './assets/theme';
import './style.css';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(PrimeVue, {
  theme: {
    preset: AuraPlus,
    options: {
      darkModeSelector: '.dark-mode',
      cssLayer: {
        name: 'primevue',
        order: 'theme, base, primevue',
      },
    },
  },
});
app.use(ToastService);

app.mount('#app');
