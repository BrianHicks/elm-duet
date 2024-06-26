port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Dict exposing (Dict)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias NewJwt =
    String


newJwtDecoder : Json.Decode.Decoder NewJwt
newJwtDecoder =
    Json.Decode.string


encodeNewJwt : NewJwt -> Json.Encode.Value
encodeNewJwt newJwt_ =
    Json.Encode.string newJwt_


port newJwt : Json.Decode.Value -> Cmd msg


sendNewJwt : NewJwt -> Cmd msg
sendNewJwt =
    encodeNewJwt >> newJwt
