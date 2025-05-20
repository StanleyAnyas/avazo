use sqlx::{FromRow, MySqlPool};
use crate::functions::{check_code, hash_password};
use serde_with::{serde_as, base64::Base64};

#[derive(Debug, FromRow, serde::Deserialize, serde::Serialize)]
pub struct FoodDetail {
    pub title: String,
    pub description: String,
    pub is_free: bool,
    pub pickup_time: String,
    pub user_id: i32,
    pub image: String,
}

#[derive(Debug, FromRow, serde::Serialize)]
pub struct Food {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_free: Option<i8>,
    pub pickup_time: Option<String>,
    pub user_id: Option<i32>,
    pub image: Option<Vec<u8>> 
}

#[derive(Debug, FromRow, serde::Serialize)]
pub struct NewUserId{
    pub  id: Option<i32>
}

#[derive(Debug, FromRow, serde::Serialize, serde::Deserialize)]
pub struct NewUserDetails{
    pub email: String,
    password_hash: String,
    first_name: Option<String>,
    last_name: Option<String>,
    num_of_food_added: Option<String>,
    num_of_food_taken: Option<String>,
    email_verified: Option<i8>
} 

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginDetail {
    pub email: String,
    pub password_hash: String,
}

#[derive(serde::Serialize)]
pub struct UserDetails{
    id: Option<i32>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    num_of_food_added: Option<i32>,
    num_of_food_taken: Option<i32>,
    profile_image: Option<Vec<u8>>,
    email_verified: Option<i8>,
    pub password_hash: String
}
#[serde_as]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct PictureDetails{
    pub user_id: i32, 

    #[serde_as(as = "Option<Base64>")]
    pub profile_image: Option<Vec<u8>>
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserCodeDetails{
    pub user_code: String, 
    pub user_email: String
}

pub async fn insert_food(pool: &MySqlPool, food: &FoodDetail) -> Result<u64, sqlx::Error>{
    let result = sqlx::query!(
        r#"
            INSERT INTO foods (title, description, is_free, pickup_time, user_id, image)
            VALUES(?, ?, ?, ?, ?, ?)
        "#,
        food.title,
        food.description, 
        food.is_free,
        food.pickup_time, 
        food.user_id,
        food.image
    )
    .execute(pool)
    .await?;

    let last_id = result.last_insert_id();
    Ok(last_id)
}

pub async fn increment_user_food_count(pool: &MySqlPool, user_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE users SET num_of_food_added = num_of_food_added + 1 WHERE id = ?
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}   

pub async fn get_all_food(pool: &MySqlPool) -> Result<Vec<Food>, sqlx::Error> {
    let food = sqlx::query_as!(
        Food,
        r#"
            SELECT id, title, description, is_free, pickup_time, user_id, image FROM foods
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(food)
}

pub async fn check_if_email_exists(pool: &MySqlPool, email: String) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
            SELECT email FROM users WHERE email = ?
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

pub async fn create_new_user(pool: &MySqlPool, user_details: NewUserDetails) -> Result<i32, sqlx::Error> {
    let result = sqlx::query!(
        r#"
            INSERT INTO users (email, password_hash, last_name, first_name, num_of_food_added,
            num_of_food_taken) 
            VALUES (?, ?, ?, ?, ?, ?)
        "#, 
        user_details.email,
        hash_password(user_details.password_hash),
        user_details.last_name, 
        user_details.first_name,
        user_details.num_of_food_added, 
        user_details.num_of_food_taken
    ).execute(pool).await?;
    let user_id = result.last_insert_id();

    Ok(user_id.try_into().unwrap())
}

pub async fn login_user(pool: &MySqlPool, login_details: &LoginDetail) -> Result<Option<UserDetails>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserDetails,
        r#"
            SELECT id, email, last_name, first_name, num_of_food_added,
            num_of_food_taken, profile_image, password_hash, email_verified FROM users WHERE email = ?
        "#,
        login_details.email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn delete_food(pool: &MySqlPool, id: i32) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            DELETE FROM foods WHERE id = ?
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn edit_profile_picture(pool: &MySqlPool, picture: &PictureDetails) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET profile_image = ? WHERE id = ?
        "#,
        picture.profile_image,
        picture.user_id
    ).execute(pool).await?;

    Ok(())
}

pub async fn verify_user_code(pool: &MySqlPool, user_code: &UserCodeDetails) -> Result<bool, sqlx::Error> {
    let code_db = sqlx::query!(
        r#"
            SELECT code_pass FROM users WHERE email = ?
        "#,
        user_code.user_email
    ).fetch_one(pool).await?;

    Ok(check_code(&user_code.user_code, code_db.code_pass.expect("REASON")))
}

pub async fn update_verified(pool: &MySqlPool, user_email: &String) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET email_verified = 1 WHERE email = ?
        "#,
        user_email
    ).execute(pool).await?;

    Ok(())
}

pub async fn add_user_code(pool: &MySqlPool, code: String, user_email: &String) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET code_pass = ? WHERE email = ?
        "#, 
        code,
        user_email
    ).execute( pool).await?;

    Ok(())
}

//TRYHARDANDO