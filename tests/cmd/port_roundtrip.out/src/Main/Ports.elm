port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Dict exposing (Dict)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias ElmToJs =
    { a : String
    }


elmToJsDecoder : Json.Decode.Decoder ElmToJs
elmToJsDecoder =
    Json.Decode.succeed ElmToJs
        |> Json.Decode.Pipeline.required "a" Json.Decode.string


encodeElmToJs : ElmToJs -> Json.Encode.Value
encodeElmToJs elmToJs_ =
    Json.Encode.object
        [ ( "a", Json.Encode.string elmToJs_.a )
        ]


type alias JsToElm =
    { a : String
    }


jsToElmDecoder : Json.Decode.Decoder JsToElm
jsToElmDecoder =
    Json.Decode.succeed JsToElm
        |> Json.Decode.Pipeline.required "a" Json.Decode.string


encodeJsToElm : JsToElm -> Json.Encode.Value
encodeJsToElm jsToElm_ =
    Json.Encode.object
        [ ( "a", Json.Encode.string jsToElm_.a )
        ]


port elmToJs : Json.Decode.Value -> Cmd msg


sendElmToJs : ElmToJs -> Cmd msg
sendElmToJs =
    encodeElmToJs >> elmToJs


port jsToElm : (Json.Decode.Value -> msg) -> Sub msg


subscribeToJsToElm : (Result Json.Decode.Error JsToElm -> msg) -> Sub msg
subscribeToJsToElm toMsg =
    jsToElm (Json.Decode.decodeValue jsToElmDecoder >> toMsg)
