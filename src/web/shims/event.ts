export type UnlistenFn = () => void;
export type EventCallback<T> = (event: { event: string; payload: T }) => void;

export async function listen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  const es = new EventSource('/api/events');
  es.onmessage = (e) => {
    try {
      const msg = JSON.parse(e.data);
      if (msg.event === event) {
        handler({ event, payload: msg.payload });
      }
    } catch (_) {}
  };
  return () => es.close();
}

export async function once<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  const unlisten = await listen<T>(event, (e) => {
    handler(e);
    unlisten();
  });
  return unlisten;
}
