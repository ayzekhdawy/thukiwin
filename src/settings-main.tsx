import React from 'react';
import ReactDOM from 'react-dom/client';
import './App.css';
import { SettingsView } from './components/SettingsView';

ReactDOM.createRoot(document.getElementById('settings-root') as HTMLElement).render(
  <React.StrictMode>
    <SettingsView modelConfig={null} isStandalone />
  </React.StrictMode>,
);