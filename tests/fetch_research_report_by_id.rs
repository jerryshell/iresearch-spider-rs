use iresearch_spider_rs::*;

#[tokio::test]
async fn test_fetch_research_report_by_id() {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();
    let research_report = fetch_research_report_by_id(client, 3922).await.unwrap();
    assert_eq!(research_report.title, "2022年数据库云管平台白皮书");
    assert_ne!(research_report.id, 3939);
}
