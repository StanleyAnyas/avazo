use sqlx::{FromRow, MySqlPool};
use crate::functions::{check_code, hash_password};
use serde_with::{serde_as, base64::Base64};

#[derive(Debug, FromRow, serde::Deserialize, serde::Serialize)]
pub struct FoodDetail {
    pub title: String,
    pub description: String,
    pub is_free: bool,
    pub pickup_time: String,
    pub pickup_address: String,
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
    pub pickup_address: Option<String>,
    pub user_id: Option<i32>,
    pub image: Option<Vec<u8>>,
    pub status: Option<String>
}

#[derive(Debug, FromRow, serde::Serialize)]
pub struct AllReserves{
    pub food_id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub first_name: Option<String>,
    pub image: Option<Vec<u8>>
}

#[derive(Debug, FromRow, serde::Serialize)]
pub struct ActiveReserve{
    pub food_id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub first_name: Option<String>,
    pub image: Option<Vec<u8>>,
    pub pickup_time: Option<String>,
    pub pickup_address: Option<String>
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditUserDetails{
    pub user_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_was_changed: bool
}

#[derive(serde::Deserialize, serde::Serialize, FromRow, Debug,)]
pub struct ReserveDetails{
    pub user_id: i32,
    pub food_id: i32
}

pub async fn insert_food(pool: &MySqlPool, food: &FoodDetail) -> Result<u64, sqlx::Error>{
    let result = sqlx::query!(
        r#"
            INSERT INTO foods (title, description, is_free, pickup_time, user_id, image, pickup_address)
            VALUES(?, ?, ?, ?, ?, ?, ?)
        "#,
        food.title,
        food.description,
        food.is_free,
        food.pickup_time,
        food.user_id,
        food.image,
        food.pickup_address
    )
    .execute(pool)
    .await?;

    let last_id = result.last_insert_id();
    Ok(last_id)
}

pub async fn increment_user_food_count(pool: &MySqlPool, user_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE users SET num_of_food_added = num_of_food_added + 1 WHERE id = ? AND is_active = 1 
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
            SELECT id, title, description, is_free, pickup_time, user_id, image, pickup_address, status FROM foods
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(food)
}

pub async fn check_if_email_exists(pool: &MySqlPool, email: String) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
            SELECT email FROM users WHERE email = ? AND is_active = 1
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
            num_of_food_taken, profile_image, password_hash, email_verified FROM users WHERE email = ? AND is_active = 1
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
            UPDATE users SET profile_image = ? WHERE id = ? AND is_active = 1
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
            UPDATE users SET email_verified = 1 WHERE email = ? AND is_active = 1
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

//edit_user_profile (check if the email was changed, if yes changed the verified email to false)
pub async fn edit_user_profile(pool: &MySqlPool, edit_user_details: &EditUserDetails) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET first_name = ?, last_name = ?, email = ? WHERE id = ? And is_active = 1
        "#,
        edit_user_details.first_name,
        edit_user_details.last_name,
        edit_user_details.email,
        edit_user_details.user_id,
    ).execute(pool).await?;

    Ok(())
}

pub async fn change_email_verified(pool: &MySqlPool, user_mail: &String) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET email_verified = 0 WHERE email = ?
        "#,
        user_mail
    ).execute(pool).await?;
    Ok(())
}

pub async fn delete_user_account(pool: &MySqlPool, user_id: i32) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET is_active = 0 WHERE id = ?
        "#,
        user_id
    ).execute(pool).await?;

    Ok(())
}

pub async fn make_reserve(pool: &MySqlPool, reserve_details: ReserveDetails) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            INSERT INTO reservations (user_id, food_id) VALUES (?, ?)
        "#,
        reserve_details.user_id,
        reserve_details.food_id
    ).execute(pool).await?;

    Ok(())
}

pub async fn mark_user_reserve(pool: &MySqlPool, user_id: i32) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE users SET has_reserve = 1 WHERE id = ?
        "#,
        user_id
    ).execute(pool).await?;

    Ok(())
}

pub async fn check_if_user_has_reserve(pool: &MySqlPool, user_id: i32) -> Result<bool, sqlx::Error>{
   let result = sqlx::query_scalar!(
    r#"
        SELECT has_reserve FROM users WHERE id = ? AND has_reserve = 1
    "#,
    user_id
   ).fetch_optional(pool).await?;

   Ok(result.is_some())
}

pub async fn get_user_reservations(pool: &MySqlPool, user_id: i32) -> Result<Vec<AllReserves>, sqlx::Error>{
    let all_reserve = sqlx::query_as!(
        AllReserves, 
        r#"
            SELECT r.food_id, f.title, f.description, u.first_name, f.image FROM reservations r
            INNER JOIN users u on u.id = r.user_id INNER JOIN foods f on
            f.id = r.food_id WHERE r.user_id = ?
        "#,
        user_id
    ).fetch_all(pool).await?;

    Ok(all_reserve)
}

// get_active_reserve
pub async fn get_active_reserve(pool: &MySqlPool, user_id: i32) ->Result<ActiveReserve, sqlx::Error>{
    let active_reserve = sqlx::query_as!(
        ActiveReserve,
        r#"
            SELECT food_id, title, description, first_name, image, 
            pickup_time, pickup_address FROM reservations r
            INNER JOIN users u on u.id = r.user_id 
            INNER JOIN foods f on f.id = r.food_id
            WHERE r.user_id = ? AND u.has_reserve = 1
            AND r.status = 'active'
        "#,
        user_id
    ).fetch_one(pool).await?;

    Ok(active_reserve)
}

//get_all_donations
pub async fn get_all_donations(pool: &MySqlPool, user_id: i32) ->Result<Vec<Food>, sqlx::Error>{
    let all_donations = sqlx::query_as!(
        Food,
        r#"
            SELECT * 
            FROM foods
            WHERE user_id = ?
        "#,
        user_id
    ).fetch_all(pool)
    .await?;

    Ok(all_donations)
}

//update_donation
pub async fn update_donation(pool: &MySqlPool, food: &FoodDetail) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE foods 
            SET title = ?, description = ?, is_free = ?, pickup_time = ?, pickup_address = ?, image = ?
            WHERE id = ?
        "#,
        food.title,
        food.description,
        food.is_free,
        food.pickup_time,
        food.pickup_address,
        food.image,
        food.user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// get_active_donations
pub async fn get_active_donation(pool: &MySqlPool, user_id: i32) -> Result<Vec<Food>, sqlx::Error>{
    let active_donation = sqlx::query_as!(
        Food,
        r#"
            SELECT * from foods WHERE user_id = ? and status = 'active'
        "#,
        user_id

    ).fetch_all(pool).await?;

    Ok(active_donation)
}

pub async fn edit_reservation(pool: &MySqlPool, reserve_details: ReserveDetails) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
            UPDATE reservations SET status = 'cancelled'
            WHERE user_id = ? OR food_id = ?
        "#,
        reserve_details.user_id,
        reserve_details.food_id
    ).execute(pool).await?;

    sqlx::query!(
        r#"
            UPDATE users SET has_reserve = 0
            WHERE id = ?
        "#,
        reserve_details.user_id
    ).execute(pool).await?;

    Ok(())
}