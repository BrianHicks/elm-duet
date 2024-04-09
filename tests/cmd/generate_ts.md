# Generating TypeScript

You can generate TypeScript types like this:

```console
$ elm-duet tests/schema.json
{
  currentTimeMillis: number;
  notificationPermission: "default" | "denied" | "granted";
}

```
