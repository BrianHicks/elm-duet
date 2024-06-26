port module Main.Ports exposing (..)

{-| Warning: this file is automatically generated. Don't edit by hand!
-}

import Dict exposing (Dict)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


type alias AddNewPingAt =
    { value : Int
    }


addNewPingAtDecoder : Json.Decode.Decoder AddNewPingAt
addNewPingAtDecoder =
    Json.Decode.succeed AddNewPingAt
        |> Json.Decode.Pipeline.required "value" Json.Decode.int


encodeAddNewPingAt : AddNewPingAt -> Json.Encode.Value
encodeAddNewPingAt addNewPingAt_ =
    Json.Encode.object
        [ ( "value", Json.Encode.int addNewPingAt_.value )
        , ( "tag", Json.Encode.string "AddNewPingAt" )
        ]


type alias SetMinutesPerPing =
    { value : Int
    }


setMinutesPerPingDecoder : Json.Decode.Decoder SetMinutesPerPing
setMinutesPerPingDecoder =
    Json.Decode.succeed SetMinutesPerPing
        |> Json.Decode.Pipeline.required "value" Json.Decode.int


encodeSetMinutesPerPing : SetMinutesPerPing -> Json.Encode.Value
encodeSetMinutesPerPing setMinutesPerPing_ =
    Json.Encode.object
        [ ( "value", Json.Encode.int setMinutesPerPing_.value )
        , ( "tag", Json.Encode.string "SetMinutesPerPing" )
        ]


type alias SetTagForPing =
    { index : Int
    , value : Maybe String
    }


setTagForPingDecoder : Json.Decode.Decoder SetTagForPing
setTagForPingDecoder =
    Json.Decode.succeed SetTagForPing
        |> Json.Decode.Pipeline.required "index" Json.Decode.int
        |> Json.Decode.Pipeline.required "value" (Json.Decode.nullable Json.Decode.string)


encodeSetTagForPing : SetTagForPing -> Json.Encode.Value
encodeSetTagForPing setTagForPing_ =
    Json.Encode.object
        [ ( "index", Json.Encode.int setTagForPing_.index )
        , ( "value"
          , case setTagForPing_.value of
                Just value ->
                    Json.Encode.string value

                Nothing ->
                    Json.Encode.null
          )
        , ( "tag", Json.Encode.string "SetTagForPing" )
        ]


type ChangeDocumentElements
    = ChangeDocumentElementsAddNewPingAt AddNewPingAt
    | ChangeDocumentElementsSetMinutesPerPing SetMinutesPerPing
    | ChangeDocumentElementsSetTagForPing SetTagForPing


changeDocumentElementsDecoder : Json.Decode.Decoder ChangeDocumentElements
changeDocumentElementsDecoder =
    Json.Decode.andThen
        (/tag ->
            case tag of
                "AddNewPingAt" ->
                    Json.Decode.map ChangeDocumentElementsAddNewPingAt addNewPingAtDecoder

                "SetMinutesPerPing" ->
                    Json.Decode.map ChangeDocumentElementsSetMinutesPerPing setMinutesPerPingDecoder

                "SetTagForPing" ->
                    Json.Decode.map ChangeDocumentElementsSetTagForPing setTagForPingDecoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "tag" Json.Decode.string)


encodeChangeDocumentElements : ChangeDocumentElements -> Json.Encode.Value
encodeChangeDocumentElements changeDocumentElements_ =
    case changeDocumentElements_ of
        ChangeDocumentElementsAddNewPingAt changeDocumentElementsAddNewPingAt ->
            encodeAddNewPingAt changeDocumentElementsAddNewPingAt

        ChangeDocumentElementsSetMinutesPerPing changeDocumentElementsSetMinutesPerPing ->
            encodeSetMinutesPerPing changeDocumentElementsSetMinutesPerPing

        ChangeDocumentElementsSetTagForPing changeDocumentElementsSetTagForPing ->
            encodeSetTagForPing changeDocumentElementsSetTagForPing


type alias ChangeDocument =
    List ChangeDocumentElements


changeDocumentDecoder : Json.Decode.Decoder ChangeDocument
changeDocumentDecoder =
    Json.Decode.list changeDocumentElementsDecoder


encodeChangeDocument : ChangeDocument -> Json.Encode.Value
encodeChangeDocument changeDocument_ =
    Json.Encode.list (/value -> encodeChangeDocumentElements value) changeDocument_


type alias PingV1 =
    { custom : Dict String String
    , tag : Maybe String
    , time : Int
    }


pingV1Decoder : Json.Decode.Decoder PingV1
pingV1Decoder =
    Json.Decode.succeed PingV1
        |> Json.Decode.Pipeline.required "custom" (Json.Decode.dict Json.Decode.string)
        |> Json.Decode.Pipeline.required "tag" (Json.Decode.nullable Json.Decode.string)
        |> Json.Decode.Pipeline.required "time" Json.Decode.int


