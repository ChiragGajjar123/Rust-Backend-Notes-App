use aws_config::Region;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use crate::config::Config;
use crate::errors::AppError;

pub async fn send_password_reset_email(
    to_email: &str,
    otp_code: &str,
    config: &Config,
) -> Result<(), AppError> {
    let from_email = match &config.aws_ses_from_email {
        Some(email) if !email.trim().is_empty() => email.trim(),
        _ => {
            tracing::info!(
                "AWS_SES_FROM_EMAIL not set. DEV MODE: Password reset OTP for [{}] is: {}",
                to_email,
                otp_code
            );
            return Ok(());
        }
    };

    let region_provider = Region::new(config.aws_region.clone());
    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = aws_sdk_sesv2::Client::new(&sdk_config);

    let subject = Content::builder()
        .data("Password Reset Verification Code - Notes App")
        .charset("UTF-8")
        .build()
        .map_err(|e| AppError::InternalServerError(format!("Failed to build email subject: {}", e)))?;

    let html_body_content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Password Reset</title>
</head>
<body style="font-family: Arial, sans-serif; background-color: #f4f6f8; padding: 20px; color: #333;">
    <div style="max-width: 500px; margin: 0 auto; background: #ffffff; padding: 30px; border-radius: 8px; box-shadow: 0 4px 10px rgba(0,0,0,0.05);">
        <h2 style="color: #4f46e5; margin-top: 0;">Password Reset Request</h2>
        <p>You requested a password reset for your Notes App account.</p>
        <p>Your 6-digit verification code is:</p>
        <div style="font-size: 32px; font-weight: bold; letter-spacing: 6px; color: #1e293b; background: #f1f5f9; padding: 15px; text-align: center; border-radius: 6px; margin: 20px 0;">
            {}
        </div>
        <p style="font-size: 14px; color: #64748b;">This code is valid for {} minutes. If you did not request this password reset, please ignore this email.</p>
    </div>
</body>
</html>"#,
        otp_code, config.password_reset_expiration_mins
    );

    let text_body_content = format!(
        "Your password reset verification code for Notes App is: {}\n\nThis code will expire in {} minutes.",
        otp_code, config.password_reset_expiration_mins
    );

    let body = Body::builder()
        .html(
            Content::builder()
                .data(html_body_content)
                .charset("UTF-8")
                .build()
                .map_err(|e| AppError::InternalServerError(format!("Failed to build HTML body: {}", e)))?,
        )
        .text(
            Content::builder()
                .data(text_body_content)
                .charset("UTF-8")
                .build()
                .map_err(|e| AppError::InternalServerError(format!("Failed to build Text body: {}", e)))?,
        )
        .build();

    let email_message = Message::builder()
        .subject(subject)
        .body(body)
        .build();

    let email_content = EmailContent::builder()
        .simple(email_message)
        .build();

    let destination = Destination::builder()
        .to_addresses(to_email)
        .build();

    match client
        .send_email()
        .from_email_address(from_email)
        .destination(destination)
        .content(email_content)
        .send()
        .await
    {
        Ok(output) => {
            tracing::info!(
                "Successfully dispatched password reset email via AWS SES to {}. MessageId: {:?}",
                to_email,
                output.message_id
            );
            Ok(())
        }
        Err(err) => {
            tracing::error!("Failed to send email via AWS SES to {}: {:?}", to_email, err);
            tracing::info!(
                "FALLBACK DEV LOG: Password reset OTP for [{}] is: {}",
                to_email,
                otp_code
            );
            Ok(())
        }
    }
}
