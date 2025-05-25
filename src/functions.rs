use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use rand::{self, Rng};
use lettre::{Message, SmtpTransport, Transport, message::header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use std::env;
use dotenvy::dotenv;

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();  
    hash
}

pub fn compare_password(inputted_password: &str, db_password: &str) -> bool {
    let parse_hash = PasswordHash::new(db_password);
    if let Ok(hash) = parse_hash {
        Argon2::default().verify_password(inputted_password.as_bytes(), &hash).is_ok()
    }else{
        false
    }
}

pub fn generate_code() -> String {
    let mut rng = rand::rng();
    let code = rng.random_range(100_000..1_000_000);
    code.to_string()
}


pub async fn send_mail(user_mail: &String, code: &String) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("EMAIL_USERNAME").expect("email username does not exist");
    let password = env::var("EMAIL_PASSWORD").expect("password not exist");
    let relay = env::var("EMAIL_SMTP").expect("smtp does not exist");

    let body = format!(r#"
        <html>
            <body>
                <div style="font-family: Arial; padding: 20px;">
                <h2 style="color: #2e7d32;">Verify Your Email - Avanzo</h2>
                <p>Thanks for signing up! Your verification code is:</p>
                <div style="font-size: 24px; font-weight: bold; background-color: #e8f5e9; padding: 10px; color: #1b5e20; border-radius: 8px;">
                    {code}
                </div>
                <p>If you didn’t request this, just ignore it.</p>
            </div>
            </body>
        </html>
    "#, code = code);
    // println!("{}", body);
    let email = Message::builder()
        .from("Avanzo app <ritrove@ritrove.com>".parse()?)
        .to(user_mail.parse()?)
        .subject("Verify your name")
        .header(ContentType::TEXT_HTML)
        .body(body)?;

    let creds = Credentials::new(username, password);

    let mailer = SmtpTransport::relay(&relay)?
    .credentials(creds)
    .build();

    mailer.send(&email)?;

    Ok(())
}

pub fn check_code(code_user: &String, code_db: String) -> bool{
    if code_db.eq(code_user) {
        return true; 
    }
    false
}

pub async fn send_goodbye_mail(user_mail: String) -> Result<(), Box<dyn std::error::Error>>{

    dotenv().ok();
    let username = env::var("EMAIL_USERNAME").expect("email username does not exist");
    let password = env::var("EMAIL_PASSWORD").expect("password not exist");
    let relay = env::var("EMAIL_SMTP").expect("smtp does not exist");

    let body = format!(r#"
        <html>
            <body>
                <div style="font-family: Arial; padding: 20px;">
                <h2 style="color: #2e7d32;">Goodbye message - Avanzo</h2>
                <p>We are sorry seeing you leave.</p>
                <p>Hopefully we will see you again.</p>
            </div>
            </body>
        </html>
    "#);
    // println!("{}", body);
    let email = Message::builder()
        .from("Avanzo app <ritrove@ritrove.com>".parse()?)
        .to(user_mail.parse()?)
        .subject("Goodbye message")
        .header(ContentType::TEXT_HTML)
        .body(body)?;

    let creds = Credentials::new(username, password);

    let mailer = SmtpTransport::relay(&relay)?
    .credentials(creds)
    .build();

    mailer.send(&email)?;

    Ok(())
}