encodePingV1 : PingV1 -> Json.Encode.Value
encodePingV1 pingV1_ =
    Json.Encode.object
        [ ( "custom", Json.Encode.dict identity (/value -> Json.Encode.string value) pingV1_.custom )
        , ( "tag"
          , case pingV1_.tag of
                Just value ->
                    Json.Encode.string value

                Nothing ->
                    Json.Encode.null
          )
        , ( "time", Json.Encode.int pingV1_.time )
        , ( "version", Json.Encode.string "v1" )
        ]


type PingsElements
    = VersionedPingsElementsV1 PingV1


pingsElementsDecoder : Json.Decode.Decoder PingsElements
pingsElementsDecoder =
    Json.Decode.andThen
        (/tag ->
            case tag of
                "v1" ->
                    Json.Decode.map VersionedPingsElementsV1 pingV1Decoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "version" Json.Decode.string)


encodePingsElements : PingsElements -> Json.Encode.Value
encodePingsElements pingsElements_ =
    case pingsElements_ of
        VersionedPingsElementsV1 versionedPingsElementsV1 ->
            encodePingV1 versionedPingsElementsV1


type alias SettingsV1 =
    { minutesPerPing : Int
    }


settingsV1Decoder : Json.Decode.Decoder SettingsV1
settingsV1Decoder =
    Json.Decode.succeed SettingsV1
        |> Json.Decode.Pipeline.required "minutesPerPing" Json.Decode.int


encodeSettingsV1 : SettingsV1 -> Json.Encode.Value
encodeSettingsV1 settingsV1_ =
    Json.Encode.object
        [ ( "minutesPerPing", Json.Encode.int settingsV1_.minutesPerPing )
        , ( "version", Json.Encode.string "v1" )
        ]


type Settings
    = VersionedSettingsV1 SettingsV1


settingsDecoder : Json.Decode.Decoder Settings
settingsDecoder =
    Json.Decode.andThen
        (/tag ->
            case tag of
                "v1" ->
                    Json.Decode.map VersionedSettingsV1 settingsV1Decoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "version" Json.Decode.string)


encodeSettings : Settings -> Json.Encode.Value
encodeSettings settings_ =
    case settings_ of
        VersionedSettingsV1 versionedSettingsV1 ->
            encodeSettingsV1 versionedSettingsV1


type alias DocV1 =
    { pings : List PingsElements
    , settings : Settings
    }


docV1Decoder : Json.Decode.Decoder DocV1
docV1Decoder =
    Json.Decode.succeed DocV1
        |> Json.Decode.Pipeline.required "pings" (Json.Decode.list pingsElementsDecoder)
        |> Json.Decode.Pipeline.required "settings" settingsDecoder


encodeDocV1 : DocV1 -> Json.Encode.Value
encodeDocV1 docV1_ =
    Json.Encode.object
        [ ( "pings", Json.Encode.list (/value -> encodePingsElements value) docV1_.pings )
        , ( "settings", encodeSettings docV1_.settings )
        , ( "version", Json.Encode.string "v1" )
        ]


type DocFromAutomerge
    = VersionedDocFromAutomergeV1 DocV1


