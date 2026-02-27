/// Скрипт для сброса пароля пользователя
/// Использование: cargo run --bin reset_password <username> <new_password>

use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Использование: cargo run --bin reset_password <username> <new_password>");
        std::process::exit(1);
    }
    
    let username = &args[1];
    let new_password = &args[2];
    
    // Валидация пароля
    if new_password.len() < 8 {
        eprintln!("Ошибка: пароль должен быть не менее 8 символов");
        std::process::exit(1);
    }
    
    let has_upper = new_password.chars().any(|c| c.is_uppercase());
    let has_lower = new_password.chars().any(|c| c.is_lowercase());
    let has_digit = new_password.chars().any(|c| c.is_numeric());
    
    if !has_upper || !has_lower || !has_digit {
        eprintln!("Ошибка: пароль должен содержать заглавные и строчные буквы, а также цифры");
        std::process::exit(1);
    }
    
    let db_path = "data/trello.db";
    let db_url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&db_url).await?;
    
    // Хеширование пароля
    let password_hash = bcrypt::hash(new_password, 12)?;
    
    // Обновление пароля
    let result = sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
        .bind(&password_hash)
        .bind(username)
        .execute(&pool)
        .await?;
    
    if result.rows_affected() == 0 {
        eprintln!("Ошибка: пользователь '{}' не найден", username);
        std::process::exit(1);
    }
    
    println!("✅ Пароль для пользователя '{}' успешно обновлён", username);
    
    Ok(())
}
