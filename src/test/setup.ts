import { vi } from 'vitest';

// Setup localStorage mock for jsdom
class LocalStorageMock {
  private store: Map<string, string> = new Map();

  getItem(key: string): string | null {
    return this.store.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.store.set(key, value);
  }

  removeItem(key: string): void {
    this.store.delete(key);
  }

  clear(): void {
    this.store.clear();
  }

  key(index: number): string | null {
    return Array.from(this.store.keys())[index] ?? null;
  }

  get length(): number {
    return this.store.size;
  }
}

// @ts-ignore - global is available in test environment
(globalThis as any).localStorage = new LocalStorageMock();

// Mock Tauri APIs for testing environment
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_evt: string, _cb: any) => ({ unlisten: () => {} })),
  emit: vi.fn(),
  once: vi.fn(async (_evt: string, _cb: any) => ({ unlisten: () => {} })),
}));

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(async (_cmd: string, _args?: any) => null),
}));

vi.mock('@tauri-apps/api/path', () => ({
  appDataDir: vi.fn(async () => '/tmp'),
  appConfigDir: vi.fn(async () => '/tmp/config'),
}));

// Quiet console noise during tests
const originalError = console.error;
console.error = (...args: any[]) => {
  if (/(tauri|webkit|audio)/i.test(String(args[0] ?? ''))) return;
  originalError(...args);
};
