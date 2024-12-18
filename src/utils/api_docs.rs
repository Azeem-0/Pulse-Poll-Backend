use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
        paths(
            crate::services::poll_service::get_all_polls,
            crate::services::poll_service::get_poll_by_id,
            crate::services::poll_service::fetch_results_by_id,
            crate::services::poll_service::get_all_polls,
            crate::services::poll_service::cast_vote_to_poll,
            crate::services::poll_service::close_poll_by_id,
            crate::services::poll_service::reset_votes_by_id,
        ),
        components(schemas(
            crate::models::poll_model::Poll,
            crate::models::poll_model::OptionItem,
            crate::models::user_model::User,
            crate::models::poll_model::VoteHistory,
            crate::models::poll_model::PollResults,
            crate::models::poll_model::PollQueryParams,
            crate::models::poll_model::ResultsOptionItem,
            crate::utils::types::PollCreation,
            crate::utils::types::VoteOption,
            crate::utils::types::UserNameRequest
        )),
        tags(
            (name = "Polls", description = "Operations related to polls, including creation, voting, and results."),
        )
    )]
pub struct ApiDoc;
