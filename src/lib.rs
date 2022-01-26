use anyhow::anyhow;
use anyhow::Result;
use futures::StreamExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;

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
    id: usize,
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

#[derive(Debug, Clone)]
pub struct RearchReport {
    id: usize,
    title: String,
    report_time: String,
    introduction: String,
    pages_count: i64,
    industry: String,
    cover: String,
    download_url: String,
}

async fn fetch_research_report_by_id(client: reqwest::Client, id: usize) -> Result<RearchReport> {
    let url = format!(
        "https://www.iresearch.com.cn/api/Detail/reportM?id={}&isfree=0",
        id
    );
    // println!("url {}", url);

    let response_payload = client
        .get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await?
        .json::<ResponsePayload>()
        .await?;
    // println!("{:#?}", response_payload);

    if response_payload.list.is_empty() {
        return Err(anyhow!("response_payload.list.is_empty()"));
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

    Ok(research_report)
}

pub async fn fetch_research_report_list_by_id_range(
    id_range: (usize, usize),
    parallel_requests: usize,
    connect_timeout: u64,
) -> Result<Vec<RearchReport>> {
    let id_list = (id_range.0..id_range.1).collect::<Vec<usize>>();
    let id_list_len = id_list.len();

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(connect_timeout))
        .build()?;

    let fetch_research_report_join_handle_list = futures::stream::iter(id_list)
        .map(|id| {
            let client = client.clone();
            tokio::spawn(fetch_research_report_by_id(client, id))
        })
        .buffer_unordered(parallel_requests);

    let success_count_arc = Arc::new(Mutex::new(0));
    let fail_count_arc = Arc::new(Mutex::new(0));
    let research_report_list_arc = Arc::new(Mutex::new(vec![]));

    fetch_research_report_join_handle_list
        .for_each(|join_handle| {
            let research_report_list_arc = research_report_list_arc.clone();
            let success_count_arc = success_count_arc.clone();
            let fail_count_arc = fail_count_arc.clone();
            async move {
                let mut success_count = success_count_arc.lock().unwrap();
                let mut fail_count = fail_count_arc.lock().unwrap();
                match join_handle {
                    Ok(fetch_result) => {
                        *success_count += 1;
                        let total_count = *success_count + *fail_count;
                        let progress = total_count as f32 / id_list_len as f32 * 100f32;
                        println!(
                            "success_count: {}, fail_cout: {}, progress: {:.2}%",
                            success_count, fail_count, progress
                        );
                        if let Ok(research_report) = fetch_result {
                            research_report_list_arc
                                .lock()
                                .unwrap()
                                .push(research_report);
                        }
                    }
                    Err(e) => {
                        *fail_count += 1;
                        let total_count = *success_count + *fail_count;
                        let progress = total_count as f32 / id_list_len as f32 * 100f32;
                        println!(
                            "success_count: {}, fail_cout: {}, progress: {:.2}%",
                            success_count, fail_count, progress
                        );
                        eprintln!("{:#?}", e)
                    }
                }
            }
        })
        .await;

    let mut research_report_list = research_report_list_arc.lock().unwrap().to_vec();
    research_report_list.sort_by(|r1, r2| r1.id.cmp(&r2.id));

    Ok(research_report_list)
}

pub async fn write_to_csv(research_report_list: &[RearchReport]) -> Result<()> {
    println!("write to csv...");

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
