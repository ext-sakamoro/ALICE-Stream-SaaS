import { create } from 'zustand';

interface StreamState {
  url: string;
  codec: string;
  targetBitrate: number;
  result: unknown | null;
  loading: boolean;
  setUrl: (u: string) => void;
  setCodec: (c: string) => void;
  setTargetBitrate: (b: number) => void;
  setResult: (r: unknown | null) => void;
  setLoading: (l: boolean) => void;
}

export const useStreamStore = create<StreamState>((set) => ({
  url: '',
  codec: 'h264',
  targetBitrate: 2_500_000,
  result: null,
  loading: false,
  setUrl: (url) => set({ url }),
  setCodec: (codec) => set({ codec }),
  setTargetBitrate: (targetBitrate) => set({ targetBitrate }),
  setResult: (result) => set({ result }),
  setLoading: (loading) => set({ loading }),
}));
