module B.B2.Flags exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Dict exposing (Dict)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias Flags =
    String


flagsDecoder : Json.Decode.Decoder Flags
flagsDecoder =
    Json.Decode.string


encodeFlags : Flags -> Json.Encode.Value
encodeFlags flags =
    Json.Encode.string flags
