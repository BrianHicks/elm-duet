// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Main {
    type Flags = Record<string, never>;

    type Ports = {
      elmToJs?: {
        subscribe: (callback: (value: { a: string }) => void) => void;
      };
      jsToElm?: {
        send: (value: { a: string }) => void;
      };
    };

    function init(config: { flags: Flags; node: HTMLElement }): {
      ports?: Ports;
    };
  }
}
