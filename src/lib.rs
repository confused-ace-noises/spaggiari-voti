use std::{collections::HashMap, time::Duration};

use bees::{
    core::{
        client,
        resource::Resource,
        resources_utils::{
            static_res::StaticResource,
            updating_token::{Token, UpdatingToken},
        },
    },
    endpoint,
    endpoint_record::endpoint::{Body, Capability, FormatString, HttpVerb},
    net::client::RequestBuilder,
    record, resource,
};
use reqwest::{Response, header::HeaderMap};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub struct AddHeaders(pub HeaderMap);

impl AddHeaders {
    pub fn new(headers: HeaderMap) -> Self {
        Self(headers)
    }
}

impl Capability for AddHeaders {
    fn apply(&self, request: RequestBuilder) -> RequestBuilder {
        request.headers(self.0.clone())
    }
}


pub struct Api(pub Credentials);

impl Api {
    fn auth_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.append("User-Agent", "CVVS/std/4.1.7 Android/10".parse().unwrap());
        headers.append("Z-Dev-ApiKey", "Tg1NWEwNGIgIC0K".parse().unwrap());
        headers.append("ContentsDiary-Type", "application/json".parse().unwrap());
        headers.append("Content-Type", "application/json".parse().unwrap());

        headers
    }

    async fn regular_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.append("User-Agent", "CVVS/std/4.1.7 Android/10".parse().unwrap());
        headers.append("Z-Dev-ApiKey", "Tg1NWEwNGIgIC0K".parse().unwrap());
        headers.append("ContentsDiary-Type", "application/json".parse().unwrap());
        headers.append(
            "Z-Auth-Token",
            resource!("token").data().await.to_string().parse().unwrap(),
        );

        headers
    }

    pub async fn register(&self) {
        let creds = &self.0;
        resource!(reg StaticResource { name: "non_picky_uid".to_owned(), data: creds.uid.clone() });
        resource!(reg StaticResource { name: "password".to_owned(), data: creds.password.clone() });

        record!(
            "auth" => "https://web.spaggiari.eu/rest/v1/auth/";
            [AddHeaders(Self::auth_headers())],
        );

        endpoint!("auth" =>
            new "login",
            "/login",
            HttpVerb::POST(
                Body::Text(
                    FormatString::new(
                        r#"{"ident": null, "pass": "<?password>", "uid": "<?non_picky_uid>"}"#
                    )
                )
            ),
            login
        );

        let resp: Value = serde_json::from_str(&client().run_endpoint(endpoint!("auth" => "login"), &HashMap::new(), &vec![]).run_resp().await.unwrap().text().await.unwrap()).unwrap();

        let picky_uid = &resp["ident"].as_str().unwrap()[1..];
        println!("{}", picky_uid);
        resource!(reg StaticResource { name: "uid".to_owned(), data: picky_uid.to_string() });

        let updating_token = UpdatingToken::new(
            "auth",
            "login",
            "token".to_string(),
            Duration::from_hours(1),
            HashMap::new(),
            vec![],
        )
        .await
        .unwrap();

        resource!(reg updating_token);

        record!(
            "students" => "https://web.spaggiari.eu/rest/v1/students/";
            [AddHeaders(Self::regular_headers().await)]
        );

        endpoint!("students" =>
            new "grades",
            "/<?uid>/grades",
            HttpVerb::GET,
            grades,
        );
    }

    pub async fn grades(&self) -> Vec<Grade> {
        client()
            .run_endpoint(endpoint!("students" => "grades"), &HashMap::new(), &vec![])
            .run::<Vec<Grade>>()
            .await
            .unwrap()
    }
}

pub async fn login(resp: Response) -> Token {
    println!("{:#?}", resp);

    let text = resp.text().await.unwrap();

    println!("{}", text);
    let json: Value = serde_json::from_str(&text).unwrap();
    Token(json["token"].as_str().unwrap().to_string())
}

pub async fn grades(resp: Response) -> Vec<Grade> {

    let mut json: Value = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
    println!("{}", json);

    let grades = json["grades"]
        .take()
        .as_array()
        .unwrap()
        .clone()
        .into_iter()
        .map(|v| Grade::deserialize(v).unwrap())
        .collect::<Vec<_>>();
    grades
}

#[derive(Debug, Deserialize)]
pub struct Grade {
    pub canceled: bool,
    pub color: String,

    #[serde(rename = "componentDesc")]
    pub component_desc: String,

    #[serde(rename = "componentPos")]
    pub component_pos: i64,

    #[serde(rename = "decimalValue")]
    pub decimal_value: Option<f32>,

    #[serde(rename = "displaPos")]
    pub display_pos: i64,

    #[serde(rename = "displayValue")]
    pub display_value: String,

    #[serde(rename = "evtCode")]
    pub evt_code: String,

    #[serde(rename = "evtDate")]
    pub evt_date: String,

    #[serde(rename = "evtId")]
    pub evt_id: i64,

    #[serde(rename = "gradeMasterId")]
    pub grade_master_id: i64,

    #[serde(rename = "noAverage")]
    pub no_average: bool,

    #[serde(rename = "notesForFamily")]
    pub notes_for_family: String,

    #[serde(rename = "oldskillDesc")]
    pub old_skill_desc: String,

    #[serde(rename = "oldskillId")]
    pub old_skill_id: i64,

    #[serde(rename = "periodDesc")]
    pub period_desc: String,

    #[serde(rename = "periodLabel")]
    pub period_label: String,

    #[serde(rename = "periodPos")]
    pub period_pos: i64,

    #[serde(rename = "skillCode")]
    pub skill_code: Option<String>,

    #[serde(rename = "skillDesc")]
    pub skill_desc: Option<String>,

    #[serde(rename = "skillId")]
    pub skill_id: i64,

    #[serde(rename = "skillMasterId")]
    pub skill_master_id: i64,

    #[serde(rename = "skillValueDesc")]
    pub skill_value_desc: String,

    #[serde(rename = "skillValueNote")]
    pub skill_value_note: String,

    #[serde(rename = "skillValueShortDesc")]
    pub skill_value_short_desc: Option<String>,

    #[serde(rename = "subjectCode")]
    pub subject_code: Option<String>,

    #[serde(rename = "subjectDesc")]
    pub subject_desc: String,

    #[serde(rename = "subjectId")]
    pub subject_id: i64,

    #[serde(rename = "teacherName")]
    pub teacher_name: String,

    pub underlined: bool,

    #[serde(rename = "weightFactor")]
    pub weight_factor: i64,
}

// pub struct Grades(HashMap<String, Vec<u8>>);

#[derive(Debug, Clone)]
pub struct Credentials {
    pub uid: String,
    pub password: String,
}