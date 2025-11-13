import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Toaster } from "sonner";
import { AudioSettings } from "./components/AudioSettings";
import "./App.css";

interface WakeWordEvent {
  keyword: string;
  score: number;
}

function App() {
  const [detections, setDetections] = useState<{ keyword: string; score: number; time: string }[]>([]);

  useEffect(() => {
    const unlisten = listen<WakeWordEvent>("wakeword::detected", (event) => {
      const detection = {
        ...event.payload,
        time: new Date().toLocaleTimeString(),
      };
      setDetections((prev) => [detection, ...prev].slice(0, 10));
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <>
      <Toaster position="top-right" richColors />
      <div className="container max-w-4xl mx-auto p-6 space-y-6">
        <div className="space-y-2">
          <h1 className="text-4xl font-bold">Emberleaf Voice Assistant</h1>
          <p className="text-muted-foreground">
            Listening for: <strong className="text-accent">"Hey Ember"</strong>
          </p>
        </div>

        <AudioSettings />

        {detections.length > 0 && (
          <div className="space-y-3">
            <h2 className="text-2xl font-semibold">Recent Detections</h2>
            <div className="space-y-2">
              {detections.map((d, i) => (
                <div
                  key={i}
                  className="flex items-center gap-4 p-3 rounded-lg bg-card border"
                >
                  <span className="text-sm text-muted-foreground min-w-[80px]">
                    {d.time}
                  </span>
                  <span className="font-medium">{d.keyword}</span>
                  <span className="ml-auto text-sm text-accent">
                    Score: {d.score.toFixed(3)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </>
  );
}

export default App;
