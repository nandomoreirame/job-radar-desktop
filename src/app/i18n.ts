import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "@/shared/locales/en.json";
import ptBR from "@/shared/locales/pt-BR.json";

void i18n.use(initReactI18next).init({
  resources: { "pt-BR": { translation: ptBR }, en: { translation: en } },
  lng: "pt-BR",
  fallbackLng: "pt-BR",
  // React already escapes rendered text.
  interpolation: { escapeValue: false },
  initAsync: false,
});

export default i18n;
