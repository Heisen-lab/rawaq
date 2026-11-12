import React from "react";
import ReactDOM from "react-dom/client";
import { I18nProvider } from "./i18n/I18nContext";
import { App } from "./App";
import "./styles/bidi.css";

const rootElement = document.getElementById("root");
if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <I18nProvider initialLocale="ar">
        <App />
      </I18nProvider>
    </React.StrictMode>,
  );
}
