# Generating TypeScript

You can generate TypeScript types like this:

```console
$ elm-duet schema.json
wrote elm.ts
Ok(
    (
        Ref(
            InflectedString(
                "Flags",
            ),
        ),
        [
            CustomTypeEnum {
                name: InflectedString(
                    "NotificationPermission",
                ),
                cases: {
                    InflectedString(
                        "default",
                    ): None,
                    InflectedString(
                        "denied",
                    ): None,
                    InflectedString(
                        "granted",
                    ): None,
                },
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
                            InflectedString(
                                "NotificationPermission",
                            ),
                        ),
                    },
                ),
            },
        ],
    ),
)

```
