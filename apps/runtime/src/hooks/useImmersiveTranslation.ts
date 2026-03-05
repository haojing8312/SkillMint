import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RuntimePreferences } from "../types";

interface UseImmersiveTranslationOptions {
  scene?: string;
  batchSize?: number;
  autoTranslate?: boolean;
}

type ImmersiveTranslationDisplayMode = "translated_only" | "bilingual_inline";
type ImmersiveTranslationTriggerMode = "auto" | "manual";

function containsLatinLetters(text: string): boolean {
  return /[A-Za-z]/.test(text);
}

function normalizeDisplayMode(raw: unknown): ImmersiveTranslationDisplayMode {
  return raw === "bilingual_inline" ? "bilingual_inline" : "translated_only";
}

function normalizeTriggerMode(raw: unknown): ImmersiveTranslationTriggerMode {
  return raw === "manual" ? "manual" : "auto";
}

export function useImmersiveTranslation(
  texts: string[],
  options?: UseImmersiveTranslationOptions,
) {
  const [translatedMap, setTranslatedMap] = useState<Record<string, string>>({});
  const [isTranslating, setIsTranslating] = useState(false);
  const [displayMode, setDisplayMode] = useState<ImmersiveTranslationDisplayMode>("translated_only");
  const [triggerMode, setTriggerMode] = useState<ImmersiveTranslationTriggerMode>("auto");
  const [immersiveEnabled, setImmersiveEnabled] = useState(true);
  const [preferencesLoaded, setPreferencesLoaded] = useState(false);
  const [translationFallbackActive, setTranslationFallbackActive] = useState(false);
  const [translationError, setTranslationError] = useState("");
  const translatingRef = useRef(false);
  const manualTranslateActiveRef = useRef(false);
  const mountedRef = useRef(true);

  const scene = options?.scene ?? null;
  const batchSize = Math.max(1, Math.min(200, options?.batchSize ?? 80));
  const autoTranslate = options?.autoTranslate ?? (preferencesLoaded && triggerMode === "auto");

  const candidates = useMemo(
    () =>
      Array.from(
        new Set(
          texts
            .map((text) => text?.trim())
            .filter((text): text is string => Boolean(text) && !translatedMap[text!]),
        ),
      ),
    [texts, translatedMap],
  );

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const prefs = await invoke<RuntimePreferences | null>("get_runtime_preferences");
        if (cancelled) return;
        setDisplayMode(normalizeDisplayMode(prefs?.immersive_translation_display));
        setTriggerMode(normalizeTriggerMode(prefs?.immersive_translation_trigger));
        setImmersiveEnabled(
          typeof prefs?.immersive_translation_enabled === "boolean"
            ? prefs.immersive_translation_enabled
            : true,
        );
      } catch {
        if (!cancelled) {
          setDisplayMode("translated_only");
          setTriggerMode("auto");
          setImmersiveEnabled(true);
        }
      } finally {
        if (!cancelled) setPreferencesLoaded(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!immersiveEnabled) {
      manualTranslateActiveRef.current = false;
      setTranslationFallbackActive(false);
      setTranslationError("");
    }
  }, [immersiveEnabled]);

  const translateBatch = useCallback(
    async (sourceBatch: string[]) => {
      if (sourceBatch.length === 0 || translatingRef.current) return false;
      translatingRef.current = true;
      setIsTranslating(true);
      try {
        const translatedRaw = await invoke<unknown>("translate_texts_with_preferences", {
          texts: sourceBatch,
          scene,
        });
        const translated = Array.isArray(translatedRaw)
          ? translatedRaw.map((value) => (typeof value === "string" ? value : String(value ?? "")))
          : [];
        if (!mountedRef.current) return false;
        const next: Record<string, string> = {};
        let latinCount = 0;
        let latinTranslatedCount = 0;
        for (let i = 0; i < sourceBatch.length; i += 1) {
          const source = sourceBatch[i];
          const target = (translated[i] ?? source).trim();
          next[source] = target;
          if (containsLatinLetters(source)) {
            latinCount += 1;
            if (target !== source) {
              latinTranslatedCount += 1;
            }
          }
        }
        setTranslatedMap((prev) => ({ ...prev, ...next }));
        if (immersiveEnabled && latinCount > 0) {
          setTranslationFallbackActive((prev) => {
            if (latinTranslatedCount > 0) return false;
            return prev || latinCount > 0;
          });
        }
        setTranslationError("");
        return true;
      } catch (error) {
        if (immersiveEnabled && sourceBatch.some(containsLatinLetters)) {
          if (mountedRef.current) setTranslationFallbackActive(true);
        }
        if (mountedRef.current) {
          setTranslationError(error instanceof Error ? error.message : String(error || "翻译失败"));
        }
        return false;
      } finally {
        translatingRef.current = false;
        if (mountedRef.current) setIsTranslating(false);
      }
    },
    [immersiveEnabled, scene],
  );

  useEffect(() => {
    if (candidates.length === 0) {
      manualTranslateActiveRef.current = false;
      return;
    }
    if (!autoTranslate && !manualTranslateActiveRef.current) return;
    if (translatingRef.current) return;
    const limited = candidates.slice(0, batchSize);
    void translateBatch(limited).then((ok) => {
      if (!ok && !autoTranslate) {
        manualTranslateActiveRef.current = false;
      }
    });
  }, [autoTranslate, batchSize, candidates, translateBatch]);

  const translateNow = useCallback(async () => {
    if (!immersiveEnabled) {
      setTranslationError("请先在设置中开启沉浸式翻译");
      return false;
    }
    const limited = candidates.slice(0, batchSize);
    if (limited.length === 0) {
      manualTranslateActiveRef.current = false;
      setTranslationError("");
      return true;
    }
    manualTranslateActiveRef.current = true;
    if (translatingRef.current) {
      setTranslationError("");
      return true;
    }
    const ok = await translateBatch(limited);
    if (!ok) {
      manualTranslateActiveRef.current = false;
    }
    return ok;
  }, [batchSize, candidates, immersiveEnabled, translateBatch]);

  const renderDisplayText = useCallback(
    (sourceText: string) => {
      const translated = translatedMap[sourceText] ?? sourceText;
      if (displayMode === "bilingual_inline" && translated !== sourceText) {
        return `${translated} (${sourceText})`;
      }
      return translated;
    },
    [displayMode, translatedMap],
  );

  return {
    translatedMap,
    isTranslating,
    displayMode,
    triggerMode,
    immersiveEnabled,
    translationFallbackActive,
    translationError,
    hasPendingTranslations: candidates.length > 0,
    translateNow,
    renderDisplayText,
  };
}
