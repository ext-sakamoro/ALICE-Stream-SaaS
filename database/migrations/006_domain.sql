-- Stream SaaS domain tables
create table if not exists public.streams (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    stream_key text unique not null,
    title text,
    status text not null default 'idle' check (status in ('idle', 'live', 'ended', 'error')),
    protocol text not null default 'hls',
    codec text default 'h264',
    resolution text,
    bitrate_kbps integer,
    ingest_url text,
    playback_url text,
    viewer_count integer default 0,
    started_at timestamptz,
    ended_at timestamptz,
    created_at timestamptz default now()
);
create table if not exists public.transcode_jobs (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    stream_id uuid references public.streams(id),
    input_codec text not null,
    output_codec text not null,
    input_resolution text,
    output_resolution text,
    status text not null default 'pending',
    created_at timestamptz default now(),
    completed_at timestamptz
);
create index idx_streams_user on public.streams(user_id);
create index idx_streams_status on public.streams(status);
create index idx_streams_key on public.streams(stream_key);
