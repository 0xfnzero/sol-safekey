"use client";

import { useCallback } from "react";
import { useParams } from "next/navigation";
import enMessages from "@/messages/en.json";
import zhMessages from "@/messages/zh.json";

const messagesMap = {
  en: enMessages,
  zh: zhMessages,
};

export function useTranslations() {
  const params = useParams();
  const locale = (params?.locale as string) || "en";
  const messages = messagesMap[locale as keyof typeof messagesMap] || messagesMap.en;

  return useCallback(function t(
    key: string,
    vars?: Record<string, string | number>,
  ): string {
    const keyParts = key.split(".");
    let value: unknown = messages;

    for (const k of keyParts) {
      if (value && typeof value === "object" && k in value) {
        value = (value as Record<string, unknown>)[k];
      } else {
        return key;
      }
    }

    let out = typeof value === "string" ? value : key;
    if (vars && out.includes("{")) {
      out = out.replace(/\{(\w+)\}/g, (_, name: string) =>
        vars[name] !== undefined ? String(vars[name]) : `{${name}}`,
      );
    }
    return out;
  }, [messages]);
}
