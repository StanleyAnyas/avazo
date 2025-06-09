
use actix_web::{delete, get, patch, post, web::{self}, Responder};
// use rand::rand_core::impls;
// use rand::rand_core::impls;
use sqlx::{MySqlPool};
use crate::{db::{add_user_code, change_email_verified, check_if_email_exists, check_if_user_has_reserve, create_new_user, delete_food, delete_user_account, delete_verification_code, edit_profile_picture, edit_reservation, edit_user_profile, get_active_donation, get_active_reserve, get_all_donations, get_all_food, get_email, get_food_detail, get_reservation_details, get_user_email, get_user_profile, get_user_reservations, increment_user_food_count, insert_food, login_user, make_reserve, mark_user_reserve, update_donation, update_verified, verify_user_code, EditUserDetails, FoodDetail, FoodDetail2, LoginDetail, NewUserDetails, PictureDetails, PicturePayload, ReserveDetails, UserCodeDetails}, functions::{compare_password, failure, generate_code, send_goodbye_mail, send_mail, success}};

#[derive(serde::Deserialize)]
struct FoodId{
    food_id: i32
}
// #[derive(serde::Deserialize)]
// struct UserId{
//     user_id: i32
// }
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MajesticRes{
    pub user_email: String
}

#[get("/foods")] // tested
async fn get_food_list(pool: web::Data<MySqlPool>) -> impl Responder{
    match get_all_food(&pool).await {
        Ok(food) => success("sucessfull", food),
        Err(err) => failure(format!("failed to get food list: {}", err)) 
    }
}

#[post("/foods")] //tested
async fn add_food(pool: web::Data<MySqlPool>, food: web::Json<FoodDetail>) -> impl Responder{
    let food_data = food.into_inner();
    match insert_food(&pool, &food_data).await {
        Ok(id) => {
                if let Err(err) = increment_user_food_count(&pool, food_data.user_id).await {
                    return failure(format!("There was an error: {}", err));
                }
                // println!("aggiunto cibo");
                success("food inserted successfully", id)
        }
        Err(err) => failure(format!("There was an error: {}", err))
    } 
}

#[post("/users")] //tested
async fn add_user(pool: web::Data<MySqlPool>, user_details: web::Json<NewUserDetails>) -> impl Responder{
    let user_data = user_details.into_inner();
    match check_if_email_exists(&pool, user_data.email.clone()).await {
        Ok(exits) =>{
            if !exits {
                match create_new_user(&pool, user_data).await {
                    Ok(user_id) => success("user added successfully", user_id),
                    Err(err) => failure(format!("There was an error {}", err))
                }
            }else{
                failure(format!("email already exists"))
            }
        }
        Err(err) => failure(format!("There was an error {}", err))
    }
    
}

#[delete("/foods/{food_id}")] // tested
async fn delete_food_handler(pool: web::Data<MySqlPool>, path: web::Path<FoodId>) -> impl Responder{
    let food_id = path.into_inner();
    match delete_food(&pool, food_id.food_id).await {
        Ok(_) => success("Food deleted", None::<()>),
        Err(err) => failure(format!("There was an error: {}", err))
    }
}

#[post("/login")] // tested
async fn login_user_handler(pool: web::Data<MySqlPool>, user: web::Json<LoginDetail>) -> impl Responder{
    let user_pass = user.into_inner();
    match login_user(&pool, &user_pass).await {
        Ok(Some(user_from_db)) => {
            if compare_password(&user_pass.password_hash, &user_from_db.password_hash) {
                success("login successfully", user_from_db)
            }else{
                failure(format!("incorrect password"))
            }
        }
        Ok(None) => failure(format!("incorrect password")),
        Err(err) => failure(format!("There was an error: {}", err))
    }
}

