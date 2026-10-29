use rest_api::database::category::{CategoriesRepository, CategoryType};
use rest_api::database::session::SessionsRepository;
use rest_api::database::user::UsersRepository;
use rest_api::Database;
use sqlx::types::time::OffsetDateTime;
use std::time::Duration;
use uuid::Uuid;

async fn db() -> Database {
    Database::connect("sqlite::memory:", true).await.unwrap()
}

#[tokio::test]
async fn create_user_then_find_it_by_email() {
    let db = db().await;
    let created = db.create_user("budi@example.com", "Budi", "hash").await.unwrap();

    let found = db.get_user_by_email("budi@example.com").await.unwrap().unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!((found.name.as_str(), found.password.as_str()), ("Budi", "hash"));
    assert!(db.get_user_by_email("nobody@example.com").await.unwrap().is_none());
}

#[tokio::test]
async fn email_is_unique_and_case_insensitive() {
    let db = db().await;
    db.create_user("a@example.com", "A", "hash").await.unwrap();

    assert!(db.create_user("A@Example.com", "A2", "hash").await.is_err());
    let found = db.get_user_by_email("A@EXAMPLE.COM").await.unwrap();
    assert_eq!(found.unwrap().email, "a@example.com");
}

#[tokio::test]
async fn session_lookup_respects_expiry() {
    let db = db().await;
    let user = db.create_user("c@example.com", "C", "hash").await.unwrap();
    let now = OffsetDateTime::now_utc();

    let active = db.new_session(user.id, "test", &(now + Duration::from_secs(7 * 24 * 3600))).await.unwrap();
    let expired = db.new_session(user.id, "test", &(now - Duration::from_secs(1))).await.unwrap();

    let found = db.get_user_by_session_id(active.id).await.unwrap();
    assert_eq!(found.unwrap().id, user.id);
    assert!(db.get_user_by_session_id(expired.id).await.unwrap().is_none());
    assert!(db.get_user_by_session_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn deleted_session_no_longer_finds_its_user() {
    let db = db().await;
    let user = db.create_user("g@example.com", "G", "hash").await.unwrap();
    let exp = OffsetDateTime::now_utc() + Duration::from_secs(60);
    let session = db.new_session(user.id, "test", &exp).await.unwrap();
    let other = db.new_session(user.id, "other", &exp).await.unwrap();

    db.delete_session(session.id).await.unwrap();

    assert!(db.get_user_by_session_id(session.id).await.unwrap().is_none());
    assert!(db.get_user_by_session_id(other.id).await.unwrap().is_some());
    db.delete_session(session.id).await.unwrap();
}

#[tokio::test]
async fn categories_crud_for_one_user() {
    let db = db().await;
    let user = db.create_user("d@example.com", "D", "hash").await.unwrap();
    let other = db.create_user("e@example.com", "E", "hash").await.unwrap();

    let cat = db
        .create_category(user.id, "Rent".into(), CategoryType::Essential)
        .await
        .unwrap();
    db.create_category(other.id, "Games".into(), CategoryType::NonEssential)
        .await
        .unwrap();
    assert_eq!(cat.cat_type, CategoryType::Essential);

    let mine = db.get_categories(user.id).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].name, "Rent");

    let updated = db
        .update_category(cat.id, "Food".into(), CategoryType::NonEssential)
        .await
        .unwrap();
    assert_eq!((updated.name.as_str(), updated.cat_type), ("Food", CategoryType::NonEssential));

    db.delete_category(cat.id).await.unwrap();
    assert!(db.get_category_by_id(cat.id).await.unwrap().is_none());
}

#[tokio::test]
async fn category_needs_an_existing_user() {
    let db = db().await;
    let result = db
        .create_category(Uuid::new_v4(), "Orphan".into(), CategoryType::Essential)
        .await;
    assert!(result.is_err(), "foreign keys must be enforced");
}

#[tokio::test]
async fn new_session_is_readable_right_away_on_a_file_database() {
    let dir = std::env::temp_dir().join(format!("rest_api_test_{}", Uuid::new_v4()));
    let url = format!("sqlite://{}/app.db", dir.display());
    let db = Database::connect(&url, true).await.unwrap();
    let user = db.create_user("f@example.com", "F", "hash").await.unwrap();
    let exp = OffsetDateTime::now_utc() + Duration::from_secs(60);

    for _ in 0..50 {
        let session = db.new_session(user.id, "test", &exp).await.unwrap();
        assert!(db.get_user_by_session_id(session.id).await.unwrap().is_some());
    }
    std::fs::remove_dir_all(dir).ok();
}
