use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    Json,
};
use hyper::{header::CONTENT_TYPE, http::HeaderValue, StatusCode};
use log::debug;
use uuid::{uuid, Uuid};

use crate::http::models::{
    character::{CharacterEquipment, CharacterEquipmentList, MaybeUuid},
    RawJson,
};

/// GET /characters
pub async fn get_characters() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Characters-1689039048546.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /character/:id
///
/// Gets the defintion and details for the character of the provided ID
pub async fn get_character(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_-1689039081314.json").into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// POST /character/:id/active
///
/// Sets the currently active character
pub async fn set_active(Path(character_id): Path<Uuid>) -> Response {
    debug!("Requested set active character: {}", character_id);

    // TODO: Set as active character

    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment
///
/// Gets the current equipment of the provided character
pub async fn get_character_equip(Path(character_id): Path<Uuid>) -> Json<CharacterEquipmentList> {
    debug!("Requested character equip: {}", character_id);

    let list = vec![
        CharacterEquipment {
            slot: "weaponSlot1".to_string(),
            name: MaybeUuid(Some(uuid!("e27b77d9-06bc-422c-9ac5-46f12510e668"))),
            attachments: vec![
                uuid!("d59f6774-f5e9-48c9-ba8c-4766e4f07fab"),
                uuid!("3815b17a-3e21-4d88-944e-0ef452dc0fb1"),
            ],
        },
        CharacterEquipment {
            slot: "weaponSlot2".to_string(),
            name: MaybeUuid(Some(uuid!("e8406e6a-01be-4844-98ed-efcc0e2d6c29"))),
            attachments: vec![
                uuid!("92cece94-cc3a-4a73-b4ea-b52462ba0404"),
                uuid!("b0a2c013-e791-4c20-9e7a-05e865bfbcaa"),
            ],
        },
        CharacterEquipment {
            slot: "equipmentSlot".to_string(),
            name: MaybeUuid(Some(uuid!("feb691f1-2b54-4455-8a44-531e2851f007"))),
            attachments: vec![],
        },
    ];

    Json(CharacterEquipmentList { list })
}

/// PUT /character/:id/equipment'
///
/// Updates the equipment for the provided character using
/// the provided equipment list
pub async fn update_character_equip(
    Path(character_id): Path<Uuid>,
    Json(req): Json<CharacterEquipmentList>,
) -> Response {
    debug!("Update charcter equipment: {} - {:?}", character_id, req);

    StatusCode::NO_CONTENT.into_response()
}

/// GET /character/:id/equipment/history
///
/// Obtains the history of the characters previous
/// equipment
pub async fn get_character_equip_history(
    Path(character_id): Path<Uuid>,
) -> Json<CharacterEquipmentList> {
    debug!("Requested character equip history: {}", character_id);

    let list = vec![
        CharacterEquipment {
            slot: "weaponSlot1".to_string(),
            name: MaybeUuid(Some(uuid!("d5bf2213-d2d2-f892-7310-c39a15fb2ef3"))),
            attachments: vec![],
        },
        CharacterEquipment {
            slot: "weaponSlot2".to_string(),
            name: MaybeUuid(Some(uuid!("ca7d0f24-fc19-4a78-9d25-9c84eb01e3a5"))),
            attachments: vec![],
        },
        CharacterEquipment {
            slot: "weaponSlot1".to_string(),
            name: MaybeUuid(Some(uuid!("e27b77d9-06bc-422c-9ac5-46f12510e668"))),
            attachments: vec![
                uuid!("790352cc-9444-4d28-ad9b-4a162492a322"),
                uuid!("d2a14c38-9a70-40bd-9022-9e9f24c15e17"),
            ],
        },
        CharacterEquipment {
            slot: "weaponSlot2".to_string(),
            name: MaybeUuid(Some(uuid!("e8406e6a-01be-4844-98ed-efcc0e2d6c29"))),
            attachments: vec![
                uuid!("92cece94-cc3a-4a73-b4ea-b52462ba0404"),
                uuid!("b0a2c013-e791-4c20-9e7a-05e865bfbcaa"),
            ],
        },
    ];

    Json(CharacterEquipmentList { list })
}

/// PUT /character/:id/skillTrees
pub async fn update_skill_tree(Path(character_id): Path<Uuid>) -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Update_Character_Skill_Trees-1688700262159.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// GET /character/classes
pub async fn get_classes() -> Response {
    let mut resp =
        include_str!("../../resources/defs/raw/Get_Character_Classes-1688700203514.json")
            .into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    resp
}

/// Definitions for rewards at each character level
static CHARACTER_LEVEL_TABLES: &str =
    include_str!("../../resources/data/characterLevelTables.json");

/// GET /character/levelTables
///
/// Contains definitions for rewards at each level of character
/// progression
pub async fn get_level_tables() -> RawJson {
    RawJson(CHARACTER_LEVEL_TABLES)
}
