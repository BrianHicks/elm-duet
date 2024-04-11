declare namespace Elm {
  class Main {
    static init({
      node: Element,
      flags: { currentTimeMillis: number, notificationPermission: string },
    }): Main;

    ports: {
      changeDocument: { subscribe(callback: (value: unknown) => void): void };
      docFromAutomerge: { send(value: unknown): void };
      gotNewNotificationsPermission: { send(value: string): void };
      requestNotificationsPermission: {
        subscribe(callback: (value: null) => void): void;
      };
      sendNotification: { subscribe(callback: (value: unknown) => void): void };
    };
  }
}
