module Main.Flags exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}


type alias Flags =
    { currentJwt : Maybe String
    }


flagsDecoder : Decoder Flags
flagsDecoder =
    Decode.map Flags
        (Decode.field "currentJwt" (Decode.nullable Decode.string))