#[get("/users/{id}")] // tested
async fn get_user_profile_details(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let user_id = path.into_inner();
    match get_user_profile(&pool, user_id).await {
        Ok(user_details) => success("successful", user_details),
        Err(err) => failure(format!("There was an error getting user details: {}", err))
    }
}

#[get("/foods/{id}")] //tested
async fn get_food_profile_details(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let food_id = path.into_inner();
    match get_food_detail(&pool, food_id).await {
        Ok(food_details) => success("successfull", food_details),
        Err(err) => failure(format!("There was an error getting user details: {}", err))
    }
}

#[patch("/users/{user_id}/picture")] // tested
async  fn edit_profile_pic(pool: web::Data<MySqlPool>, path: web::Path<i32>, payload: web::Json<PicturePayload>) -> impl Responder{ 
    let user_id = path.into_inner();
    let profile_image = payload.profile_image.clone();
    let user_pic = PictureDetails { user_id, profile_image };
    match edit_profile_picture(&pool, &user_pic).await {
        Ok(_) => success("picture added", None::<()>),
        Err(err) => failure(format!("There was an error: {}", err))
    }
}

#[post("/users/{user_id}/verify")] // tested
async fn verify_code(pool: web::Data<MySqlPool>, path: web::Path<i32>, code: web::Json<UserCodeDetails>) -> impl Responder{
    let user_id = path.into_inner();
    let details = UserCodeDetails{
        user_code: code.user_code.clone(),
        user_id,
        user_email: code.user_email.clone()
    };
    match verify_user_code(&pool, &details).await {
        Ok(exist) => {
            if exist {
                match update_verified(&pool, &details.user_email).await {
                    Ok(_) => {
                        match delete_verification_code(&pool, &details.user_email).await {
                            Ok(_) => success("email verified", None::<()>),
                            Err(err) => failure(format!("there was error: {}", err))
                        }
                    }
                    Err(err) => failure(format!("an errror occured: {}", err))
                }
            }else{
                failure(format!("wrong code"))
            }
        }
        Err(err) => failure(format!("there was err: {}", err))
    }
}


#[post("/users/{user_id}/mail")] // tested
async fn send_verify_mail(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let code = generate_code();

    let user_id = path.into_inner();
    match get_user_email(&pool, &user_id).await {
        Ok(user_mail) =>{
            match add_user_code(&pool, code.clone(), user_id).await {
                Ok(_) => {
                    match send_mail(&user_mail, &code).await {
                        Ok(_) => success("email sent successfully", None::<()>), 
                        Err(err) => failure(format!("there was an error qui: {}", err))
                    }   
                }
                
                Err(err) => failure(format!("there was an error qui: {}", err))
            }
        }
        Err(err) => failure(format!("there was error verifying user email: {}", err))
    } 
}

#[delete("/users/{id}/profile")] //tested
async fn delete_user(pool: web::Data<MySqlPool>, path: web::Path<i32>, user_details: web::Json<MajesticRes>) -> impl Responder{
    let id = path.into_inner();
    let user_mail = user_details.user_email.clone();
    match delete_user_account(&pool, id, &user_mail).await {
        Ok(_) => {
            match send_goodbye_mail(user_mail).await {
                Ok(_) => success("user deleted successfully", None::<()>),
                Err(err) => failure(format!("couldn't send mail: {}", err))
            }
        }
        Err(err) => failure(format!("there was an error: {}", err))
    }
}


#[patch("/users/{id}/profile")] // tested
async fn edit_profile(pool: web::Data<MySqlPool>, path: web::Path<i32>, user_edit_details: web::Json<EditUserDetails>) -> impl Responder {
    
    let user_id = path.into_inner();
    let user_edit = EditUserDetails {
        user_id: user_id,
        first_name: user_edit_details.first_name.clone(),
        last_name: user_edit_details.last_name.clone(),
        email: user_edit_details.email.clone(),
    };

    match get_email(&pool, &user_id, &user_edit.email).await {
        Ok(is_same) =>{
            match edit_user_profile(&pool, &user_edit).await {
                Ok(_) => {
                    if !is_same {
                        match change_email_verified(&pool, &user_edit_details.email).await {
                            Ok(_) => success("successful", user_edit_details),
                            Err(err) => failure(format!("there was an error {}", err))
                        }
                    }else{
                        return success("successful", user_edit_details)
                    }
                }
                Err(err) => failure(format!("there was an error: {}", err))
            }
        }
        Err(err) => failure(format!("there was an error: {}", err))
    }
   
}

