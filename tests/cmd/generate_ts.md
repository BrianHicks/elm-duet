# Generating TypeScript

You can generate TypeScript types like this:

```console
$ elm-duet tests/schema.json
Cli {
    source: "tests/schema.json",
}
{
  currentTimeMillis: number;
  notificationPermission: "default" | "denied" | "granted";
}

```
