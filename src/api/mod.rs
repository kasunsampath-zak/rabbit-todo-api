pub mod handlers;
mod response;

use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::domain::repositories::*;
use crate::infrastructure::session_cache::SessionCache;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<dyn UserRepository>,
    pub group_repo: Arc<dyn GroupRepository>,
    pub todo_repo: Arc<dyn TodoRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub session_cache: SessionCache,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // User routes
        .route("/api/users", post(handlers::users::create_user))
        .route("/api/users/admin", post(handlers::users::create_admin_user))
        .route("/api/users", get(handlers::users::list_users))
        .route("/api/users/:id", get(handlers::users::get_user))
        .route("/api/users/:id", put(handlers::users::update_user))
        .route("/api/users/:id", delete(handlers::users::delete_user))
        .route("/api/users/:id/profile", get(handlers::users::get_profile))
        
        // Group routes
        .route("/api/groups", post(handlers::groups::create_group))
        .route("/api/groups", get(handlers::groups::list_groups))
        .route("/api/groups/:id", get(handlers::groups::get_group))
        .route("/api/groups/:id", put(handlers::groups::update_group))
        .route("/api/groups/:id", delete(handlers::groups::delete_group))
        .route("/api/groups/:id/members", get(handlers::groups::get_members))
        .route("/api/groups/:id/members/:user_id", post(handlers::groups::add_member))
        .route("/api/groups/:id/members/:user_id", delete(handlers::groups::remove_member))
        .route("/api/groups/:id/stats", get(handlers::groups::get_stats))
        
        // Todo routes
        .route("/api/todos", post(handlers::todos::create_todo))
        .route("/api/todos/:id", get(handlers::todos::get_todo))
        .route("/api/todos/:id", put(handlers::todos::update_todo))
        .route("/api/todos/:id", delete(handlers::todos::delete_todo))
        .route("/api/groups/:group_id/todos", get(handlers::todos::list_group_todos))
        .route("/api/users/:user_id/todos", get(handlers::todos::list_user_todos))
        .route("/api/todos/:id/assign/:user_id", post(handlers::todos::assign_user))
        .route("/api/todos/:id/assign/:user_id", delete(handlers::todos::unassign_user))
        .route("/api/todos/:id/assigned-users", get(handlers::todos::get_assigned_users))
        
        .with_state(state)
}
