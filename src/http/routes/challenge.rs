use crate::{
    database::entity::ChallengeProgress,
    http::{
        middleware::user::Auth,
        models::{
            challenge::{ChallengeCategories, ChallengeItem, ChallengesResponse},
            HttpError,
        },
    },
    state::App,
};
use axum::{Extension, Json};
use sea_orm::DatabaseConnection;

/// GET /challenges/categories
///
/// Intended to obtain a list of challenge categories but
/// seems to just return an empty response
pub async fn get_challenge_categories() -> Json<ChallengeCategories> {
    Json(ChallengeCategories { categories: vec![] })
}

/// GET /challenges
///
/// Obtains a list of all the challenges that can be completed
pub async fn get_challenges(
    Extension(db): Extension<DatabaseConnection>,
    Auth(user): Auth,
) -> Result<Json<ChallengesResponse>, HttpError> {
    let services = App::services();

    let user_progress = ChallengeProgress::all_with_counters(&db, &user).await?;

    let challenges: Vec<ChallengeItem> = services
        .challenges
        .defs
        .iter()
        .map(|definition| {
            let progress = user_progress
                .iter()
                .filter(|value| value.progress.challenge_id == definition.name)
                .cloned()
                .collect::<Vec<_>>();
            ChallengeItem {
                definition,
                progress: if progress.is_empty() {
                    None
                } else {
                    Some(progress)
                },
            }
        })
        .collect();

    Ok(Json(ChallengesResponse { challenges }))
}

/// GET /challenges/user
///
/// Obtains a list of all the challenges the user has either
/// completed or has started.
pub async fn get_user_challenges(
    Extension(db): Extension<DatabaseConnection>,
    Auth(user): Auth,
) -> Result<Json<ChallengesResponse>, HttpError> {
    let services = App::services();

    let user_progress = ChallengeProgress::all_with_counters(&db, &user).await?;

    let challenges: Vec<ChallengeItem> = services
        .challenges
        .defs
        .iter()
        .filter_map(|definition| {
            let progress = user_progress
                .iter()
                .filter(|value| value.progress.challenge_id == definition.name)
                .cloned()
                .collect::<Vec<_>>();
            if progress.is_empty() {
                None
            } else {
                Some(ChallengeItem {
                    definition,
                    progress: Some(progress),
                })
            }
        })
        .collect();

    Ok(Json(ChallengesResponse { challenges }))
}
