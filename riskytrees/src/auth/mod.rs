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
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl, ExtraTokenFields, IdToken, EmptyAdditionalClaims, Nonce
};
use openidconnect::reqwest::http_client;

use crate::errors::{AuthError, self};

pub struct AuthRequestData {
   pub url: openidconnect::url::Url,
   pub csrf_token: CsrfToken,
   pub nonce: Nonce
}

pub fn start_flow() -> Result<AuthRequestData, AuthError> {
    let auth_url = AuthUrl::new(env::var("RISKY_TREES_GOOGLE_AUTH_URL").expect("to exist").to_string()).map_err(|e| AuthError {
        message: "Error getting auth URL".to_owned()
    })?;
    let redirect_url = RedirectUrl::new(env::var("RISKY_TREES_GOOGLE_REDIRECT_URL").expect("to exist").to_string());


    let provider_metadata = openidconnect::core::CoreProviderMetadata::discover(
        &openidconnect::IssuerUrl::new(env::var("RISKY_TREES_GOOGLE_PROVIDER_URL").expect("to exist").to_string())?,
        http_client
    ).map_err(|e| AuthError {
        message: "Error getting provider metadata".to_owned()
    })?;

    match redirect_url {
        Ok(redirect_url) => {
            let client =
            openidconnect::core::CoreClient::from_provider_metadata(
                provider_metadata,
                ClientId::new(env::var("RISKY_TREES_GOOGLE_CLIENT_ID").expect("to exist").to_string()),
                Some(ClientSecret::new(env::var("RISKY_TREES_GOOGLE_CLIENT_SECRET").expect("to exist").to_string()))
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

    let provider_metadata = openidconnect::core::CoreProviderMetadata::discover(
        &openidconnect::IssuerUrl::new(env::var("RISKY_TREES_GOOGLE_PROVIDER_URL").expect("to exist").to_string())?,
        http_client
    ).map_err(|e| AuthError {
        message: "Error getting provider metadata".to_owned()
    })?;

    let client =
    openidconnect::core::CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(env::var("RISKY_TREES_GOOGLE_CLIENT_ID").expect("to exist").to_string()),
        Some(ClientSecret::new(env::var("RISKY_TREES_GOOGLE_CLIENT_SECRET").expect("to exist").to_string()))
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
                        Err(err) => Err(AuthError { message: "Claims extraction failed".to_owned() })
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