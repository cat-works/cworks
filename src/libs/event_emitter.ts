type Listener = (...args) => (boolean | void);

export class EventEmitter {
  private listeners: { [key: string]: Listener[] } = {};

  on(event: string, listener: Listener): () => void {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(listener);

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

  emit(event: string, ...args: Parameters<Listener>) {
    if (this.listeners[event]) {
      for (let listener of this.listeners[event]) {
        if (listener(...args)) {
          break;
        }
      }
    }
  }
}