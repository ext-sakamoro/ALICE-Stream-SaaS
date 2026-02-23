import Link from 'next/link';
export default function Home() {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b border-border">
        <div className="max-w-6xl mx-auto px-6 py-4 flex justify-between items-center">
          <h1 className="text-xl font-bold">ALICE Stream SaaS</h1>
          <div className="flex gap-3">
            <Link href="/auth/login" className="px-4 py-2 text-sm text-muted-foreground hover:text-foreground">Sign in</Link>
            <Link href="/auth/register" className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:opacity-90">Get Started</Link>
          </div>
        </div>
      </header>
      <main>
        <section className="max-w-4xl mx-auto px-6 py-24 text-center space-y-6">
          <h2 className="text-5xl font-bold tracking-tight">Don&apos;t push packets.<br />Push the law of streams.</h2>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">Adaptive streaming optimization powered by ALICE-Streaming-Protocol. Optimize, ingest, and analyze video streams with battle-tested protocols via a simple API.</p>
          <div className="flex gap-4 justify-center">
            <Link href="/auth/register" className="px-6 py-3 bg-primary text-primary-foreground rounded-md font-medium hover:opacity-90">Start Free</Link>
            <Link href="#features" className="px-6 py-3 border border-border rounded-md font-medium hover:bg-accent">Learn More</Link>
          </div>
        </section>
        <section id="features" className="max-w-5xl mx-auto px-6 py-16 grid grid-cols-1 md:grid-cols-3 gap-8">
          {[
            { t: 'Stream Optimization', d: 'H.264, H.265, AV1, VP9 and more. Adaptive bitrate with sub-50ms latency tuning.' },
            { t: 'Protocol Analysis', d: 'Inspect HLS, DASH, CMAF, WebRTC, SRT, and RTMP streams. Full metadata in milliseconds.' },
            { t: 'Multi-format Ingest', d: 'Accept any protocol, transcode on the fly, and deliver via CDN-ready endpoints.' },
          ].map((f) => (
            <div key={f.t} className="border border-border rounded-lg p-6 space-y-2">
              <h3 className="font-semibold text-lg">{f.t}</h3>
              <p className="text-sm text-muted-foreground">{f.d}</p>
            </div>
          ))}
        </section>
      </main>
      <footer className="border-t border-border py-8 text-center text-sm text-muted-foreground">AGPL-3.0 | ALICE Stream SaaS</footer>
    </div>
  );
}
