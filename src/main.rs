use std::sync::Arc;
use std::sync::Mutex;

use futures::StreamExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsePayload {
    #[serde(rename = "Status")]
    pub status: i64,
    #[serde(rename = "Msg")]
    pub msg: Value,
    #[serde(rename = "Item")]
    pub item: Value,
    #[serde(rename = "List")]
    pub list: Vec<ResponsePayloadItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsePayloadItem {
    pub id: i64,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "TuijianText")]
    pub tuijian_text: String,
    #[serde(rename = "Uptime")]
    pub uptime: String,
    pub keywords: Value,
    #[serde(rename = "ReportList")]
    pub report_list: String,
    #[serde(rename = "GraphList")]
    pub graph_list: String,
    #[serde(rename = "PagesCount")]
    pub pages_count: i64,
    #[serde(rename = "tID")]
    pub t_id: i64,
    pub industry: String,
    #[serde(rename = "Topic")]
    pub topic: String,
    pub is_free: i64,
    #[serde(rename = "Content")]
    pub content: String,
}

#[derive(Debug)]
pub struct RearchReport {
    pub id: i64,
    pub title: String,
    pub report_time: String,
    pub introduction: String,
    pub pages_count: i64,
    pub industry: String,
    pub cover: String,
    pub download_url: String,
}

async fn fetch_research_report_by_id(
    client: reqwest::Client,
    report_id: i64,
) -> Option<RearchReport> {
    let url = format!(
        "https://www.iresearch.com.cn/api/Detail/reportM?id={}&isfree=0",
        report_id
    );
    // println!("url {}", url);

    let response_payload = client
        .get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .unwrap()
        .json::<ResponsePayload>()
        .await
        .unwrap();
    // println!("{:#?}", response_payload);

    if response_payload.list.is_empty() {
        return None;
    }

    let response_payload_item = &response_payload.list[0];
    // println!("{:#?}", response_payload_item);

    let download_url = format!(
        "https://www.iresearch.cn/include/ajax/user_ajax.ashx?work=idown&rid={}",
        response_payload_item.id
    );
    // println!("download_url {:#?}", download_url);

    let research_report = RearchReport {
        id: response_payload_item.id,
        title: response_payload_item.title.to_string(),
        report_time: response_payload_item.uptime.to_string(),
        introduction: response_payload_item.tuijian_text.to_string(),
        pages_count: response_payload_item.pages_count,
        industry: response_payload_item.industry.to_string(),
        cover: response_payload_item.topic.to_string(),
        download_url,
    };
    // println!("research_report {:#?}", research_report);

    Some(research_report)
}

async fn fech_research_report_list_by_id_list(
    client: reqwest::Client,
    report_id_list: Vec<i64>,
) -> Result<Arc<Mutex<Vec<RearchReport>>>, String> {
    let result = futures::stream::iter(report_id_list)
        .map(|report_id| {
            let client = client.clone();
            tokio::spawn(async move { fetch_research_report_by_id(client, report_id).await })
        })
        .buffer_unordered(64);

    let research_report_list_arc = Arc::new(Mutex::new(vec![]));
    result
        .for_each(|r| {
            let research_report_list_arc_clone = research_report_list_arc.clone();
            async move {
                match r {
                    Ok(o) => {
                        if let Some(research_report) = o {
                            println!("{:#?}", research_report);
                            let mut task = research_report_list_arc_clone.lock().unwrap();
                            task.push(research_report);
                        }
                    }
                    Err(_) => todo!(),
                }
            }
        })
        .await;

    Ok(research_report_list_arc.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report_id_range_begin = 0;
    let report_id_range_end = 4000;
    let report_id_list = (report_id_range_begin..report_id_range_end).collect::<Vec<i64>>();

    let research_report_list_arc =
        fech_research_report_list_by_id_list(reqwest::Client::new(), report_id_list).await?;

    let research_report_list = research_report_list_arc.lock().unwrap();
    println!(
        "{:#?} len {}",
        research_report_list,
        research_report_list.len()
    );

    Ok(())
}
