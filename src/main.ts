import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import Tooltip from 'primevue/tooltip';
import router from './router';
import App from './App.vue';

import { AuraPlus } from './assets/theme';
import './style.css';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.directive('tooltip', Tooltip);
app.use(PrimeVue, {
  theme: {
    preset: AuraPlus,
    options: {
      darkModeSelector: '.dark',
      cssLayer: {
        name: 'ui',
        order: 'theme, base, ui',
      },
    },
  },
});
app.use(ToastService);

app.mount('#app');
