type Listener = (...args) => (boolean | void);

export class EventEmitter {
  private listeners: { [key: string]: Listener[] } = {};
  private marked_unused: string[] = [];


  mark_can_be_unused(event: string) {
    this.marked_unused.push(event);
  }

  on(event: string, listener: Listener): () => void {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event] = [listener, ...this.listeners[event]];

    return () => {
      this.listeners[event] = this.listeners[event].filter((x) => x !== listener);
    }
  }

  on_or(events: string[], listener: Listener): () => void {
    let unsubscribe_functions = [];

    for (let event of events) {
      unsubscribe_functions.push(this.on(event, (...args) => listener(event, ...args)));
    }

    return () => {
      for (let unsubscribe of unsubscribe_functions) {
        unsubscribe();
      }
    }
  }


  once_or(events: string[], listener: Listener) {
    let unsubscribe = this.on_or(events, (...args) => {
      unsubscribe();
      return listener(...args);
    });
  }

  once(event: string, listener: Listener) {
    let unsubscribe = this.on(event, (...args) => {
      unsubscribe();
      return listener(...args);
    });
  }

  emit(event: string, ...args: Parameters<Listener>): boolean {
    /// console.log(`Event emitted: ${event} ${args}`);
    if (this.listeners[event]) {
      for (let listener of this.listeners[event]) {
        let r = listener(...args);
        if (r === true) {
          return true;
        }
      }
    }
    if (!this.marked_unused.includes(event)) {
      console.log(
        `Unhandled Event emitted: ${event} ${args
          .map((x) =>
            JSON.stringify(x, (_, v) =>
              typeof v === "bigint" ? v.toString() : v,
            ),
          )
          .join(", ")}`,
      );
    }
    return false;
  }
}