import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Glass from './theme';
import App from './App.vue';
import router from './router';
import './style.css';
import 'primeicons/primeicons.css';

document.documentElement.classList.add('light');

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(PrimeVue, {
  theme: {
    preset: Glass,
    options: {
      prefix: 'p',
      darkModeSelector: '.dark',
      cssLayer: {
        name: 'ui',
        order: 'theme, base, ui',
      },
    },
  },
});

app.mount('#app');
