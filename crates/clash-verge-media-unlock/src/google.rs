use reqwest::Client;

use super::UnlockItem;

pub(crate) const GOOGLE_NAME: &str = "Google";

// ccTLDs served by Google that do not map to an ISO 3166-1 alpha-2 code via
// simple suffix matching.
const CCTLD_OVERRIDES: [(&str, &str); 6] = [
    ("co.uk", "GB"),
    ("co.jp", "JP"),
    ("co.kr", "KR"),
    ("co.in", "IN"),
    ("co.id", "ID"),
    ("co.za", "ZA"),
];

fn region_code_from_host(host: &str) -> Option<String> {
    let host = host.trim_end_matches('.');

    for (cctld, code) in CCTLD_OVERRIDES {
        if host.ends_with(cctld) {
            return Some(code.to_string());
        }
    }

    host.rsplit('.').next().and_then(|tld| {
        if tld.len() == 2 && tld.bytes().all(|c| c.is_ascii_alphabetic()) {
            Some(tld.to_ascii_uppercase())
        } else {
            None
        }
    })
}

pub(super) async fn check_google(client: &Client) -> UnlockItem {
    let Ok(response) = client.get("https://www.google.com/").send().await else {
        return UnlockItem::checked(GOOGLE_NAME, "Failed", None);
    };

    let Some(url) = response.url().host_str() else {
        return UnlockItem::checked(GOOGLE_NAME, "Failed", None);
    };

    // google.com.hk is served to mainland China exits as well as Hong Kong,
    // so the ccTLD alone is not a reliable region indicator there.
    if url == "www.google.com.hk" {
        return UnlockItem::checked(GOOGLE_NAME, "Yes", Some("🇨🇳CN/🇭🇰HK".to_string()));
    }

    let Some(code) = region_code_from_host(url) else {
        return UnlockItem::checked(GOOGLE_NAME, "Failed", None);
    };

    UnlockItem::checked_region(GOOGLE_NAME, "Yes", &code)
}
