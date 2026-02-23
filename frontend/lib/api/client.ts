const BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function authFetch(path: string, init?: RequestInit) {
  const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(init?.headers as Record<string, string>) };
  const token = typeof window !== 'undefined' ? document.cookie.match(/sb-access-token=([^;]+)/)?.[1] : undefined;
  if (token) headers['Authorization'] = `Bearer ${token}`;
  return fetch(`${BASE}${path}`, { ...init, headers });
}

export const StreamClient = {
  optimize: (data: { url?: string; target_bitrate?: number; codec?: string }) =>
    authFetch('/api/v1/stream/optimize', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  ingest: (data: { source_url?: string; output_format?: string }) =>
    authFetch('/api/v1/stream/ingest', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  analyze: (data: { url?: string }) =>
    authFetch('/api/v1/stream/analyze', { method: 'POST', body: JSON.stringify(data) }).then(r => r.json()),
  formats: () =>
    authFetch('/api/v1/stream/formats').then(r => r.json()),
};
