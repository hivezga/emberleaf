/**
 * Static translations for Emberleaf
 * Supports English (en) and Spanish (es-MX)
 */

export type Language = "en" | "es";

export interface Translations {
  // Simple Home
  audioReady: string;
  checkMicrophone: string;
  speakNow: string;
  micDetected: string;
  micNotHeard: string;
  changeMicrophone: string;
  playTestSound: string;

  // Device Picker
  suggested: string;
  otherMicrophones: string;
  applyNow: string;
  saveForLater: string;
  defaultDevice: string;
  recentlyActive: string;
  quicklyReconnect: string;

  // Status messages
  micConnected: string;
  checkMicOrChange: string;
  anotherAppUsing: string;
  micUnplugged: string;

  // Advanced Mode
  advanced: string;
  processingParams: string;
  fullDeviceTables: string;
  diagnoseAudio: string;
  showLogs: string;
  micMonitor: string;
  testTone: string;
  experimental: string;

  // Actions
  apply: string;
  cancel: string;
  save: string;
  close: string;
  undo: string;

  // Common
  good: string;
  error: string;
  loading: string;
  success: string;

  // Device Loss (BE/FE-017)
  deviceLoss: {
    lost: string;
    fallbackOk: string;
    fallbackFailed: string;
    change: string;
    retry: string;
    stableId: string;
    monitorGuarded: string;
  };

  // Settings (BE/FE-017)
  settings: {
    rememberMonitor: string;
  };

  // Preflight (REL-001-UI)
  preflight: {
    title: string;
    checkMySetup: string;
    statusReady: string;
    statusWarning: string;
    statusError: string;
    canProceed: string;
    cannotProceed: string;
    continue: string;
    fixGuide: string;
    rerun: string;
    copyJson: string;
    advancedMode: string;
    checking: string;
    checkAudio: string;
    checkWebkit: string;
    checkPortal: string;
    checkMicAccess: string;
    detailsLabel: string;
    versionLabel: string;
    backendLabel: string;
    copiedToClipboard: string;
  };

  // Onboarding (ONB-001)
  onboarding: {
    // Step labels
    stepWelcome: string;
    stepPreflight: string;
    stepMicTest: string;
    stepVoice: string;
    stepReady: string;

    // Welcome step
    welcomeTitle: string;
    welcomeSubtitle: string;
    welcomeIntro: string;
    welcomePrivacy: string;
    getStarted: string;

    // Preflight step
    preflightTitle: string;
    preflightSubtitle: string;
    preflightWait: string;

    // Mic test step
    micTestTitle: string;
    micTestSubtitle: string;
    micTestInstruction: string;
    micTestHint: string;
    micTestNoSignal: string;
    micTestDetected: string;
    skipMicTest: string;

    // Voice enrollment step
    voiceTitle: string;
    voiceSubtitle: string;
    voiceOptional: string;
    voiceSkip: string;
    voiceEnroll: string;
    voiceComingSoon: string;

    // Ready step
    readyTitle: string;
    readySubtitle: string;
    readyMessage: string;
    openEmberleaf: string;

    // Navigation
    next: string;
    back: string;
    skip: string;
    finish: string;

    // Settings
    rerunOnboarding: string;
  };

  // Validation Errors (SEC-001B)
  errors: {
    invalid_device_name: string;
    invalid_frequency: string;
    invalid_duration: string;
    invalid_gain: string;
    invalid_threshold: string;
    invalid_sensitivity: string;
    permission_denied: string;
    device_busy: string;
    device_not_found: string;
    unknown_error: string;
  };

