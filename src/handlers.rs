
use actix_web::{get, post, web::{self}, HttpResponse, Responder};
// use rand::rand_core::impls;
// use rand::rand_core::impls;
use sqlx::{MySqlPool};
use crate::{db::{add_user_code, change_email_verified, check_if_email_exists, check_if_user_has_reserve, create_new_user, delete_food, delete_user_account, edit_profile_picture, edit_reservation, edit_user_profile, get_active_donation, get_active_reserve, get_all_donations, get_all_food, get_user_reservations, increment_user_food_count, insert_food, login_user, make_reserve, mark_user_reserve, update_donation, update_verified, verify_user_code, EditUserDetails, FoodDetail, LoginDetail, NewUserDetails, PictureDetails, ReserveDetails, UserCodeDetails}, functions::{compare_password, generate_code, send_goodbye_mail, send_mail}};

#[derive(serde::Deserialize)]
struct FoodId{
    food_id: i32
}
#[derive(serde::Deserialize)]
struct UserId{
    user_id: i32
}
#[derive(serde::Deserialize, serde::Serialize)]
struct MajesticRes{
    user_id: i32,
    user_email: String
}


#[derive(serde::Deserialize)]
struct VerifyMail{
    user_mail: String
}

#[get("/get_food_list")] // tested
async fn get_food_list(pool: web::Data<MySqlPool>) -> impl Responder{
    match get_all_food(&pool).await {
        Ok(food) => HttpResponse::Ok().json(food),
        Err(err) => HttpResponse::InternalServerError().body(format!("failed to get food list: {}", err))
    }
}

