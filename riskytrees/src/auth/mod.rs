use std::env;
use hmac::{Hmac, Mac};
use jwt::claims::SecondsSinceEpoch;
use openidconnect::AccessToken;
use openidconnect::OAuth2TokenResponse;
use sha2::Sha256;
use jwt;
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use openidconnect::core::{CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreJsonWebKeyType};
use openidconnect::{
    AuthorizationCode,
    AuthUrl,
    IssuerUrl,  
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl, ExtraTokenFields, IdToken, EmptyAdditionalClaims, Nonce,
    JsonWebKeySet
};
use openidconnect::reqwest::async_http_client;

use crate::database;
use crate::database::CSRFValidationResult;
use crate::database::Tenant;
use crate::errors::{AuthError, self};

pub struct AuthRequestData {
   pub url: openidconnect::url::Url,
   pub csrf_token: CsrfToken,
   pub nonce: Nonce
}

pub struct ApiKey {
    pub email: String,
    pub tenants: Vec<Tenant>
}


pub fn start_flow(provider: String) -> Result<AuthRequestData, AuthError> {
    let oidc_auth_url = match provider.as_str() {
        "google" => "RISKY_TREES_GOOGLE_AUTH_URL",
        "github" => "RISKY_TREES_GITHUB_AUTH_URL",
        _ => ""
    };

    let oidc_redirect_url = match provider.as_str() {
        "google" => "RISKY_TREES_GOOGLE_REDIRECT_URL",
        "github" => "RISKY_TREES_GITHUB_REDIRECT_URL",
        _ => ""
    };

    let oidc_client_secret = match provider.as_str() {
        "google" => "RISKY_TREES_GOOGLE_CLIENT_SECRET",
        "github" => "RISKY_TREES_GITHUB_CLIENT_SECRET",
        _ => ""
    };

    let oidc_client_id = match provider.as_str() {
        "google" => "RISKY_TREES_GOOGLE_CLIENT_ID",
        "github" => "RISKY_TREES_GITHUB_CLIENT_ID",
        _ => ""
    };

    let oidc_issuer_url = match provider.as_str() {
        "google" => "RISKY_TREES_GOOGLE_ISSUER_URL",
        "github" => "RISKY_TREES_GITHUB_ISSUER_URL",
        _ => ""
    };



    let auth_url = AuthUrl::new(env::var(oidc_auth_url).expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting auth URL".to_owned()
    })?;
    let redirect_url = RedirectUrl::new(env::var(oidc_redirect_url).expect("to exist").to_string());


    match redirect_url {
        Ok(redirect_url) => {
            let client =
            openidconnect::core::CoreClient::new(
                ClientId::new(env::var(oidc_client_id).expect("to exist").to_string()),
                Some(ClientSecret::new(env::var(oidc_client_secret).expect("to exist").to_string())),
                IssuerUrl::new(env::var(oidc_issuer_url).expect("to exist").to_string()).expect("Should be able to create Issuer URL"),
                AuthUrl::new(env::var(oidc_auth_url).expect("to exist").to_string()).expect("Should be able to create auth URL"),
                None, None, JsonWebKeySet::new(vec![])

            )
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(redirect_url);


            // Generate the full authorization URL.
            let (auth_url, csrf_token, nonce) = client
                .authorize_url(openidconnect::core::CoreAuthenticationFlow::AuthorizationCode,
                    CsrfToken::new_random,
                    openidconnect::Nonce::new_random,)
                // Set the desired scopes.
                .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
                .url();

            Ok(AuthRequestData {
                url: auth_url,
                csrf_token: csrf_token,
                nonce: nonce
            })
        },
        Err(err) => {
            Err(errors::AuthError {
                message: "No redirect URL".to_owned()
            })
        }
    }

}

pub async fn exchange_github_access_token_for_email(access_token: &AccessToken) -> Result<String, AuthError> {
    let request = octocrab::Octocrab::builder()
    .user_access_token(secrecy::SecretString::from(access_token.secret().as_str()))
    .build();

    match request {
        Ok(request) => {
            let user: Result<octocrab::models::UserProfile, octocrab::Error> = request.get("/user", None::<&()>)
        .await;
            match user {
                Ok(user_profile) => {
                    match user_profile.email {
                        Some(email) => {
                            return Ok(email)
                        },
                        None => Err(AuthError { message: "No email associated with user".to_owned() })
                    }
                },
                Err(err) => Err(AuthError { message: "GitHub API call to /user failed".to_owned() })
            }
        },
        Err(err) => Err(AuthError { message: "Failed to build request".to_owned() })
    }
}

