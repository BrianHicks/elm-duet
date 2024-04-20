# Generating TypeScript

You can generate TypeScript types like this:

```console
$ elm-duet schema.json
wrote elm.ts
Ok(
    (
        TypeRef(
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
                        "notificationPermission": TypeRef(
                            "NotificationPermission",
                        ),
                    },
                ),
            },
        ],
    ),
)

```
