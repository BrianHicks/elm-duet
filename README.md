# elm-duet

I like Elm and TypeScript for building apps, but I find it annoying to make the boundary between them type-safe.

I can get around this in various ways, of course, either by maintaining a definitions by hand or generating one side from the other.
In general, though, you run into a couple different issues:

- It's easy for one side or the other to get out of date and errors to slip through CI and code review into production.
- Definitions in one language may not be translatable to the other (despite the two type systems having a high degree of overlap.)

`elm-duet` tries to get around this by creating a single source of truth to generate both TypeScript definitions and Elm types with decoders.
We use [JSON Type Definitions](https://jsontypedef.com/) (JTD, [five-minute tutorial](https://jsontypedef.com/docs/jtd-in-5-minutes/)) to say precisely what we want and generate ergonomic types on both sides (plus helpers like encoders to make testing easy!)

Here's an example for an app that stores a [jwt](https://jwt.io/) in `localStorage` or similar to present to Elm:

```json {source=examples/jwt_schema.yaml}
modules:
  Main:
    flags:
      properties:
        currentJwt:
          type: string
          nullable: true
    ports:
      newJwt:
        metadata:
          direction: ElmToJs
        type: string
      logout:
        metadata:
          direction: ElmToJs

```

(We're using YAML in this example so we can use comments, but JSON schemas also work just fine.)

You can generate code from this by calling `elm-duet path/to/your/schema.(yaml|json)`:

```console
$ elm-duet examples/jwt_schema.yaml --typescript-dest examples/jwt_schema.ts --elm-dest examples/jwt_schema
wrote examples/jwt_schema.ts
wrote examples/jwt_schema/Main/Flags.elm
wrote examples/jwt_schema/Main/Ports.elm

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
    }): {
      ports: Ports;
    }
  }
}
```

And these Elm flags:

```elm {source=examples/jwt_schema/Main/Flags.elm}
module Main.Flags exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias Flags =
    { currentJwt : Maybe String
    }


flagsDecoder : Decoder Flags
flagsDecoder =
    Json.Decode.succeed Flags
        |> Json.Decode.Pipeline.required "currentJwt" (Json.Decode.nullable Json.Decode.string)


encodeFlags : Flags -> Json.Encode.Value
encodeFlags flags =
    Json.Encode.object
        [ ( "currentJwt"
          , case flags.currentJwt of
                Just value ->
                    Json.Encode.string value

                Nothing ->
                    Json.Encode.null
          )
        ]

```

And these for the ports:

```elm {source=examples/jwt_schema/Main/Ports.elm}
port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias Logout =
    ()


logoutDecoder : Decoder Logout
logoutDecoder =
    Json.Decode.null ()


encodeLogout : Logout -> Json.Encode.Value
encodeLogout logout =
    Json.Encode.null


type alias NewJwt =
    String


newJwtDecoder : Decoder NewJwt
newJwtDecoder =
    Json.Decode.string


encodeNewJwt : NewJwt -> Json.Encode.Value
encodeNewJwt newJwt =
    Json.Encode.string newJwt


port logout : Value -> Cmd msg


sendLogout : Logout -> Cmd msg
sendLogout value =
    logout (encodeLogout value)


port newJwt : Value -> Cmd msg


sendNewJwt : NewJwt -> Cmd msg
sendNewJwt value =
    newJwt (encodeNewJwt value)

```

Here's the full help to give you an idea of what you can do with the tool:

```console
$ elm-duet --help
Generate Elm and TypeScript types from a single shared definition.

Usage: elm-duet [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Location of the definition file

Options:
      --typescript-dest <TYPESCRIPT_DEST>  Destination for TypeScript types [default: elm.ts]
      --elm-dest <ELM_DEST>                Destination for Elm types [default: src/]
  -h, --help                               Print help
  -V, --version                            Print version

```
