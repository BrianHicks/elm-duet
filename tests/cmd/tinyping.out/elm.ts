// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Main {
    type Flags = {
      currentTimeMillis: number;
      notificationPermission: "default" | "denied" | "granted";
    };

    type Ports = {
      changeDocument: {
        subscribe: (
          callback: (
            value:
              | {
                  tag: "AddNewPingAt";
                  value: number;
                }
              | {
                  tag: "SetMinutesPerPing";
                  value: number;
                }
              | {
                  index: number;
                  tag: "SetTagForPing";
                  value: string | null;
                },
          ) => void,
        ) => void;
      };
      docFromAutomerge: {
        send: (value: {
          pings: {
            custom: Record<string, string>;
            tag: string | null;
            time: number;
            version: "v1";
          }[];
          settings: {
            minutesPerPing: number;
            version: "v1";
          };
          version: "v1";
        }) => void;
      };
      newNotification: {
        subscribe: (
          callback: (value: {
            options: {
              badge: string | null;
              body: string | null;
              icon: string | null;
              lang: string | null;
              requireInteraction: boolean | null;
              silent: boolean | null;
              tag: string | null;
            };
            title: string;
          }) => void,
        ) => void;
      };
      notificationPermission: {
        send: (value: "default" | "denied" | "granted") => void;
      };
      requestNotificationPermission: {
        subscribe: (callback: (value: Record<string, never>) => void) => void;
      };
    };

    function init(config: { flags: Flags; node: HTMLElement }): {
      ports: Ports;
    };
  }
}
