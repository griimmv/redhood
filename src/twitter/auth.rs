use hmac::{Hmac, Mac};
use rand::Rng;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

#[derive(Clone)]
pub struct TwitterAuth {
    api_key: String,
    api_secret_key: String,
    access_token: String,
    access_token_secret: String,
    client: reqwest::Client,
}

impl TwitterAuth {
    pub fn new(
        api_key: &str,
        api_secret_key: &str,
        access_token: &str,
        access_token_secret: &str,
    ) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_secret_key: api_secret_key.to_string(),
            access_token: access_token.to_string(),
            access_token_secret: access_token_secret.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn sign_request(
        &self,
        method: reqwest::Method,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::RequestBuilder, reqwest::Error> {
        let builder = self.client.request(method.clone(), url);

        let oauth_nonce = generate_nonce();
        let oauth_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let mut all_params: Vec<(String, String)> = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        all_params.push(("oauth_consumer_key".into(), self.api_key.clone()));
        all_params.push(("oauth_nonce".into(), oauth_nonce.clone()));
        all_params.push(("oauth_signature_method".into(), "HMAC-SHA1".into()));
        all_params.push(("oauth_timestamp".into(), oauth_timestamp.clone()));
        all_params.push(("oauth_token".into(), self.access_token.clone()));
        all_params.push(("oauth_version".into(), "1.0".into()));

        all_params.sort_by(|a, b| a.0.cmp(&b.0));

        let param_string: String = all_params
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let signature_base = format!(
            "{}&{}&{}",
            method.as_str(),
            encode(url),
            encode(&param_string),
        );

        let signing_key = format!(
            "{}&{}",
            encode(&self.api_secret_key),
            encode(&self.access_token_secret)
        );

        let mut mac =
            HmacSha1::new_from_slice(signing_key.as_bytes()).expect("HMAC key should be valid");
        mac.update(signature_base.as_bytes());
        let signature = mac.finalize().into_bytes();
        let oauth_signature = base64_encode(&signature);

        let auth_header = format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", \
             oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", \
             oauth_token=\"{}\", oauth_version=\"1.0\"",
            encode(&self.api_key),
            encode(&oauth_nonce),
            encode(&oauth_signature),
            encode(&oauth_timestamp),
            encode(&self.access_token),
        );

        Ok(builder.header("Authorization", auth_header).query(params))
    }
}

fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.r#gen()).collect();
    base64_encode(&bytes)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(32)
        .collect()
}

fn base64_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

fn encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_alpha_numeric() {
        assert_eq!(encode("hello123"), "hello123");
    }

    #[test]
    fn encode_spaces_and_symbols() {
        assert_eq!(encode("a b&c"), "a%20b%26c");
    }

    #[test]
    fn base64_encode_known() {
        assert_eq!(base64_encode(b"test"), "dGVzdA==");
    }
}