docFromAutomergeDecoder : Json.Decode.Decoder DocFromAutomerge
docFromAutomergeDecoder =
    Json.Decode.andThen
        (/tag ->
            case tag of
                "v1" ->
                    Json.Decode.map VersionedDocFromAutomergeV1 docV1Decoder

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        (Json.Decode.field "version" Json.Decode.string)


encodeDocFromAutomerge : DocFromAutomerge -> Json.Encode.Value
encodeDocFromAutomerge docFromAutomerge_ =
    case docFromAutomerge_ of
        VersionedDocFromAutomergeV1 versionedDocFromAutomergeV1 ->
            encodeDocV1 versionedDocFromAutomergeV1


type NotificationPermission
    = NotificationPermissionDefault
    | NotificationPermissionDenied
    | NotificationPermissionGranted


notificationPermissionDecoder : Json.Decode.Decoder NotificationPermission
notificationPermissionDecoder =
    Json.Decode.andThen
        (/tag ->
            case tag of
                "default" ->
                    Json.Decode.succeed NotificationPermissionDefault

                "denied" ->
                    Json.Decode.succeed NotificationPermissionDenied

                "granted" ->
                    Json.Decode.succeed NotificationPermissionGranted

                unknown ->
                    Json.Decode.fail ("Unknown value `" ++ unknown ++ "`")
        )
        Json.Decode.string


encodeNotificationPermission : NotificationPermission -> Json.Encode.Value
encodeNotificationPermission notificationPermission_ =
    case notificationPermission_ of
        NotificationPermissionDefault ->
            Json.Encode.string "default"

        NotificationPermissionDenied ->
            Json.Encode.string "denied"

        NotificationPermissionGranted ->
            Json.Encode.string "granted"


type alias Options =
    { badge : Maybe String
    , body : Maybe String
    , icon : Maybe String
    , lang : Maybe String
    , requireInteraction : Maybe Bool
    , silent : Maybe Bool
    , tag : Maybe String
    }


optionsDecoder : Json.Decode.Decoder Options
optionsDecoder =
    Json.Decode.succeed Options
        |> Json.Decode.Pipeline.optional "badge" (Json.Decode.nullable Json.Decode.string) Nothing
        |> Json.Decode.Pipeline.optional "body" (Json.Decode.nullable Json.Decode.string) Nothing
        |> Json.Decode.Pipeline.optional "icon" (Json.Decode.nullable Json.Decode.string) Nothing
        |> Json.Decode.Pipeline.optional "lang" (Json.Decode.nullable Json.Decode.string) Nothing
        |> Json.Decode.Pipeline.optional "requireInteraction" (Json.Decode.nullable Json.Decode.bool) Nothing
        |> Json.Decode.Pipeline.optional "silent" (Json.Decode.nullable Json.Decode.bool) Nothing
        |> Json.Decode.Pipeline.optional "tag" (Json.Decode.nullable Json.Decode.string) Nothing


encodeOptions : Options -> Json.Encode.Value
encodeOptions options_ =
    List.filterMap identity
        [ Maybe.map (/badge_ -> ( "badge", Json.Encode.string badge_ )) options_.badge
        , Maybe.map (/body_ -> ( "body", Json.Encode.string body_ )) options_.body
        , Maybe.map (/icon_ -> ( "icon", Json.Encode.string icon_ )) options_.icon
        , Maybe.map (/lang_ -> ( "lang", Json.Encode.string lang_ )) options_.lang
        , Maybe.map (/requireInteraction_ -> ( "requireInteraction", Json.Encode.bool requireInteraction_ )) options_.requireInteraction
        , Maybe.map (/silent_ -> ( "silent", Json.Encode.bool silent_ )) options_.silent
        , Maybe.map (/tag_ -> ( "tag", Json.Encode.string tag_ )) options_.tag
        ]
        |> Json.Encode.object


type alias Notification =
    { options : Options
    , title : String
    }


notificationDecoder : Json.Decode.Decoder Notification
notificationDecoder =
    Json.Decode.succeed Notification
        |> Json.Decode.Pipeline.required "options" optionsDecoder
        |> Json.Decode.Pipeline.required "title" Json.Decode.string


encodeNotification : Notification -> Json.Encode.Value
encodeNotification notification_ =
    Json.Encode.object
        [ ( "options", encodeOptions notification_.options )
        , ( "title", Json.Encode.string notification_.title )
        ]


type alias RequestNotificationsPermission =
    ()


requestNotificationsPermissionDecoder : Json.Decode.Decoder RequestNotificationsPermission
requestNotificationsPermissionDecoder =
    Json.Decode.null ()


encodeRequestNotificationsPermission : RequestNotificationsPermission -> Json.Encode.Value
encodeRequestNotificationsPermission requestNotificationsPermission_ =
    Json.Encode.null


port changeDocument : Json.Decode.Value -> Cmd msg


sendChangeDocument : ChangeDocument -> Cmd msg
sendChangeDocument =
    encodeChangeDocument >> changeDocument


port docFromAutomerge : (Json.Decode.Value -> msg) -> Sub msg


subscribeToDocFromAutomerge : (Result Json.Decode.Error DocFromAutomerge -> msg) -> Sub msg
subscribeToDocFromAutomerge toMsg =
    docFromAutomerge (Json.Decode.decodeValue docFromAutomergeDecoder >> toMsg)


port gotNewNotificationsPermission : (Json.Decode.Value -> msg) -> Sub msg


subscribeToGotNewNotificationsPermission : (Result Json.Decode.Error NotificationPermission -> msg) -> Sub msg
subscribeToGotNewNotificationsPermission toMsg =
    gotNewNotificationsPermission (Json.Decode.decodeValue notificationPermissionDecoder >> toMsg)


port notify : Json.Decode.Value -> Cmd msg


sendNotify : Notification -> Cmd msg
sendNotify =
    encodeNotification >> notify


port requestNotificationsPermission : Json.Decode.Value -> Cmd msg


sendRequestNotificationsPermission : RequestNotificationsPermission -> Cmd msg
sendRequestNotificationsPermission =
    encodeRequestNotificationsPermission >> requestNotificationsPermission
