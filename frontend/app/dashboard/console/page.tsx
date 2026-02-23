'use client';
import { useState } from 'react';
export default function ConsolePage() {
  const [url, setUrl] = useState('');
  const [codec, setCodec] = useState('h264');
  const [targetBitrate, setTargetBitrate] = useState(2500000);
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleOptimize = async () => {
    setLoading(true);
    try {
      const r = await fetch(`${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/stream/optimize`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url, codec, target_bitrate: targetBitrate }),
      });
      const data = await r.json();
      setResult(JSON.stringify(data, null, 2));
    } catch (e) { setResult(`Error: ${e}`); }
    finally { setLoading(false); }
  };

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Stream Console</h1>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="border border-border rounded-lg p-4 space-y-4">
          <h2 className="font-semibold">Optimize Stream</h2>
          <div>
            <label className="text-sm font-medium">Stream URL</label>
            <input
              type="text"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="rtmp://your-source/live/stream"
              className="mt-1 w-full px-3 py-2 border border-input rounded-md bg-background text-sm"
            />
          </div>
          <div>
            <label className="text-sm font-medium">Codec</label>
            <select value={codec} onChange={(e) => setCodec(e.target.value)} className="mt-1 w-full px-3 py-2 border border-input rounded-md bg-background text-sm">
              <option value="h264">H.264</option>
              <option value="h265">H.265</option>
              <option value="av1">AV1</option>
              <option value="vp9">VP9</option>
            </select>
          </div>
          <div>
            <label className="text-sm font-medium">Target Bitrate: {(targetBitrate / 1_000_000).toFixed(1)} Mbps</label>
            <input
              type="range"
              min={500000}
              max={20000000}
              step={500000}
              value={targetBitrate}
              onChange={(e) => setTargetBitrate(Number(e.target.value))}
              className="mt-1 w-full"
            />
          </div>
          <button
            onClick={handleOptimize}
            disabled={loading}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:opacity-90 disabled:opacity-50"
          >
            {loading ? 'Optimizing...' : 'Optimize'}
          </button>
        </div>
        <div className="border border-border rounded-lg p-4 space-y-2">
          <h2 className="font-semibold">Result</h2>
          <pre className="bg-muted rounded-md p-3 text-xs font-mono overflow-auto max-h-64">{result || 'No result yet'}</pre>
        </div>
      </div>
    </div>
  );
}