// Returns email if trade succeeds
pub async fn trade_token(code: &String, validation_result: CSRFValidationResult) -> Result<String, AuthError> {
    let oidc_auth_url = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_AUTH_URL",
        true => "RISKY_TREES_GITHUB_AUTH_URL"
    };

    let oidc_redirect_url = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_REDIRECT_URL",
        true => "RISKY_TREES_GITHUB_REDIRECT_URL"
    };

    let oidc_jwks_url = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_JWKS_URL",
        true => "RISKY_TREES_GITHUB_JWKS_URL"
    };

    let oidc_client_id = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_CLIENT_ID",
        true => "RISKY_TREES_GITHUB_CLIENT_ID"
    };

    let oidc_client_secret = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_CLIENT_SECRET",
        true => "RISKY_TREES_GITHUB_CLIENT_SECRET"
    };

    let oidc_issuer_url = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_ISSUER_URL",
        true => "RISKY_TREES_GITHUB_ISSUER_URL"
    };

    let oidc_token_url = match validation_result.provider == "github" {
        false => "RISKY_TREES_GOOGLE_TOKEN_URL",
        true => "RISKY_TREES_GITHUB_TOKEN_URL"
    };

    let redirect_url = RedirectUrl::new(env::var(oidc_redirect_url).expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting redirect URL".to_owned()
    })?;

    let jwks_url = openidconnect::JsonWebKeySetUrl::new(env::var(oidc_jwks_url).expect("to exist").to_string()).expect("Should work");
    let http_client = openidconnect::reqwest::async_http_client;
    let jwks = JsonWebKeySet::fetch_async(&jwks_url, http_client).await.expect("Should resolve JWKS");

    let client =
    openidconnect::core::CoreClient::new(
        ClientId::new(env::var(oidc_client_id).expect("to exist").to_string()),
        Some(ClientSecret::new(env::var(oidc_client_secret).expect("to exist").to_string())),
        IssuerUrl::new(env::var(oidc_issuer_url).expect("to exist").to_string()).expect("Should be able to create Issuer URL"),
        AuthUrl::new(env::var(oidc_auth_url).expect("to exist").to_string()).expect("Should be able to create auth URL"),
        Some(TokenUrl::new(env::var(oidc_token_url).expect("to exist").to_string()).expect("Should be able to create token URL")), 
        None, jwks

    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(redirect_url);

    let token_result =
    client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(http_client).await;

    match token_result {
        Ok(token_result) => {
            let id_token = token_result.id_token();
            match id_token {
                Some(id_token) => {
                    // Extract the ID token claims after verifying its authenticity and nonce.
                    let claims = id_token.claims(&client.id_token_verifier(), &validation_result.nonce);

                    match claims {
                        Ok(claims) => {
                            match claims.email() {
                                Some(email) => Ok(email.to_string()),
                                None => Err(AuthError { message: "Email unavailable".to_owned() })
                            }
                        },
                        Err(err) => {
                            eprintln!("{}", err);
                            Err(AuthError { message: "Claims extraction failed".to_owned() })
                        }
                    }
                },
                None => {
                    match validation_result.provider.as_str() {
                        "google" => Err(AuthError { message: "Did not receive an ID Token".to_owned() }),
                        _ => {
                            // Some login providers don't support OIDC's Identity Token so we have to do a manual identity lookup
                            let access_token = token_result.access_token();
                            match exchange_github_access_token_for_email(access_token).await {
                                Ok(email) => {
                                    Ok(email.to_string())
                                },
                                Err(err) => Err(err)
                            }
                        }
                    }
                }
            }
        },
        Err(err) => {
            Err(AuthError { message: "Trade token failed".to_owned() })
        }
    }

}

pub fn generate_user_jwt(email: &String, expiration: SecondsSinceEpoch, identifier: Option<String> ) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(env::var("RISKY_TREES_JWT_SECRET").expect("to exist").as_bytes()).expect("Should be able to create key");
    let mut claims = std::collections::BTreeMap::new();
    claims.insert("email", email.clone());
    claims.insert("expiration", expiration.to_string());

    if identifier.is_some() {
        claims.insert("identifier", identifier.expect("Checked"));
    }

    let token_str = claims.sign_with_key(&key).expect("Sign should work");

    Ok(token_str)
}

pub fn verify_user_jwt(token: &String) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(env::var("RISKY_TREES_JWT_SECRET").expect("to exist").as_bytes()).expect("Should be able to create key");
    let claims: Result<std::collections::BTreeMap<String, String>, jwt::error::Error>  = token.verify_with_key(&key);

    match claims {
        Ok(claims) => {
            if (!claims.contains_key("expiration")) {
                Err(AuthError {
                    message: "JWT verification failed (missing keys)!".to_owned()
                })
            } else {
                let start = std::time::SystemTime::now();
                let since_the_epoch = start
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards").as_secs();
                let valid_until: u64 = claims["expiration"].parse().expect("Should be a u64");
                if since_the_epoch < valid_until {
                    Ok(claims["email"].clone())
                } else {
                    Err(AuthError {
                        message: "JWT verification failed (expired)!".to_owned()
                    })
                }
            }
        },
        Err(err) => {
            Err(AuthError {
                message: "JWT verification failed!".to_owned()
            })
        }
    }

}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for ApiKey {
    type Error = AuthError;

    async fn from_request(request: &'r rocket::request::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => rocket::request::Outcome::Error((rocket::http::Status::BadRequest, AuthError {
                message: "No Auth header".to_owned()
            })),
            1 => {
                // Validate token
                let mut token = keys[0].to_string();

                if token.contains("Bearer ") {
                    token = token.replace("Bearer ", "");
                } 

                match verify_user_jwt(&token.to_string()) {
                    Ok(email) => {
                        let user_self_tenant = Tenant {name: email.to_string()};

                        // Create user in DB if they don't exist
                        let db_client = database::get_instance().await.expect("Should always work");

                        let user_exists = database::get_user(&db_client, user_self_tenant.clone(), email.clone()).await;
                        match user_exists {
                            Some(user) => {},
                            None => {
                                database::new_user(&db_client, email.clone()).await;
                                ()
                            }
                        }

                        let org_tenants = database::get_tenants_for_user(&db_client, &email).await;
                        let mut all_tenants = org_tenants.clone();
                        all_tenants.push(user_self_tenant);
                        println!("{:#?}", all_tenants);

                        rocket::request::Outcome::Success(ApiKey {
                            email: email.to_string(),
                            tenants: all_tenants
                        })
                    },
                    Err(err) => {
                        rocket::request::Outcome::Error((rocket::http::Status::BadRequest, AuthError {
                            message: "JWT Verification failed".to_owned()
                        }))
                    }
                }
            },
            _ => rocket::request::Outcome::Error((rocket::http::Status::BadRequest, AuthError {
                message: "Too many auth headers!".to_owned()
            })),
        }
    }
}