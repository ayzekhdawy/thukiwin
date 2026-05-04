import React from 'react';
import ReactDOM from 'react-dom/client';
import { SettingsWindow } from './settings/SettingsWindow';
import { ConfigProvider } from './contexts/ConfigContext';
import './App.css';

ReactDOM.createRoot(document.getElementById('settings-root') as HTMLElement).render(
  <React.StrictMode>
    <ConfigProvider>
      <SettingsWindow />
    </ConfigProvider>
  </React.StrictMode>,
);