#[get("/users/{id}/donations")] // tested
async fn get_donations(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder {
    let user_id = path.into_inner();
    match get_all_donations(&pool, user_id).await {
        Ok(all_donations) => success("successfull", all_donations),
        Err(err) => failure(format!("there was an error: {}", err))
    }
}

#[get("/users/{id}/reservations")] // tested
async fn get_reserves(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let user_id = path.into_inner();
    match get_user_reservations(&pool, user_id).await {
       Ok(all_reserves) =>{
            success("successfull", all_reserves)
       }
       Err(err) => failure(format!("error getting all reserves: {}", err)) 
    }
}

#[patch("/donations")] // tested
async fn edit_donation(pool: web::Data<MySqlPool>, food_edit_details: web::Json<FoodDetail2>) -> impl Responder {
    let food_edit_details = food_edit_details.into_inner();
    match update_donation(&pool, &food_edit_details).await {
        Ok(_) => return success("successfull", food_edit_details),
        Err(err) => failure(format!("there was an error: {}", err))
    }
}

#[get("/donations/{id}/active")] // tested
async fn get_user_active_donations(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let user_id = path.into_inner();
    match get_active_donation(&pool, user_id).await {
        Ok(all_donations) => success("successfull", all_donations),
        Err(err) => failure(format!("there was an error getting user active donations: {}", err))
    }
}

#[post("/users/{id}/reserve")] // tested
async fn make_user_reserve(pool: web::Data<MySqlPool>, path: web::Path<i32>, reserve_details: web::Json<ReserveDetails>) ->impl Responder{
    let id = path.into_inner();
    match check_if_user_has_reserve(&pool, id.clone()).await {
        Ok(has) => {
            if has {
                return failure(format!("already has a reservation"))
            }
                match make_reserve(&pool, reserve_details.into_inner()).await {
                    Ok(_) =>{
                        match mark_user_reserve(&pool, id).await {
                            Ok(id) => {
                                match get_reservation_details(&pool, id).await {
                                    Ok(reserve_details) => {
                                        return success("successfull", reserve_details)
                                    }
                                    Err(err) => failure(format!("there was an error getting reservation {}", err))
                                }
                            }
                            Err(err) => failure(format!("error marking reservation but reservation made: {}", err))
                        }
                    }
                    Err(err) => failure(format!("error making reserve: {}", err))
                }
        }
        Err(err) => failure(format!("error making reserve: {}", err))
    }
}

#[delete("/users/{id}/reserve")] // tested
async fn cancel_reserve(pool: web::Data<MySqlPool>, path: web::Path<i32>, reserve_details: web::Json<ReserveDetails>) -> impl Responder{
    let user_id = path.into_inner();
    let reserve = ReserveDetails {
        food_id: reserve_details.food_id,
        user_id
    };
    match edit_reservation(&pool, reserve).await { 
        Ok(_) => success("reservation cancelled", None::<()>),
    Err(err) => failure(format!("there was an error: {}", err))
    }
}

#[get("/users/{id}/reserve")] // tested
async fn get_user_active_reserve(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> impl Responder{
    let user_id = path.into_inner();
    match get_active_reserve(&pool, user_id).await {
        Ok(active_reserve) => success("successfull", active_reserve),
        Err(err) => failure(format!("error getting active reservation: {}", err))
    }
}

// App mobile in Compose Multiplatform (AMCM)