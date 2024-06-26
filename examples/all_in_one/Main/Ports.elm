port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Dict exposing (Dict)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias Close =
    { code : Int
    , reason : String
    , wasClean : Bool
    }


closeDecoder : Json.Decode.Decoder Close
closeDecoder =
    Json.Decode.succeed Close
        |> Json.Decode.Pipeline.required "code" Json.Decode.int
        |> Json.Decode.Pipeline.required "reason" Json.Decode.string
        |> Json.Decode.Pipeline.required "wasClean" Json.Decode.bool


encodeClose : Close -> Json.Encode.Value
encodeClose close_ =
    Json.Encode.object
        [ ( "code", Json.Encode.int close_.code )
        , ( "reason", Json.Encode.string close_.reason )
        , ( "wasClean", Json.Encode.bool close_.wasClean )
        , ( "tag", Json.Encode.string "close" )
        ]


type alias TagError =
    {}


tagErrorDecoder : Json.Decode.Decoder TagError
tagErrorDecoder =
    Json.Decode.succeed TagError


encodeTagError : TagError -> Json.Encode.Value
encodeTagError tagError_ =
    Json.Encode.object
        [ ( "tag", Json.Encode.string "error" )
        ]


type alias Message =
    { data : String
    , origin : String
    }


messageDecoder : Json.Decode.Decoder Message
messageDecoder =
    Json.Decode.succeed Message
        |> Json.Decode.Pipeline.required "data" Json.Decode.string
        |> Json.Decode.Pipeline.required "origin" Json.Decode.string


encodeMessage : Message -> Json.Encode.Value
encodeMessage message_ =
    Json.Encode.object
        [ ( "data", Json.Encode.string message_.data )
        , ( "origin", Json.Encode.string message_.origin )
        , ( "tag", Json.Encode.string "message" )
        ]


type alias TagOpen =
    {}


tagOpenDecoder : Json.Decode.Decoder TagOpen
tagOpenDecoder =
    Json.Decode.succeed TagOpen


encodeTagOpen : TagOpen -> Json.Encode.Value
encodeTagOpen tagOpen_ =
    Json.Encode.object
        [ ( "tag", Json.Encode.string "open" )
        ]


type FromWorld
    = FromWorldClose Close
    | FromWorldError TagError
    | FromWorldMessage Message
    | FromWorldOpen TagOpen


fromWorldDecoder : Json.Decode.Decoder FromWorld
fromWorldDecoder =
    Json.Decode.andThen
        (\tag ->
            case tag of
                "close" ->
                    Json.Decode.map FromWorldClose closeDecoder

                "error" ->
                    Json.Decode.map FromWorldError tagErrorDecoder

                "message" ->
                    Json.Decode.map FromWorldMessage messageDecoder

                "open" ->
                    Json.Decode.map FromWorldOpen tagOpenDecoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "tag" Json.Decode.string)


encodeFromWorld : FromWorld -> Json.Encode.Value
encodeFromWorld fromWorld_ =
    case fromWorld_ of
        FromWorldClose fromWorldClose ->
            encodeClose fromWorldClose

        FromWorldError fromWorldError ->
            encodeTagError fromWorldError

        FromWorldMessage fromWorldMessage ->
            encodeMessage fromWorldMessage

        FromWorldOpen fromWorldOpen ->
            encodeTagOpen fromWorldOpen


type alias Close =
    { code : Int
    , reason : String
    }


closeDecoder : Json.Decode.Decoder Close
closeDecoder =
    Json.Decode.succeed Close
        |> Json.Decode.Pipeline.required "code" Json.Decode.int
        |> Json.Decode.Pipeline.required "reason" Json.Decode.string


encodeClose : Close -> Json.Encode.Value
encodeClose close_ =
    Json.Encode.object
        [ ( "code", Json.Encode.int close_.code )
        , ( "reason", Json.Encode.string close_.reason )
        , ( "tag", Json.Encode.string "close" )
        ]


type alias Connect =
    { protocols : Maybe (List String)
    , url : String
    }


connectDecoder : Json.Decode.Decoder Connect
connectDecoder =
    Json.Decode.succeed Connect
        |> Json.Decode.Pipeline.optional "protocols" (Json.Decode.nullable (Json.Decode.list Json.Decode.string)) Nothing
        |> Json.Decode.Pipeline.required "url" Json.Decode.string


encodeConnect : Connect -> Json.Encode.Value
encodeConnect connect_ =
    List.filterMap identity
        [ Maybe.map (\protocols_ -> ( "protocols", Json.Encode.list (\value -> Json.Encode.string value) protocols_ )) connect_.protocols
        , Just ( "url", Json.Encode.string connect_.url )
        , Just ( "tag", Json.Encode.string "connect" )
        ]
        |> Json.Encode.object


type alias Send =
    { message : String
    }


sendDecoder : Json.Decode.Decoder Send
sendDecoder =
    Json.Decode.succeed Send
        |> Json.Decode.Pipeline.required "message" Json.Decode.string


encodeSend : Send -> Json.Encode.Value
encodeSend send_ =
    Json.Encode.object
        [ ( "message", Json.Encode.string send_.message )
        , ( "tag", Json.Encode.string "send" )
        ]


type ToWorld
    = ToWorldClose Close
    | ToWorldConnect Connect
    | ToWorldSend Send


toWorldDecoder : Json.Decode.Decoder ToWorld
toWorldDecoder =
    Json.Decode.andThen
        (\tag ->
            case tag of
                "close" ->
                    Json.Decode.map ToWorldClose closeDecoder

                "connect" ->
                    Json.Decode.map ToWorldConnect connectDecoder

                "send" ->
                    Json.Decode.map ToWorldSend sendDecoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "tag" Json.Decode.string)


encodeToWorld : ToWorld -> Json.Encode.Value
encodeToWorld toWorld_ =
    case toWorld_ of
        ToWorldClose toWorldClose ->
            encodeClose toWorldClose

        ToWorldConnect toWorldConnect ->
            encodeConnect toWorldConnect

        ToWorldSend toWorldSend ->
            encodeSend toWorldSend


port fromWorld : (Json.Decode.Value -> msg) -> Sub msg


subscribeToFromWorld : (Result Json.Decode.Error FromWorld -> msg) -> Sub msg
subscribeToFromWorld toMsg =
    fromWorld (Json.Decode.decodeValue fromWorldDecoder >> toMsg)


port toWorld : Json.Decode.Value -> Cmd msg


sendToWorld : ToWorld -> Cmd msg
sendToWorld =
    encodeToWorld >> toWorld
