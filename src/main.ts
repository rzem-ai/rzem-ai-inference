import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Glass from './theme';
import App from './App.vue';
import router from './router';
import { LucidePlugin } from './plugins/lucide';
import { PrimeVueComponents } from './plugins/primevue-components';
import './theme/fonts/fonts.css';
import './style.css';
import 'primeicons/primeicons.css';

document.documentElement.classList.add('light');

// Prevent WebKit/pywebview from navigating to dropped files
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', (e) => e.preventDefault());

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(LucidePlugin);
app.use(PrimeVueComponents);
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
