import { mount } from 'svelte';
import '@fontsource-variable/fraunces';
import './styles/app.css';
import App from './App.svelte';

mount(App, { target: document.getElementById('app')! });
