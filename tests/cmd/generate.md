# Generating TypeScript

You can generate TypeScript types like this:

```console
$ elm-duet schema.json
wrote elm.ts
Ok(
    (
        Ref(
            "Flags",
        ),
        [
            CustomTypeEnum {
                name: InflectedString(
                    "NotificationPermission",
                ),
                cases: [
                    InflectedString(
                        "default",
                    ),
                    InflectedString(
                        "denied",
                    ),
                    InflectedString(
                        "granted",
                    ),
                ],
            },
            TypeAlias {
                name: InflectedString(
                    "Flags",
                ),
                type_: Record(
                    {
                        InflectedString(
                            "currentTimeMillis",
                        ): Scalar(
                            "Float",
                        ),
                        InflectedString(
                            "notificationPermission",
                        ): Ref(
                            "NotificationPermission",
                        ),
                    },
                ),
            },
        ],
    ),
)

```
