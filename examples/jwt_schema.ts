// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Main {
    type Flags = {
      currentJwt: string | null;
    };

    type Ports = {
      logout: {
        subscribe: (callback: (value: Record<string, never>) => void) => void;
      };
      newJwt: {
        subscribe: (callback: (value: string) => void) => void;
      };
    };

    function init(config: { flags: Flags; node: HTMLElement }): {
      ports: Ports;
    };
  }
}
