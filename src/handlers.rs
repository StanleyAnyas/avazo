use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::{MySqlPool};
use crate::{db::{add_user_code, check_if_email_exists, create_new_user, delete_food, edit_profile_picture, get_all_food, increment_user_food_count, insert_food, login_user, update_verified, verify_user_code, FoodDetail, LoginDetail, NewUserDetails, PictureDetails, UserCodeDetails}, functions::{compare_password, generate_code, send_mail}};

#[derive(serde::Deserialize)]
struct FoodId{
    food_id: i32
}

#[derive(serde::Deserialize)]
struct VerifyMail{
    user_mail: String
}

#[get("/get_food_list")]
async fn get_food_list(pool: web::Data<MySqlPool>) -> impl Responder{
    match get_all_food(&pool).await {
        Ok(food) => HttpResponse::Ok().json(food),
        Err(err) => HttpResponse::InternalServerError().body(format!("failed to get food list: {}", err))
    }
}

#[post("/add_food")]
async fn add_food(pool: web::Data<MySqlPool>, food: web::Json<FoodDetail>) -> impl Responder{
    let food_data = food.into_inner();
    match insert_food(&pool, &food_data).await {
        Ok(id) => {
                if let Err(err) = increment_user_food_count(&pool, food_data.user_id).await {
                    return HttpResponse::InternalServerError()
                        .body(format!("There was an error: {}", err));
                }
                HttpResponse::Ok().json(id)
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    } 
}

#[post("/add_user")]
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

#[post("/delete_food_handler")]
async fn delete_food_handler(pool: web::Data<MySqlPool>, food_id: web::Json<FoodId>) -> impl Responder{
    match delete_food(&pool, food_id.food_id).await {
        Ok(_) => HttpResponse::Ok().body("Food deleted"),
        Err(err) => HttpResponse::InternalServerError().body(format!("There was an error: {}", err))
    }
}

#[post("/login_user_handler")]
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

#[get("/verify_code")]
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

#[get("/send_verify_mail")]
async fn send_verify_mail(pool: web::Data<MySqlPool>, email: web::Query<VerifyMail>) -> impl Responder{
    let code = generate_code();
    println!("{}", code);
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