use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use highway::HighwayHasher;
use mail_send::{SmtpClient, SmtpClientBuilder};
use tokio_rustls::client::TlsStream;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::forms::student::StudentSignUp;
use crate::models::students::Department;
use crate::models::users::{Role, User};
use crate::schema::users;

#[derive(Clone)]
pub struct SiteState {
    pub connection: Pool<ConnectionManager<PgConnection>>,
    pub bulk_hasher: HighwayHasher,
    pub image_dir: PathBuf,
    pub mailer: Arc<Mutex<SmtpClient<TlsStream<TcpStream>>>>,
}

impl SiteState {
    pub async fn init() -> anyhow::Result<Self> {
        let email_client = SmtpClientBuilder::new("smtp.gmail.com", 587)
            .implicit_tls(false)
            .credentials((
                env::var("GOOGLE_CLIENT_ID")?.as_str(),
                env::var("GOOGLE_SECRET")?.as_str(),
            ))
            .connect()
            .await?;
        let database_url = env::var("DATABASE_URL")?;
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder().test_on_check_out(true).build(manager)?;
        let num_super_admin: i64 = users::table
            .count()
            .filter(users::role.eq(Role::SUPER_ADMIN))
            .get_result(&mut pool.get()?)?;
        if num_super_admin < 1 {
            log::warn!("No SUPER ADMIN found, asking to create a new user");
            let name = inquire::Text::new("Enter your name")
                .with_validator(inquire::required!())
                .prompt()?;
            let dob = inquire::DateSelect::new("Enter your date of birth").prompt()?;
            let contact = inquire::Text::new("Enter your contact number")
                .with_validator(inquire::required!())
                .prompt()?;
            let email = inquire::Text::new("Enter your Email")
                .with_validator(inquire::required!())
                .prompt()?;
            let password = inquire::Password::new("Password:")
                .with_display_toggle_enabled()
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .with_validator(inquire::min_length!(10))
                .with_formatter(&|_| String::from("Input received"))
                .with_help_message("It is recommended to generate a new one only for this purpose")
                .with_custom_confirmation_error_message("The keys don't match.")
                .prompt()?;

            let mut req: User = StudentSignUp {
                name,
                dob,
                email,
                password,
                phone: contact,
                college: "SLIET".to_owned(),
                reg_no: "000000".to_owned(),
                dept: Department::CS,
            }
            .try_into()
            .unwrap();
            req.role = Role::SUPER_ADMIN;
            req.verified = true;
            diesel::insert_into(users::table)
                .values(req)
                .execute(&mut pool.get()?)?;
        }
        Ok(Self {
            connection: pool,
            bulk_hasher: HighwayHasher::default(),
            image_dir: env::var("IMAGE_URL").unwrap_or("images/".into()).into(),
            mailer: Arc::from(Mutex::from(email_client)),
        })
    }
}
