use crate::{
    database::entity::ChallengeProgress,
    definitions::challenges::Challenges,
    http::{
        middleware::user::Auth,
        models::{challenge::*, HttpResult},
    },
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
) -> HttpResult<ChallengesResponse> {
    let challenge_definitions = Challenges::get();
    let user_progress = ChallengeProgress::all(&db, &user).await?;

    let challenges: Vec<ChallengeItem> = challenge_definitions
        .values
        .iter()
        .map(|definition| {
            let progress = user_progress
                .iter()
                .filter(|value| value.challenge_id == definition.name)
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
) -> HttpResult<ChallengesResponse> {
    let challenge_definitions = Challenges::get();

    let user_progress = ChallengeProgress::all(&db, &user).await?;

    let challenges: Vec<ChallengeItem> = challenge_definitions
        .values
        .iter()
        .filter_map(|definition| {
            let progress = user_progress
                .iter()
                .filter(|value| value.challenge_id == definition.name)
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