  // KWS (Keyword Spotting) - KWS-REAL-001-UI
  kws: {
    title: string;
    statusStub: string;
    statusReal: string;
    currentModel: string;
    selectModel: string;
    enable: string;
    disable: string;
    downloading: string;
    verifying: string;
    verified: string;
    verifyFailed: string;
    enabledToast: string;
    disabledToast: string;
    simpleCta: string;
    noModels: string;
    autoChoiceNote: string;
    modelLang: string;
    modelSize: string;
    downloadProgress: string;
    // QA-019: Test feature strings
    testButton: string;
    testing: string;
    testPass: string;
    testFailRetry: string;
    testHintSayNow: string;
    loopbackMissingHint: string;
  };
}

export const translations: Record<Language, Translations> = {
  en: {
    // Simple Home
    audioReady: "Audio Ready",
    checkMicrophone: "Check my microphone",
    speakNow: "Speak now — we should see movement",
    micDetected: "Microphone detected: {device} (Good)",
    micNotHeard: "We can't hear you — try 'Change microphone'",
    changeMicrophone: "Change microphone",
    playTestSound: "Play test sound",

    // Device Picker
    suggested: "Suggested",
    otherMicrophones: "Other microphones",
    applyNow: "Apply now",
    saveForLater: "Save for later",
    defaultDevice: "(Default)",
    recentlyActive: "(Recently active)",
    quicklyReconnect: "We'll quickly reconnect your mic.",

    // Status messages
    micConnected: "Microphone connected: {device}",
    checkMicOrChange: "We can't hear you. Check your microphone or change device.",
    anotherAppUsing: "Another app is using this mic.",
    micUnplugged: "The selected mic was unplugged.",

    // Advanced Mode
    advanced: "Advanced",
    processingParams: "Processing Parameters",
    fullDeviceTables: "Full Device Tables",
    diagnoseAudio: "Diagnose Audio",
    showLogs: "Show Logs",
    micMonitor: "Mic Monitor",
    testTone: "Test Tone",
    experimental: "Experimental",

    // Actions
    apply: "Apply",
    cancel: "Cancel",
    save: "Save",
    close: "Close",
    undo: "Undo",

    // Common
    good: "Good",
    error: "Error",
    loading: "Loading...",
    success: "Success",

    // Device Loss (BE/FE-017)
    deviceLoss: {
      lost: '"{name}" was disconnected.',
      fallbackOk: "Switched to the default device.",
      fallbackFailed: "We couldn't switch automatically.",
      change: "Change…",
      retry: "Retry",
      stableId: "Stable ID: {id}",
      monitorGuarded: "Monitoring wasn't resumed to avoid feedback.",
    },

    // Settings (BE/FE-017)
    settings: {
      rememberMonitor: "Remember mic monitoring",
    },

    // Preflight (REL-001-UI)
    preflight: {
      title: "System Check",
      checkMySetup: "Check my setup",
      statusReady: "Ready",
      statusWarning: "Warning",
      statusError: "Error",
      canProceed: "Your system is ready to use Emberleaf.",
      cannotProceed: "Please fix the issues below before continuing.",
      continue: "Continue",
      fixGuide: "Fix guide",
      rerun: "Re-run check",
      copyJson: "Copy JSON",
      advancedMode: "Advanced",
      checking: "Checking your system...",
      checkAudio: "Audio Stack",
      checkWebkit: "WebKit2GTK",
      checkPortal: "Desktop Portal",
      checkMicAccess: "Microphone Access",
      detailsLabel: "Details",
      versionLabel: "Version",
      backendLabel: "Backend",
      copiedToClipboard: "Copied to clipboard",
    },

    // Onboarding (ONB-001)
    onboarding: {
      // Step labels
      stepWelcome: "Welcome",
      stepPreflight: "System Check",
      stepMicTest: "Microphone Test",
      stepVoice: "Voice Setup",
      stepReady: "Ready",

      // Welcome step
      welcomeTitle: "Welcome to Emberleaf",
      welcomeSubtitle: "Your Private Voice Assistant",
      welcomeIntro: "Emberleaf runs entirely on your device. Your voice never leaves your computer, ensuring complete privacy and security.",
      welcomePrivacy: "No cloud services. No data collection. Just you and your assistant.",
      getStarted: "Get Started",

      // Preflight step
      preflightTitle: "Checking Your System",
      preflightSubtitle: "We're making sure everything is ready",
      preflightWait: "This will only take a moment...",

      // Mic test step
      micTestTitle: "Test Your Microphone",
      micTestSubtitle: "Let's make sure we can hear you",
      micTestInstruction: "Speak now — you should see the meter move",
      micTestHint: "Try saying 'Hello' or 'Testing'",
      micTestNoSignal: "No signal detected yet. Check your microphone connection or try changing devices.",
      micTestDetected: "Great! We can hear you.",
      skipMicTest: "Skip this step",

      // Voice enrollment step
      voiceTitle: "Voice Recognition (Optional)",
      voiceSubtitle: "Teach Emberleaf to recognize your voice",
      voiceOptional: "This step is optional. You can set it up later from settings.",
      voiceSkip: "Skip for now",
      voiceEnroll: "Set up voice recognition",
      voiceComingSoon: "Voice enrollment will be available in a future update.",

      // Ready step
      readyTitle: "You're All Set!",
      readySubtitle: "Emberleaf is ready to use",
      readyMessage: "Your microphone is working and your system is configured. You can start using Emberleaf right away.",
      openEmberleaf: "Open Emberleaf",

      // Navigation
      next: "Next",
      back: "Back",
      skip: "Skip",
      finish: "Finish",

      // Settings
      rerunOnboarding: "Re-run onboarding",
    },

    // Validation Errors (SEC-001B)
    errors: {
      invalid_device_name: "Device name is invalid.",
      invalid_frequency: "Frequency out of range (50–4000 Hz).",
      invalid_duration: "Duration out of range (10–5000 ms).",
      invalid_gain: "Gain out of range (0.0–0.5).",
      invalid_threshold: "VAD threshold out of range (0.0–1.0).",
      invalid_sensitivity: "Sensitivity out of range (0.0–1.0).",
      permission_denied: "Permission denied. Check your system settings.",
      device_busy: "Another application is using this device.",
      device_not_found: "Selected device was not found.",
      unknown_error: "An unexpected error occurred.",
    },

    // KWS (Keyword Spotting) - KWS-REAL-001-UI
    kws: {
      title: "Wake Word (Real KWS)",
      statusStub: "Stub mode",
      statusReal: "Real KWS",
      currentModel: "Model",
      selectModel: "Select a model",
      enable: "Enable Real KWS",
      disable: "Disable",
      downloading: "Downloading…",
      verifying: "Verifying…",
      verified: "Model verified",
      verifyFailed: "Model verification failed",
      enabledToast: "Real KWS enabled",
      disabledToast: "Real KWS disabled",
      simpleCta: "Enable Real Recognition",
      noModels: "No models available",
      autoChoiceNote: "Auto-selecting recommended model",
      modelLang: "Language",
      modelSize: "Size",
      downloadProgress: "Downloading: {percent}%",
      // QA-019: Test feature strings
      testButton: "Test wake word",
      testing: "Testing…",
      testPass: "✓ Test passed",
      testFailRetry: "No wake word detected. Try again.",
      testHintSayNow: "Say 'Hey Ember' now",
      loopbackMissingHint: "Audio loopback not available. Speak the wake word when prompted.",
    },
  },
  es: {
    // Simple Home
    audioReady: "Audio listo",
    checkMicrophone: "Verificar micrófono",
    speakNow: "Habla ahora — deberíamos ver movimiento",
    micDetected: "Micrófono detectado: {device} (Bien)",
    micNotHeard: "No te escuchamos — prueba 'Cambiar micrófono'",
    changeMicrophone: "Cambiar micrófono",
    playTestSound: "Probar sonido",

    // Device Picker
    suggested: "Sugeridos",
    otherMicrophones: "Otros micrófonos",
    applyNow: "Aplicar ahora",
    saveForLater: "Guardar para después",
    defaultDevice: "(Predeterminado)",
    recentlyActive: "(Activo recientemente)",
    quicklyReconnect: "Reconectaremos tu micrófono rápidamente.",

    // Status messages
    micConnected: "Micrófono conectado: {device}",
    checkMicOrChange: "No te escuchamos. Revisa el micrófono o cambia de dispositivo.",
    anotherAppUsing: "Otra app está usando este micrófono.",
    micUnplugged: "El micrófono seleccionado fue desconectado.",

    // Advanced Mode
    advanced: "Avanzado",
    processingParams: "Parámetros de procesamiento",
    fullDeviceTables: "Tablas completas de dispositivos",
    diagnoseAudio: "Diagnosticar audio",
    showLogs: "Mostrar registros",
    micMonitor: "Monitor de micrófono",
    testTone: "Tono de prueba",
    experimental: "Experimental",

    // Actions
    apply: "Aplicar",
    cancel: "Cancelar",
    save: "Guardar",
    close: "Cerrar",
    undo: "Deshacer",

    // Common
    good: "Bien",
    error: "Error",
    loading: "Cargando...",
    success: "Éxito",

    // Device Loss (BE/FE-017)
    deviceLoss: {
      lost: 'Se desconectó "{name}".',
      fallbackOk: "Cambiamos al dispositivo predeterminado.",
      fallbackFailed: "No pudimos cambiar automáticamente.",
      change: "Cambiar…",
      retry: "Reintentar",
      stableId: "ID estable: {id}",
      monitorGuarded: "No reanudamos el monitoreo para evitar retroalimentación.",
    },

    // Settings (BE/FE-017)
    settings: {
      rememberMonitor: "Recordar monitoreo del micrófono",
    },

    // Preflight (REL-001-UI)
    preflight: {
      title: "Comprobación del sistema",
      checkMySetup: "Revisar mi configuración",
      statusReady: "Listo",
      statusWarning: "Advertencia",
      statusError: "Error",
      canProceed: "Tu sistema está listo para usar Emberleaf.",
      cannotProceed: "Por favor, soluciona los problemas abajo antes de continuar.",
      continue: "Continuar",
      fixGuide: "Guía de solución",
      rerun: "Volver a comprobar",
      copyJson: "Copiar JSON",
      advancedMode: "Avanzado",
      checking: "Comprobando tu sistema...",
      checkAudio: "Sistema de audio",
      checkWebkit: "WebKit2GTK",
      checkPortal: "Portal de escritorio",
      checkMicAccess: "Acceso al micrófono",
      detailsLabel: "Detalles",
      versionLabel: "Versión",
      backendLabel: "Backend",
      copiedToClipboard: "Copiado al portapapeles",
    },

    // Onboarding (ONB-001)
    onboarding: {
      // Step labels
      stepWelcome: "Bienvenida",
      stepPreflight: "Comprobación",
      stepMicTest: "Prueba de Micrófono",
      stepVoice: "Configurar Voz",
      stepReady: "Listo",

      // Welcome step
      welcomeTitle: "Bienvenido a Emberleaf",
      welcomeSubtitle: "Tu Asistente de Voz Privado",
      welcomeIntro: "Emberleaf se ejecuta completamente en tu dispositivo. Tu voz nunca sale de tu computadora, garantizando privacidad y seguridad total.",
      welcomePrivacy: "Sin servicios en la nube. Sin recopilación de datos. Solo tú y tu asistente.",
      getStarted: "Comenzar",

      // Preflight step
      preflightTitle: "Comprobando tu Sistema",
      preflightSubtitle: "Nos aseguramos de que todo esté listo",
      preflightWait: "Esto solo tomará un momento...",

      // Mic test step
      micTestTitle: "Prueba tu Micrófono",
      micTestSubtitle: "Asegurémonos de que podemos escucharte",
      micTestInstruction: "Habla ahora — deberías ver moverse el medidor",
      micTestHint: "Intenta decir 'Hola' o 'Probando'",
      micTestNoSignal: "Aún no se detecta señal. Revisa tu micrófono o intenta cambiar de dispositivo.",
      micTestDetected: "¡Excelente! Podemos escucharte.",
      skipMicTest: "Omitir este paso",

      // Voice enrollment step
      voiceTitle: "Reconocimiento de Voz (Opcional)",
      voiceSubtitle: "Enseña a Emberleaf a reconocer tu voz",
      voiceOptional: "Este paso es opcional. Puedes configurarlo más tarde desde ajustes.",
      voiceSkip: "Omitir por ahora",
      voiceEnroll: "Configurar reconocimiento de voz",
      voiceComingSoon: "La inscripción de voz estará disponible en una actualización futura.",

      // Ready step
      readyTitle: "¡Todo Listo!",
      readySubtitle: "Emberleaf está listo para usar",
      readyMessage: "Tu micrófono funciona y tu sistema está configurado. Puedes empezar a usar Emberleaf ahora mismo.",
      openEmberleaf: "Abrir Emberleaf",

      // Navigation
      next: "Siguiente",
      back: "Atrás",
      skip: "Omitir",
      finish: "Finalizar",

      // Settings
      rerunOnboarding: "Volver a ejecutar configuración inicial",
    },

    // Validation Errors (SEC-001B)
    errors: {
      invalid_device_name: "El nombre del dispositivo no es válido.",
      invalid_frequency: "Frecuencia fuera de rango (50–4000 Hz).",
      invalid_duration: "Duración fuera de rango (10–5000 ms).",
      invalid_gain: "Ganancia fuera de rango (0.0–0.5).",
      invalid_threshold: "Umbral VAD fuera de rango (0.0–1.0).",
      invalid_sensitivity: "Sensibilidad fuera de rango (0.0–1.0).",
      permission_denied: "Permiso denegado. Revisa la configuración del sistema.",
      device_busy: "Otro programa está usando este dispositivo.",
      device_not_found: "No se encontró el dispositivo seleccionado.",
      unknown_error: "Ocurrió un error inesperado.",
    },

    // KWS (Keyword Spotting) - KWS-REAL-001-UI
    kws: {
      title: "Palabra Clave (KWS Real)",
      statusStub: "Modo simulado",
      statusReal: "KWS real",
      currentModel: "Modelo",
      selectModel: "Selecciona un modelo",
      enable: "Activar KWS Real",
      disable: "Desactivar",
      downloading: "Descargando…",
      verifying: "Verificando…",
      verified: "Modelo verificado",
      verifyFailed: "Falló la verificación del modelo",
      enabledToast: "KWS real activado",
      disabledToast: "KWS real desactivado",
      simpleCta: "Activar Reconocimiento Real",
      noModels: "No hay modelos disponibles",
      autoChoiceNote: "Seleccionando automáticamente el modelo recomendado",
      modelLang: "Idioma",
      modelSize: "Tamaño",
      downloadProgress: "Descargando: {percent}%",
      // QA-019: Test feature strings
      testButton: "Probar palabra clave",
      testing: "Probando…",
      testPass: "✓ Prueba exitosa",
      testFailRetry: "No se detectó la palabra clave. Intenta de nuevo.",
      testHintSayNow: "Di 'Hey Ember' ahora",
      loopbackMissingHint: "Bucle de audio no disponible. Pronuncia la palabra clave cuando se indique.",
    },
  },
};

/**
 * Format a translation string with placeholders
 * Example: format("Microphone detected: {device}", { device: "USB Mic" })
 */
export function format(template: string, values: Record<string, string>): string {
  return Object.entries(values).reduce(
    (result, [key, value]) => result.replace(`{${key}}`, value),
    template
  );
}
