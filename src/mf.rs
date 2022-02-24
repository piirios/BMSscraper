use std::path::Path;

use crate::mf_config::MFConfig;
use crate::string_utils::StringUtils;
use chrono::prelude::*;
use log::{error, info};
use reqwest;
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub trait ResultLog<T, E> {
    fn log_result<O, F>(self, fun_ok: O, fun_err: F) -> Self
    where
        O: Fn(&T) -> (),
        F: Fn(&E) -> ();
}
impl<T, E> ResultLog<T, E> for Result<T, E> {
    fn log_result<O, F>(self, fun_ok: O, fun_err: F) -> Self
    where
        O: Fn(&T) -> (),
        F: Fn(&E) -> (),
    {
        match &self {
            Ok(t) => fun_ok(t),
            Err(e) => fun_err(e),
        };
        self
    }
}

fn transform_token(token: String) -> String {
    token
        .chars()
        .into_iter()
        .map(|el| {
            if el.is_ascii_alphabetic() {
                let t = if el.is_uppercase() { 65 } else { 97 };
                char::from_u32(t + (el as u32 - t + 13) % 26).unwrap()
            } else {
                el
            }
        })
        .collect::<String>()
}
pub struct MFparser {
    pub config: MFConfig,
}

impl MFparser {
    pub fn new(config: MFConfig) -> Self {
        Self { config }
    }
    //return Result<Option<BMS>, Option<BMR>>
    pub async fn make_request(&self) -> anyhow::Result<(String, Option<String>)> {
        let bmr_url = format!("https://rpcache-aa.meteofrance.com/internet2018client/2.0/report?domain=BMRCOTE-0{}-0{}&report_type=marine&report_subtype=BMR_cote_fr&format=xml", self.config.region, self.config.zone);
        let bms_url = format!("https://rpcache-aa.meteofrance.com/internet2018client/2.0/report?domain=BMSCOTE-0{}-0{}&report_type=marine&report_subtype=BMS_cote_fr&format=json", self.config.region, self.config.zone);

        //get the token
        let c = String::from_utf8(
            reqwest::get(
                "https://meteofrance.com/meteo-marine/penmarc-h-anse-de-l-aiguillon/BMSCOTE-01-04",
            )
            .await?
            .headers()
            .get("set-cookie")
            .expect("we need a cookie")
            .as_bytes()
            .to_vec(),
        )
        .log_result(
            |_| info!("successfully get the token"),
            |e| error!("failed to get token with error: {e}"),
        )
        .unwrap();
        let cookie = c.after("mfsession=");

        let token = transform_token(
            cookie
                .before(";")
                .expect("can't cut the cookie for get the token")
                .to_owned(),
        );

        let client = reqwest::Client::new();
        //get the BMS
        let bms = client
            .get(bms_url)
            .bearer_auth(&token)
            .send()
            .await
            .log_result(
                |_| info!("successfully get the BMS file"),
                |e| error!("failed to get BMS file with error: {e}"),
            )?
            .text()
            .await?;
        let mut bmr = None;
        if self.config.want_bmr {
            bmr = Some(
                client
                    .get(bmr_url)
                    .bearer_auth(token)
                    .send()
                    .await
                    .log_result(
                        |_| info!("successfully get the BMR file"),
                        |e| error!("failed to get the BMR file with error: {e}"),
                    )?
                    .text()
                    .await?,
            );
        }
        Ok((bms, bmr))
    }

    pub async fn write_raw(
        &self,
        res: String,
        typereport: &str,
        path: &str,
        ext: &str,
    ) -> anyhow::Result<()> {
        let utc = Utc::now();
        let fname = format!(
            "{}_zone{}_{}.{}",
            typereport.to_uppercase(),
            self.config.zone,
            utc.format("%Y_%m_%d_%H_%M").to_string(),
            ext
        );
        let path = Path::new(path).join(fname);

        let mut file = File::create(path).await?;
        file.write_all(res.as_bytes()).await?;
        Ok(())
    }

    pub async fn write_pretty_xml(
        &self,
        res: String,
        typereport: &str,
        path: &str,
    ) -> anyhow::Result<()> {
        let content = res
            .split("<echeance")
            .map(|section| {
                section
                    .split("![CDATA[")
                    .skip(1)
                    .map(|seg| {
                        seg.clone()
                            .before("]]")
                            .expect("we can't parse the response")
                            .to_owned()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let utc = Utc::now();
        let fname = format!(
            "{}_pretty_zone{}_{}.txt",
            typereport.to_uppercase(),
            self.config.zone,
            utc.format("%Y_%m_%d_%H").to_string()
        );
        let path = Path::new(path).join(fname);

        let mut file = File::create(path).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }
    pub async fn write_pretty_json(
        &self,
        res: String,
        typereport: &str,
        path: &str,
    ) -> anyhow::Result<()> {
        let mut content = String::new();
        let v: Value = serde_json::from_str(&res)?;
        content.push_str(&format!("{}\n\n", v["report_title"].as_str().unwrap()));
        content.push_str(&format!(
            "{}\n\n\n",
            v["text_bloc_item"][0]["text_items"]
                .as_array()
                .unwrap()
                .into_iter()
                .map(|el| { el["title"].as_str().unwrap() })
                .collect::<Vec<_>>()
                .join("\n")
        ));
        content.push_str(&format!(
            "{}\n\n",
            v["text_bloc_item"][1]["bloc_title"]
                .as_str()
                .unwrap()
                .to_uppercase()
        ));
        content.push_str(&format!(
            "{}\n\n",
            v["text_bloc_item"][1]["text_items"]
                .as_array()
                .unwrap()
                .into_iter()
                .map(|el| { el["text"].as_str().unwrap() })
                .collect::<Vec<_>>()
                .join("\n")
        ));

        let utc = Utc::now();
        let fname = format!(
            "{}_pretty_zone{}_{}.txt",
            typereport.to_uppercase(),
            self.config.zone,
            utc.format("%Y_%m_%d_%H").to_string()
        );
        let path = Path::new(path).join(fname);

        let mut file = File::create(path).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let (bms, bmr) = self.make_request().await?;
        if let Some(bmreport) = bmr {
            if self.config.pretty {
                self.write_pretty_xml(bmreport, "BMR", &self.config.bmrpath)
                    .await
                    .log_result(
                        |_| info!("successfully write BMS"),
                        |e| error!("failed to write BMS with error: {e}"),
                    )?
            } else {
                self.write_raw(bmreport, "BMR", &self.config.bmrpath, "xml")
                    .await
                    .log_result(
                        |_| info!("successfully write BMR"),
                        |e| error!("failed to write BMR with error: {e}"),
                    )?
            }
        }
        if !bms.is_empty() {
            if self.config.pretty {
                self.write_pretty_json(bms, "BMS", &self.config.bmspath)
                    .await
                    .log_result(
                        |_| info!("successfully write BMR"),
                        |e| error!("failed to write BMR with error: {e}"),
                    )?
            } else {
                self.write_raw(bms, "BMS", &self.config.bmspath, "json")
                    .await
                    .log_result(
                        |_| info!("successfully write BMS"),
                        |e| error!("failed to write BMS with error: {e}"),
                    )?
            }
        }
        Ok(())
    }
}
