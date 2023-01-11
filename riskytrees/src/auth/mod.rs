use std::env;
use hmac::{Hmac, Mac};
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
use openidconnect::reqwest::http_client;

use crate::errors::{AuthError, self};

pub struct AuthRequestData {
   pub url: openidconnect::url::Url,
   pub csrf_token: CsrfToken,
   pub nonce: Nonce
}

pub struct ApiKey(String);


pub fn start_flow() -> Result<AuthRequestData, AuthError> {
    let auth_url = AuthUrl::new(env::var("RISKY_TREES_GOOGLE_AUTH_URL").expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting auth URL".to_owned()
    })?;
    let redirect_url = RedirectUrl::new(env::var("RISKY_TREES_GOOGLE_REDIRECT_URL").expect("to exist").to_string());


    match redirect_url {
        Ok(redirect_url) => {
            let client =
            openidconnect::core::CoreClient::new(
                ClientId::new(env::var("RISKY_TREES_GOOGLE_CLIENT_ID").expect("to exist").to_string()),
                Some(ClientSecret::new(env::var("RISKY_TREES_GOOGLE_CLIENT_SECRET").expect("to exist").to_string())),
                IssuerUrl::new(env::var("RISKY_TREES_GOOGLE_ISSUER_URL").expect("to exist").to_string()).expect("Should be able to create Issuer URL"),
                AuthUrl::new(env::var("RISKY_TREES_GOOGLE_AUTH_URL").expect("to exist").to_string()).expect("Should be able to create auth URL"),
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
                .add_scope(Scope::new("read".to_string()))
                .add_scope(Scope::new("write".to_string()))
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

// Returns email if trade succeeds
pub fn trade_token(code: &String, nonce: Nonce) -> Result<String, AuthError> {
    let auth_url = AuthUrl::new(env::var("RISKY_TREES_GOOGLE_AUTH_URL").expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting auth URL".to_owned()
    })?;
    let redirect_url = RedirectUrl::new(env::var("RISKY_TREES_GOOGLE_REDIRECT_URL").expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting redirect URL".to_owned()
    })?;
    let token_url = AuthUrl::new(env::var("RISKY_TREES_GOOGLE_TOKEN_URL").expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting auth URL".to_owned()
    })?;

    let jwks_url = openidconnect::JsonWebKeySetUrl::new(env::var("RISKY_TREES_GOOGLE_JWKS_URL").expect("to exist").to_string()).expect("Should work");
    let http_client = openidconnect::reqwest::http_client;
    let jwks = JsonWebKeySet::fetch(&jwks_url, http_client).expect("Should resolve JWKS");

    let client =
    openidconnect::core::CoreClient::new(
        ClientId::new(env::var("RISKY_TREES_GOOGLE_CLIENT_ID").expect("to exist").to_string()),
        Some(ClientSecret::new(env::var("RISKY_TREES_GOOGLE_CLIENT_SECRET").expect("to exist").to_string())),
        IssuerUrl::new(env::var("RISKY_TREES_GOOGLE_ISSUER_URL").expect("to exist").to_string()).expect("Should be able to create Issuer URL"),
        AuthUrl::new(env::var("RISKY_TREES_GOOGLE_AUTH_URL").expect("to exist").to_string()).expect("Should be able to create auth URL"),
        Some(TokenUrl::new(env::var("RISKY_TREES_GOOGLE_TOKEN_URL").expect("to exist").to_string()).expect("Should be able to create token URL")), 
        None, jwks

    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(redirect_url);

    let token_result =
    client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request(http_client);

    match token_result {
        Ok(token_result) => {
            let id_token = token_result.id_token();
            match id_token {
                Some(id_token) => {
                    // Extract the ID token claims after verifying its authenticity and nonce.
                    let claims = id_token.claims(&client.id_token_verifier(), &nonce);

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
                None =>  Err(AuthError { message: "Did not receive an ID Token".to_owned() })
            }
        },
        Err(err) => {
            Err(AuthError { message: "Trade token failed".to_owned() })
        }
    }

}

pub fn generate_user_jwt(email: &String) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(env::var("RISKY_TREES_JWT_SECRET").expect("to exist").as_bytes()).expect("Should be able to create key");
    let mut claims = std::collections::BTreeMap::new();
    claims.insert("email", email.clone());
    let token_str = claims.sign_with_key(&key).expect("Sign should work");

    Ok(token_str)
}

pub fn verify_user_jwt(token: &String) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(env::var("RISKY_TREES_JWT_SECRET").expect("to exist").as_bytes()).expect("Should be able to create key");
    let claims: Result<std::collections::BTreeMap<String, String>, jwt::error::Error>  = token.verify_with_key(&key);

    match claims {
        Ok(claims) => {
            Ok(claims["email"].clone())
        },
        Err(err) => {
            Err(AuthError {
                message: "JWT verification failed!".to_owned()
            })
        }
    }

}


impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for ApiKey {
    type Error = AuthError;

    fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => rocket::Outcome::Failure((rocket::http::Status::BadRequest, AuthError {
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
                        rocket::Outcome::Success(ApiKey(token.to_string()))
                    },
                    Err(err) => {
                        rocket::Outcome::Failure((rocket::http::Status::BadRequest, AuthError {
                            message: "JWT Verification failed".to_owned()
                        }))
                    }
                }
            },
            _ => rocket::Outcome::Failure((rocket::http::Status::BadRequest, AuthError {
                message: "Too many auth headers!".to_owned()
            })),
        }
    }
}