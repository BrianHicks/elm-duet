# elm-duet

Elm is great, and TypeScript is great, but the flags and ports between them are hard to use safely.
They're the only part of the a system between those two languages that aren't typed by default.

You can get around this in various ways, of course:

- Manually maintain a `.d.ts` for your Elm app: works, but now you have two sources of truth and have to keep them in sync by hand. That falls apart quickly as the app grows or new people join the team.
- Generate a `.d.ts` from Elm types. Beats doing it by hand, but it constrains you to simple types unless you want to write an interpreter for both your type aliases and inevitable custom types that need decoders from `Json.Decode.Value`.
- Generate Elm types from your TypeScript types. Again, needs a lot of work to intepret the types correctly and translate them.

elm-duet works around this by creating a single source of truth to generate both `.d.ts` files and Elm types+decoders.
We use [JSON Type Definitions](https://jsontypedef.com/) (JTD, [five-minute tutorial](https://jsontypedef.com/docs/jtd-in-5-minutes/)) to say precisely what we want and generate ergonomic types on both sides (plus helpers like encoders to make testing easy!)

Here's an example for an app that stores a [jwt](https://jwt.io/) in `localStorage` or similar to present to Elm:

```json
{
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
        "direction": "elmToJs"
      },
      "type": "string"
    },
    "logout": {
      "metadata": {
        "direction": "elmToJs"
      }
    }
  }
}
```

You can generate code from this like so:

```console
$ elm-duet examples/jwt_schema.json
{
  currentJwt: string;
}

```

Here are more things you can do with the tool:

```console
$ elm-duet --help
Usage: elm-duet <SOURCE>

Arguments:
  <SOURCE>  Location of the definition file

Options:
  -h, --help     Print help
  -V, --version  Print version

```
