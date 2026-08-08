use std::collections::HashMap;

use crate::{
    database::entity::{
        ChallengeProgress,
        challenge_progress::{
            ChallengeProgressCounter, ChallengeProgressCounterWithDefinition, ChallengeState,
        },
    },
    definitions::challenges::Challenges,
    http::{
        middleware::user::Auth,
        models::{HttpResult, challenge::*},
    },
};
use axum::{Extension, Json};
use chrono::Utc;
use sea_orm::DatabaseConnection;

/// GET /challenges/categories
///
/// Intended to obtain a list of challenge categories but
/// seems to just return an empty response
pub async fn get_challenge_categories() -> Json<ChallengeCategories> {
    Json(ChallengeCategories { categories: vec![] })
}

// /// GET /challenges
// ///
// /// Obtains a list of all the challenges that can be completed
// pub async fn get_challenges(
//     Extension(db): Extension<DatabaseConnection>,
//     Auth(user): Auth,
// ) -> HttpResult<AllChallengesResponse> {
//     let challenge_definitions = Challenges::get();
//     let user_progress = ChallengeProgress::all(&db, &user).await?;

//     let challenges: Vec<ChallengeAllItem> = challenge_definitions
//         .values
//         .iter()
//         .map(|definition| {
//             let progress = user_progress
//                 .iter()
//                 .filter(|value| value.challenge_id == definition.base.name)
//                 .cloned()
//                 .collect::<Vec<_>>();
//             ChallengeAllItem {
//                 definition,
//                 progress: if progress.is_empty() {
//                     None
//                 } else {
//                     Some(progress)
//                 },
//             }
//         })
//         .collect();

//     Ok(Json(AllChallengesResponse { challenges }))
// }
/// GET /challenges
///
/// Obtains a list of all the challenges that can be completed
pub async fn get_challenges() -> HttpResult<serde_json::Value> {
    Ok(Json(
        serde_json::from_str(include_str!("./mock_challenges.json")).unwrap(),
    ))
}

// /// GET /challenges/user
// ///
// /// Obtains a list of all the challenges the user has either
// /// completed or has started.
// pub async fn get_user_challenges(
//     Extension(db): Extension<DatabaseConnection>,
//     Auth(user): Auth,
// ) -> HttpResult<UserChallengesResponse> {
//     let challenge_definitions = Challenges::get();

//     let user_progress = ChallengeProgress::all(&db, &user).await?;

//     let mut user_progress_lookup = HashMap::new();
//     for progress in user_progress {
//         user_progress_lookup.insert(progress.challenge_id, progress);
//     }

//     let challenges: Vec<UserChallengeItem> = challenge_definitions
//         .values
//         .iter()
//         .map(|definition| {
//             let progress = match user_progress_lookup.get(&definition.base.name).cloned() {
//                 Some(value) => value,
//                 // Default states
//                 None => {
//                     let counters = definition
//                         .counters
//                         .iter()
//                         .map(|counter| ChallengeProgressCounterWithDefinition {
//                             definition: counter.clone(),
//                             counter: ChallengeProgressCounter::new(counter.name.clone()),
//                         })
//                         .collect();

//                     return UserChallengeItem {
//                         definition: &definition.base,
//                         counters,
//                         state: ChallengeState::NotStarted,
//                         times_completed: 0,
//                         last_completed: None,
//                         first_completed: None,
//                         // last_changed: Utc::now(),
//                         last_changed: "2023-07-05T01:56:31.430+0000".to_string(),
//                         rewarded: false,
//                     };
//                 }
//             };

//             let counters = definition
//                 .counters
//                 .iter()
//                 .map(|counter| {
//                     let progress = progress
//                         .counters
//                         .0
//                         .iter()
//                         .find(|counter_progress| counter_progress.name == counter.name)
//                         .cloned()
//                         .unwrap_or_else(|| ChallengeProgressCounter::new(counter.name.clone()));

//                     ChallengeProgressCounterWithDefinition {
//                         definition: counter.clone(),
//                         counter: progress,
//                     }
//                 })
//                 .collect();

//             UserChallengeItem {
//                 definition: &definition.base,
//                 counters,
//                 state: progress.state,
//                 times_completed: progress.times_completed,
//                 // last_completed: progress.last_completed,
//                 // first_completed: progress.first_completed,
//                 // last_changed: progress.last_changed,
//                 last_completed: Some("2023-07-05T01:56:31.430+0000".to_string()),
//                 first_completed: Some("2023-07-05T01:56:31.430+0000".to_string()),
//                 last_changed: "2023-07-05T01:56:31.430+0000".to_string(),
//                 rewarded: progress.rewarded,
//             }
//         })
//         .collect();

//     Ok(Json(UserChallengesResponse { challenges }))
// }

/// GET /challenges/user
///
/// Obtains a list of all the challenges the user has either
/// completed or has started.
pub async fn get_user_challenges() -> HttpResult<serde_json::Value> {
    Ok(Json(
        serde_json::from_str(include_str!("./mock_user_challenges.json")).unwrap(),
    ))
}
