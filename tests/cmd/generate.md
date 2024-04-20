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
                name: "NotificationPermission",
                cases: {
                    "DEFAULT": "default",
                    "DENIED": "denied",
                    "GRANTED": "granted",
                },
            },
            TypeAlias {
                name: "Flags",
                type_: Record(
                    {
                        "currentTimeMillis": Scalar(
                            "Float",
                        ),
                        "notificationPermission": Ref(
                            "NotificationPermission",
                        ),
                    },
                ),
            },
        ],
    ),
)

```
