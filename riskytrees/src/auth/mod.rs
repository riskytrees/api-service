use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;

use crate::errors::{AuthError, self};

pub struct AuthRequestData {
   pub url: oauth2::url::Url,
   pub csrf_token: CsrfToken
};

pub fn start_flow() -> Result<AuthRequestData, AuthError> {
    let auth_url = AuthUrl::new("http://authorize".to_string());
    let redirect_url = RedirectUrl::new("http://redirect".to_string());

    match auth_url {
        Ok(auth_url) => {
            match redirect_url {
                Ok(redirect_url) => {
                    let client =
                    BasicClient::new(
                        ClientId::new("client_id".to_string()),
                        Some(ClientSecret::new("client_secret".to_string())),
                        auth_url,
                        None
                    )
                    // Set the URL the user will be redirected to after the authorization process.
                    .set_redirect_uri(redirect_url);

                    // Generate a PKCE challenge.
                    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

                    // Generate the full authorization URL.
                    let (auth_url, csrf_token) = client
                        .authorize_url(CsrfToken::new_random)
                        // Set the desired scopes.
                        .add_scope(Scope::new("read".to_string()))
                        .add_scope(Scope::new("write".to_string()))
                        // Set the PKCE code challenge.
                        .set_pkce_challenge(pkce_challenge)
                        .url();


                    Ok(AuthRequestData {
                        url: auth_url,
                        csrf_token: csrf_token
                    })
                },
                Err(err) => {
                    Err(errors::AuthError {
                        message: "No redirect URL".to_owned()
                    })
                }
            }


        },
        Err(err) => {
            Err(errors::AuthError {
                message: "No auth URL".to_owned()
            })
        }
    }

}