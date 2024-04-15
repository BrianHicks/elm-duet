// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Foo {
    namespace Bar {
      namespace Main {
        type Flags = {
          currentTimeMillis: number;
          notificationPermission: "default" | "denied" | "granted";
        }
        type Ports = {
          notificationPermission: {
            send: (value: "default" | "denied" | "granted") => void;
          };
          requestNotificationPermission: {
            subscribe: (callback: (value: Record<string, never>) => void) => void;
          };
          sendNotification: {
            subscribe: (callback: (value: {
              options: {
                badge: string;
                body: string;
                icon: string;
                lang: string;
                requireInteraction: bool;
                silent: bool;
                tag: string;
              };
              title: string;
            }) => void) => void;
          };
        }
        function init(config: {
          flags: Flags;
          node: HTMLElement;
        }): void
      }
    }
  }
}