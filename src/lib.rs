use std::sync::Arc;
use std::sync::Mutex;

use futures::StreamExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePayload {
    #[serde(rename = "Status")]
    status: i64,
    #[serde(rename = "Msg")]
    msg: Value,
    #[serde(rename = "Item")]
    item: Value,
    #[serde(rename = "List")]
    list: Vec<ResponsePayloadItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePayloadItem {
    id: i64,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "TuijianText")]
    tuijian_text: String,
    #[serde(rename = "Uptime")]
    uptime: String,
    keywords: Value,
    #[serde(rename = "ReportList")]
    report_list: String,
    #[serde(rename = "GraphList")]
    graph_list: String,
    #[serde(rename = "PagesCount")]
    pages_count: i64,
    #[serde(rename = "tID")]
    t_id: i64,
    industry: String,
    #[serde(rename = "Topic")]
    topic: String,
    is_free: i64,
    #[serde(rename = "Content")]
    content: String,
}

#[derive(Debug)]
pub struct RearchReport {
    id: i64,
    title: String,
    report_time: String,
    introduction: String,
    pages_count: i64,
    industry: String,
    cover: String,
    download_url: String,
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

pub async fn fech_research_report_list_by_id_list(
    client: reqwest::Client,
    report_id_list: Vec<i64>,
    parallel_requests: usize,
) -> Result<Arc<Mutex<Vec<RearchReport>>>, String> {
    let result = futures::stream::iter(report_id_list)
        .map(|report_id| {
            let client = client.clone();
            tokio::spawn(fetch_research_report_by_id(client, report_id))
        })
        .buffer_unordered(parallel_requests);

    let research_report_list_arc = Arc::new(Mutex::new(vec![]));
    result
        .for_each(|r| {
            let research_report_list_arc_clone = research_report_list_arc.clone();
            async move {
                match r {
                    Ok(o) => {
                        if let Some(research_report) = o {
                            println!("{:#?}", research_report);
                            let mut research_report_list =
                                research_report_list_arc_clone.lock().unwrap();
                            research_report_list.push(research_report);
                        }
                    }
                    Err(e) => eprintln!("{:#?}", e),
                }
            }
        })
        .await;

    research_report_list_arc
        .lock()
        .unwrap()
        .sort_by(|r1, r2| r1.id.cmp(&r2.id));

    Ok(research_report_list_arc.clone())
}

pub async fn write_to_csv(
    research_report_list_arc: Arc<Mutex<Vec<RearchReport>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("write to csv...");

    let research_report_list = research_report_list_arc.lock().unwrap();
    println!("research_report_list len {:#?}", research_report_list.len());

    let mut csv_writer = csv::Writer::from_path("data.csv")?;

    csv_writer.write_record(&[
        "id",
        "title",
        "report_time",
        "introduction",
        "pages_count",
        "industry",
        "cover",
        "download_url",
    ])?;

    for research_report in research_report_list.iter() {
        csv_writer.write_record(&[
            research_report.id.to_string(),
            research_report.title.to_string(),
            research_report.report_time.to_string(),
            research_report.introduction.to_string(),
            research_report.pages_count.to_string(),
            research_report.industry.to_string(),
            research_report.cover.to_string(),
            research_report.download_url.to_string(),
        ])?;
    }

    csv_writer.flush()?;

    Ok(())
}