#[post("/add_food")] //tested
async fn add_food(pool: web::Data<MySqlPool>, food: web::Json<FoodDetail>) -> impl Responder{
    let food_data = food.into_inner();
    match insert_food(&pool, &food_data).await {
        Ok(id) => {
                if let Err(err) = increment_user_food_count(&pool, food_data.user_id).await {
                    return HttpResponse::InternalServerError()
                        .body(format!("There was an error: {}", err));
                }
                println!("aggiunto cibo");
                HttpResponse::Ok().json(id)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    } 
}

#[post("/add_user")] //tested
async fn add_user(pool: web::Data<MySqlPool>, user_details: web::Json<NewUserDetails>) -> impl Responder{
    let user_data = user_details.into_inner();
    match check_if_email_exists(&pool, user_data.email.clone()).await {
        Ok(exits) =>{
            if !exits {
                match create_new_user(&pool, user_data).await {
                    Ok(user_id) => HttpResponse::Ok().json(user_id),
                    Err(err) => HttpResponse::InternalServerError().body(format!("There was an error {}", err))
                }
            }else{
                // HttpResponse::InternalServerError().body("email already in use")
                HttpResponse::Conflict().body("email already exists")
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error {}", err))
    }
    
}

#[post("/delete_food_handler")] //tested
async fn delete_food_handler(pool: web::Data<MySqlPool>, food_id: web::Json<FoodId>) -> impl Responder{
    match delete_food(&pool, food_id.food_id).await {
        Ok(_) => HttpResponse::Ok().body("Food deleted"),
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    }
}

#[post("/login_user_handler")] // tested
async fn login_user_handler(pool: web::Data<MySqlPool>, user: web::Json<LoginDetail>) -> impl Responder{
    let user_pass = user.into_inner();
    match login_user(&pool, &user_pass).await {
        Ok(Some(user_from_db)) => {
            if compare_password(&user_pass.password_hash, &user_from_db.password_hash) {
                HttpResponse::Ok().json(user_from_db)
            }else{
                HttpResponse::Unauthorized().body("incorrect password")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("incorrect password"),
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    }
}

#[post("/edit_profile_pic")]
async  fn edit_profile_pic(pool: web::Data<MySqlPool>, picture_details: web::Json<PictureDetails>) -> impl Responder{
    let user_pic = picture_details.into_inner();
    match edit_profile_picture(&pool, &user_pic).await {
        Ok(_) => HttpResponse::Ok().body("picture added"),
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    }
}

#[get("/verify_code")] // tested
async fn verify_code(pool: web::Data<MySqlPool>, code: web::Query<UserCodeDetails>) -> impl Responder{
    match verify_user_code(&pool, &code).await {
        Ok(exist) => {
            if exist {
                match update_verified(&pool, &code.user_email).await {
                    Ok(_) => HttpResponse::Ok().body("email verified"),

                    Err(err) => HttpResponse::NotModified().body(format!("an errror occured: {}", err))
                }
                
            }else{
                HttpResponse::NotFound().body("wrong code")
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("there was err: {}", err))
    }
}

#[get("/send_verify_mail")] //tested
async fn send_verify_mail(pool: web::Data<MySqlPool>, email: web::Query<VerifyMail>) -> impl Responder{
    let code = generate_code();
    // println!("{}", code);
    match add_user_code(&pool, code.clone(), &email.user_mail).await {
        Ok(_) => {
            match send_mail(&email.user_mail, &code).await {
                Ok(_) => HttpResponse::Ok().body("email sent successfully"), 
                Err(err) =>HttpResponse::ExpectationFailed().body(format!("there was an error quiiii: {}", err))
            }   
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error qui: {}", err))
    }
}

#[get("/delete_user")]
async fn delete_user(pool: web::Data<MySqlPool>, user_details: web::Query<MajesticRes>) -> impl Responder{
    let id = user_details.user_id;
    let user_mail = user_details.user_email.clone();
    match delete_user_account(&pool, id).await {
        Ok(_) => {
            match send_goodbye_mail(user_mail).await {
                Ok(_) => HttpResponse::Ok().body("user deleted successfully"),
                Err(err) => HttpResponse::ExpectationFailed().body(format!("couldn't send mail: {}", err))
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error: {}", err))
    }
}

#[post("/edit_user_profile")]
async fn edit_profile(pool: web::Data<MySqlPool>, user_edit_details: web::Json<EditUserDetails>) -> impl Responder {
    let user_edit_details = user_edit_details.into_inner();
    match edit_user_profile(&pool, &user_edit_details).await {
        Ok(_) => {
            if user_edit_details.email_was_changed {
                match change_email_verified(&pool, &user_edit_details.email).await {
                    Ok(_) => HttpResponse::Ok().json(user_edit_details),
                    Err(err) => HttpResponse::InternalServerError().body(format!("there was an error {}", err))
                }
            }else{
                return HttpResponse::Ok().json(user_edit_details)
            }
            
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error: {}", err))
    }
}

#[get("/check_if_user_has_reserve")]
async fn has_reserve(pool: web::Data<MySqlPool>, reserve_details: web::Query<ReserveDetails>) ->impl Responder{
    let id = reserve_details.user_id;
    match check_if_user_has_reserve(&pool, id.clone()).await {
        Ok(has) => {
            if has {
                return HttpResponse::NotAcceptable().body("already has a reservation")
            }
                match make_reserve(&pool, reserve_details.into_inner()).await {
                    Ok(_) =>{
                        match mark_user_reserve(&pool, id).await {
                            Ok(_) => HttpResponse::Ok().body("reservation made"),
                            Err(err) => HttpResponse::InternalServerError().body(format!("error marking reservation but reservation made: {}", err))
                        }
                    }
                    Err(err) => HttpResponse::InternalServerError().body(format!("error making reserve: {}", err))
                }
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("error making reserve: {}", err))
    }
}

#[get("/get_all_donations")]
async fn get_donations(pool: web::Data<MySqlPool>, user_info: web::Query<UserId>) -> impl Responder {

    match get_all_donations(&pool, user_info.user_id).await {
        Ok(all_donations) => HttpResponse::Ok().json(all_donations),
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error: {}", err))
    }
}

#[post("/get_user_reservations")]  
async fn get_reserves(pool: web::Data<MySqlPool>, user_id: web::Query<UserId>) -> impl Responder{
    match get_user_reservations(&pool, user_id.user_id).await {
       Ok(all_reserves) =>{
           HttpResponse::Ok().json(all_reserves)
       }
       Err(err) => HttpResponse::InternalServerError().body(format!("error getting all reserves: {}", err)) 
    }
}

#[post("/edit_donation")]
async fn edit_donation(pool: web::Data<MySqlPool>, food_edit_details: web::Json<FoodDetail>) -> impl Responder {
    let food_edit_details = food_edit_details.into_inner();
    match update_donation(&pool, &food_edit_details).await {
        Ok(_) => return HttpResponse::Ok().json(food_edit_details),
        Err(err) => HttpResponse::NotModified().body(format!("there was an error: {}", err))
    }
}

#[get("/get_active_donations")]
async fn get_user_active_donations(pool: web::Data<MySqlPool>, user_info: web::Query<UserId>) -> impl Responder{
    match get_active_donation(&pool, user_info.user_id).await {
        Ok(all_donations) => HttpResponse::Ok().json(all_donations),
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error getting user active donations: {}", err))
    }
}

#[get("/cancel_reserve")]
async fn cancel_reserve(pool: web::Data<MySqlPool>, reserve_details: web::Json<ReserveDetails>) -> impl Responder{
    match edit_reservation(&pool, reserve_details.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("reservation cancelled"),
        Err(err) => HttpResponse::InternalServerError().body(format!("there was an error: {}", err))
    }
}

#[get("/get_user_active_reserve")]
async fn get_user_active_reserve(pool: web::Data<MySqlPool>, user_info: web::Query<UserId>) -> impl Responder{
    match get_active_reserve(&pool, user_info.user_id).await {
        Ok(active_reserve) => HttpResponse::Ok().json(active_reserve),
        Err(err) => HttpResponse::InternalServerError().body(format!("error getting active reservation: {}", err))
    }
}