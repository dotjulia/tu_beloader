use regex::Regex;
use reqwest::blocking::Client;

#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    pub id: String,
    pub resolution: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct MediaEntry {
    pub id: String,
    pub mimetype: String,
    pub url: String,
    pub video: Option<Video>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub track: Vec<MediaEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaPackage {
    pub duration: u32,
    pub id: String,
    pub media: Media,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub id: String,
    #[serde(rename = "dcTitle")]
    pub title: String,
    #[serde(rename = "dcCreator")]
    pub creator: Option<String>,
    #[serde(rename = "dcCreated")]
    pub created: String,
    pub mediapackage: MediaPackage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub offset: u32,
    pub limit: u32,
    pub total: u32,
    #[serde(rename = "searchTime")]
    pub search_time: u32,
    #[serde(rename = "result")]
    pub results: Vec<QueryResult>,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct SeriesRequest {
    #[serde(rename = "search-results")]
    pub search_results: SearchResults,
}

pub fn login(client: &Client, username: &str, password: &str, otp_token: &str) -> Result<(), String> {
    let resp = client.get("https://tube.tugraz.at/Shibboleth.sso/Login?target=/paella/ui/index.html").send().map_err(|e| e.to_string())?;
    let res_text = resp.text().map_err(|e| e.to_string())?;
    let re = Regex::new(r#"/idp/profile/SAML2/Redirect/SSO;jsessionid=.*?execution=e1s1"#).unwrap();
    let login_url = match re.captures_iter(&res_text).next() {
        Some(e) => e.get(0).unwrap().as_str(),
        None => return Err("No login url in response".to_owned()),
    };
    let params = [
        ("j_username", username),
        ("j_password", password),
        ("lang", "de"),
        ("_eventId_proceed", ""),
    ];
    let login_response = client.post("https://sso.tugraz.at".to_owned() + login_url).form(&params).send().map_err(|e| e.to_string())?;
    let login_response_text = login_response.text().map_err(|e| e.to_string())?;
    if login_response_text.contains("Welcome to TU Graz TUbe") {
        Ok(())
    } else {
        println!("Trying OTP value");
        let params_otp = [
            ("lang", "de"),
            ("j_tokenNumber", otp_token),
            ("_eventId_proceed", ""),
        ];
        let otp_url = &login_url.to_string().replace("e1s1", "e1s2");
        let otp_response = client.post("https://sso.tugraz.at".to_owned() + otp_url).form(&params_otp).send().map_err(|e| e.to_string())?;
        let otp_response_text = otp_response.text().map_err(|e| e.to_string())?;
        if otp_response_text.contains("Welcome to TU Graz TUbe") {
            Ok(())
        } else {
            println!("{}", otp_response_text);
            Err("Login failed".to_owned())
        }
    }
}

pub fn get_series(client: &Client, series_id: &str) -> Result<SeriesRequest, String>{
    let res = client.get("https://tube.tugraz.at/search/episode.json?sid=".to_owned() + series_id).send().map_err(|e| e.to_string())?;
    let res = res.text().map_err(|e| e.to_string())?;
    let series_request = serde_json::from_str(&res).map_err(|e| e.to_string())?;
    Ok(series_request)
}

#[derive(Serialize,Deserialize, Debug)]
pub struct Value {
    pub value: String,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct SearchEntry {
    pub identifier: Vec<Value>,
    pub creator: Option<Vec<Value>>,
    pub title: Vec<Value>,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct PurlEntry {
    #[serde(rename = "http://purl.org/dc/terms/")]
    pub body: SearchEntry,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct SearchRequest {
    pub catalogs: Vec<PurlEntry>,
}

pub fn search_series(client: &Client, search_term: &str) -> Result<SearchRequest, String> {
    let res = client.get("https://tube.tugraz.at/series/series.json?sort=TITLE&count=20&q=".to_owned() + search_term).send().map_err(|e| e.to_string())?;
    let res = res.text().map_err(|e| e.to_string())?;
    serde_json::from_str(&res).map_err(|e| e.to_string())
}
