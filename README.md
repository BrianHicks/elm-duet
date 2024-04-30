# elm-duet

I like Elm and TypeScript for building apps, but I find it annoying to make the boundary between them type-safe.

I can get around this in various ways, of course, either by maintaining a definitions by hand or generating one side from the other.
In general, though, you run into a couple different issues:

- It's easy for one side or the other to get out of date and errors to slip through CI and code review into production.
- Definitions in one language may not be translatable to the other (despite the two type systems having a high degree of overlap.)

`elm-duet` tries to get around this by creating a single source of truth to generate both TypeScript definitions and Elm types with decoders.
We use [JSON Type Definitions](https://jsontypedef.com/) (JTD, [five-minute tutorial](https://jsontypedef.com/docs/jtd-in-5-minutes/)) to say precisely what we want and generate ergonomic types on both sides (plus helpers like encoders to make testing easy!)

In addition, `elm-duet` produces files that make following good practices around interop easier!
I'll call those out as we get to them.

## Example 1: JWTs

Here's an example for an app that stores [JWTs](https://jwt.io/) in `localStorage`:

```yaml {source=examples/jwt_schema.yaml}
# An example app that uses JWTs to manage authentication. Imagine that the JWTs
# are stored in localStorage so that they can persist across sessions. The
# lifecycle of this app might look like:
#
# 1. On init, the JS side passes the Elm runtime the current value of the JWT
#    (or `null`, if unset)
# 2. Elm is responsible for authentication and for telling JS when it gets a
#    new JWT (for example, when the user logs in)
# 3. Elm is also responsible for telling JS if someone logs out, in which case
#    we should clear the JWT from localStorage.

# For elm-duet, we start by defining our data types. In this case, we're just
# going to keep things simple and define a "jwt" that will just be an alias to
# a string.
definitions:
  jwt:
    type: string

# Now we say how to use it. Each key in this object is a module in your Elm
# application, in which we can define our flags and ports.
modules:
  Main:
    # First we'll define flags. As we mentioned above, that's either a JWT or
    # null. We'll define it by referring to
    #
    # If your app doesn't use flags, you can omit this key.
    flags:
      properties:
        currentJwt:
          ref: jwt
          nullable: true

    # Next we'll do ports. As with flags, if your app doesn't use ports, you
    # can omit this key.
    ports:
      # Like flags, ports are specified with a JTD schema. In this case, we
      # want a non-nullable version of the same JWT as earlier.
      #
      # Ports also have a direction. We specify this in the
      # `metadata.direction` key, either `ElmToJs` or `JsToElm`.
      newJwt:
        metadata:
          direction: ElmToJs
        ref: jwt

      # Finally, logout is a bit odd: we really don't need a payload. JTD
      # handles this by defining an "empty" case, which we specify by omitting
      # any type information. We still need the direction metadata, though!
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
formatted TypeScript
formatted Elm

```

This produces this TypeScript file:

```typescript {source=examples/jwt_schema.ts}
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

```

This should be flexible enough to use both if you're embedding your Elm app (e.g. with `esbuild`) or referring to it as an external JS file.

We also get this file containing Elm flags:

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

In your `init`, you can accept a `Json.Decode.Value` and call `Decode.decodeValue Main.Flags.flagsDecoder flags` to get complete control over the error experience.
It also lets you custom types in your flags, since you're specifying the decoder.

Note that `elm-duet` creates both decoders and encoders for all the types it generates.
This is to make your life easier during testing: you can hook up tools like [elm-program-test](https://package.elm-lang.org/packages/avh4/elm-program-test/latest/) without having to redo your data encoding.

Finally, we have the ports:

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

You'll notice that in addition to decoders and encoders, `elm-duet` generates type-safe wrappers around the ports.
This is, again, to let you send custom types through the ports in a way we control: if you specify an `enum`, for example, we ensure that both Elm and TypeScript have enough information to take advantage of the best parts of their respective type systems without falling back to plain strings.

## Example 2: An All-in-One Port

Some people like to define an all-in-one port for their application to make sure that they only have a single place to hook up new messages.
`elm-duet` and JDT support this with discriminators and mappings:

```yaml {source=examples/all_in_one.yaml}
# We're going to define a port named `toWorld` that sends all our messages to
# the JS in the same place. You can do the same thing for `fromWorld` for
# subscriptions, but we're leaving that off to keep things succinct.
modules:
  Main:
    ports:
      toWorld:
        metadata:
          direction: ElmToJs

        # JTD lets us distinguish between different types of an object by using
        # the discriminator/mapping case. The first step is to define the field
        # that "tags" the message. In this case, we'll literally use `tag`:
        discriminator: tag

        # Next, tell JTD all the possible values of `tag` and what types are
        # associated with them. We'll use the same option as in the JWT schema
        # example, but all in one port. Note that all the options need to be
        # non-nullable objects, since we can't set a tag otherwise.
        mapping:
          newJwt:
            properties:
              value:
                ref: jwt

          logout:
            # in cases where we don't want any payload, we specify an empty
            # object aside from the tag.
            properties:

definitions:
  jwt:
    type: string

```

Again, we generate everything in `examples`:

```console
$ elm-duet examples/all_in_one.yaml --typescript-dest examples/all_in_one.ts --elm-dest examples/all_in_one
wrote examples/all_in_one.ts
wrote examples/all_in_one/Main/Ports.elm
formatted TypeScript
formatted Elm

```

We get this in TypeScript:

```typescript {source=examples/all_in_one.ts}
// Warning: this file is automatically generated. Don't edit by hand!

declare module Elm {
  namespace Main {
    type Flags = Record<string, never>;

    type Ports = {
      toWorld: {
        subscribe: (
          callback: (
            value:
              | {
                  tag: "logout";
                }
              | {
                  tag: "newJwt";
                  value: string;
                },
          ) => void,
        ) => void;
      };
    };

    function init(config: { flags: Flags; node: HTMLElement }): {
      ports: Ports;
    };
  }
}

```

And this Elm:

```elm {source=examples/all_in_one/Main/Ports.elm}
port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias TagLogout =
    {}


tagLogoutDecoder : Decoder TagLogout
tagLogoutDecoder =
    Json.Decode.succeed TagLogout


encodeTagLogout : TagLogout -> Json.Encode.Value
encodeTagLogout tagLogout =
    Json.Encode.object
        [ ( "tag", Json.Encode.string "logout" )
        ]


type alias NewJwt =
    { value : String
    }


newJwtDecoder : Decoder NewJwt
newJwtDecoder =
    Json.Decode.succeed NewJwt
        |> Json.Decode.Pipeline.required "value" Json.Decode.string


encodeNewJwt : NewJwt -> Json.Encode.Value
encodeNewJwt newJwt =
    Json.Encode.object
        [ ( "value", Json.Encode.string newJwt.value )
        , ( "tag", Json.Encode.string "newJwt" )
        ]


type ToWorld
    = Logout TagLogout
    | NewJwt NewJwt


toWorldDecoder : Decoder ToWorld
toWorldDecoder =
    Json.Decode.andThen
        (\tag ->
            case tag of
                "logout" ->
                    Json.Decode.map Logout tagLogoutDecoder

                "newJwt" ->
                    Json.Decode.map NewJwt newJwtDecoder
        )
        (Json.Decode.field "tag" Json.Decode.string)


encodeToWorld : ToWorld -> Json.Encode.Value
encodeToWorld toWorld =
    case toWorld of
        Logout logout ->
            encodeTagLogout logout

        NewJwt newJwt ->
            encodeNewJwt newJwt


port toWorld : Value -> Cmd msg


sendToWorld : ToWorld -> Cmd msg
sendToWorld value =
    toWorld (encodeToWorld value)

```

## The Full Help

Here's the full help to give you an idea of what you can do with the tool:

```console
$ elm-duet --help
Generate Elm and TypeScript types from a single shared definition.

Usage: elm-duet [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Location of the definition file

Options:
      --typescript-dest <TYPESCRIPT_DEST>
          Destination for TypeScript types [default: elm.ts]
      --elm-dest <ELM_DEST>
          Destination for Elm types [default: src/]
      --no-format
          Turn off automatic formatting discovery
      --ts-formatter <TS_FORMATTER>
          What formatter should I use for TypeScript? (Assumed to take a `-w` flag to modify files in place.) [default: prettier]
      --elm-formatter <ELM_FORMATTER>
          What formatter should I use for Elm? (Assumed to take a `--yes` flag to modify files in place without confirmation.) [default: elm-format]
  -h, --help
          Print help
  -V, --version
          Print version

```

## License

BSD 3-Clause, same as Elm.
