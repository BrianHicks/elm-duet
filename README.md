# elm-duet

Elm is great, and TypeScript is great, but the flags and ports between them are hard to use safely.
They're the only part of the a system between those two languages that aren't typed by default.

You can get around this in various ways, of course, either by maintaining a definitions by hand or generating one side from the other.
In general, though, you run into a couple different issues:

- It's easy for one side or the other to get out of date and errors to slip through CI and code review into production.
- Definitions in one language may not be translatable to the other (despite the two type systems having pretty good overlap.)

`elm-duet` tries to get around this by creating a single source of truth to generate both TypeScript definitions and Elm types with decoders.
We use [JSON Type Definitions](https://jsontypedef.com/) (JTD, [five-minute tutorial](https://jsontypedef.com/docs/jtd-in-5-minutes/)) to say precisely what we want and generate ergonomic types on both sides (plus helpers like encoders to make testing easy!)

Here's an example for an app that stores a [jwt](https://jwt.io/) in `localStorage` or similar to present to Elm:

```json {source=examples/jwt_schema.json}
{
  "modules": {
    "Main": {
      "flags": {
        "properties": {
          "currentJwt": {
            "type": "string",
            "nullable": true
          }
        }
      },
      "ports": {
        "newJwt": {
          "metadata": {
            "direction": "ElmToJs"
          },
          "type": "string"
        },
        "logout": {
          "metadata": {
            "direction": "ElmToJs"
          }
        }
      }
    }
  }
}

```

You can generate code from this by calling `elm-duet path/to/your/schema.json`:

```console
$ elm-duet examples/jwt_schema.json --typescript-dest examples/jwt_schema.ts
wrote examples/jwt_schema.ts
Ok(
    (
        Ref(
            InflectedString(
                "Flags",
            ),
        ),
        [
            TypeAlias {
                name: InflectedString(
                    "Flags",
                ),
                type_: Record(
                    {
                        InflectedString(
                            "currentJwt",
                        ): Maybe(
                            Scalar(
                                "String",
                            ),
                        ),
                    },
                ),
            },
        ],
    ),
)

```

Which results in this schema:

```typescript {source=examples/jwt_schema.ts}
// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Main {
    type Flags = {
      currentJwt: string | null;
    }
  
    type Ports = {
      logout: {
        subscribe: (callback: (value: Record<string, never>) => void) => void;
      };
      newJwt: {
        subscribe: (callback: (value: string) => void) => void;
      };
    }
  
    function init(config: {
      flags: Flags;
      node: HTMLElement;
    }): void
  }
}
```

Here are more things you can do with the tool:

```console
$ elm-duet --help
Generate Elm and TypeScript types from a single shared definition.

Usage: elm-duet [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Location of the definition file

Options:
      --typescript-dest <TYPESCRIPT_DEST>  Destination for TypeScript types [default: elm.ts]
      --elm-dest <ELM_DEST>                Destination for Elm types [default: src/]
      --elm-prefix <ELM_PREFIX>            Prefix for Elm module path [default: Interop]
  -h, --help                               Print help
  -V, --version                            Print version

```
