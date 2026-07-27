export const ticker = {
  callbacks: new Set<() => void>(),
  intervalId: null as number | null,

  subscribe(cb: () => void) {
    this.callbacks.add(cb);
    if (!this.intervalId) {
      this.intervalId = setInterval(() => {
        for (const fn of this.callbacks) fn();
      }, 1000) as unknown as number;
    }
    return () => this.unsubscribe(cb);
  },

  unsubscribe(cb: () => void) {
    this.callbacks.delete(cb);
    if (this.callbacks.size === 0 && this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
};
