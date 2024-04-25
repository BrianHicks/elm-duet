port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias Logout =
    ()


logoutDecoder : Decoder Logout
logoutDecoder =
    Json.Decode.null ()


encodeLogout : Logout -> Json.Encode.Value
encodeLogout logout =
    Json.Encode.null


type alias NewJwt =
    String


newJwtDecoder : Decoder NewJwt
newJwtDecoder =
    Json.Decode.string


encodeNewJwt : NewJwt -> Json.Encode.Value
encodeNewJwt newJwt =
    Json.Encode.string newJwt


port logout : Value -> Cmd msg


logout_ : Logout -> Cmd msg
logout_ value =
    logout (encodeLogout value)


port newJwt : Value -> Cmd msg


newJwt_ : NewJwt -> Cmd msg
newJwt_ value =
    newJwt (encodeNewJwt value)
